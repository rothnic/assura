//! Platform-specific process spawning for the managed local daemon.

use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};

/// Child-process handle used while the daemon publishes its readiness artifact.
pub(super) struct DaemonChild {
    #[cfg(not(windows))]
    inner: std::process::Child,
    #[cfg(windows)]
    process: std::os::windows::io::OwnedHandle,
    #[cfg(windows)]
    pid: u32,
}

/// Keeps detached standard streams alive for the daemon process lifetime.
pub(super) struct DetachedStdio {
    #[cfg(windows)]
    _stdin: std::fs::File,
    #[cfg(windows)]
    _stdout: std::fs::File,
    #[cfg(windows)]
    _stderr: std::fs::File,
}

pub(super) fn install_detached_stdio(log_file: Option<&Path>) -> io::Result<DetachedStdio> {
    #[cfg(not(windows))]
    {
        let _ = log_file;
        Ok(DetachedStdio {})
    }

    #[cfg(windows)]
    {
        install_windows_stdio(log_file)
    }
}

pub(super) fn spawn_without_inherited_handles(command: &mut Command) -> io::Result<DaemonChild> {
    #[cfg(not(windows))]
    {
        command.spawn().map(|inner| DaemonChild { inner })
    }

    #[cfg(windows)]
    spawn_windows(command)
}

pub(super) fn serve_managed_daemon(
    project_root: PathBuf,
    config: Option<PathBuf>,
    listen_addr: String,
    ready_file: Option<PathBuf>,
    log_file: Option<PathBuf>,
) -> Result<(), String> {
    let detached_stdio = install_detached_stdio(log_file.as_deref())
        .map_err(|error| format!("prepare detached daemon streams: {error}"))?;
    #[cfg(windows)]
    std::mem::forget(detached_stdio);
    #[cfg(not(windows))]
    let _detached_stdio = detached_stdio;

    let result = super::process::serve_daemon(project_root, config, listen_addr, ready_file);
    if let Err(message) = &result {
        append_detached_daemon_error(log_file.as_deref(), message);
    }
    result
}

fn append_detached_daemon_error(log_file: Option<&Path>, message: &str) {
    #[cfg(windows)]
    if let Some(log_file) = log_file {
        use std::io::Write;

        if let Ok(mut log) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_file)
        {
            let _ = writeln!(log, "serve-error: {message}");
        }
    }

    #[cfg(not(windows))]
    let _ = (log_file, message);
}

impl DaemonChild {
    pub(super) fn id(&self) -> u32 {
        #[cfg(not(windows))]
        {
            self.inner.id()
        }

        #[cfg(windows)]
        {
            self.pid
        }
    }

    pub(super) fn try_wait(&mut self) -> io::Result<Option<ExitStatus>> {
        #[cfg(not(windows))]
        {
            self.inner.try_wait()
        }

        #[cfg(windows)]
        {
            self.windows_exit_status()
        }
    }

    pub(super) fn kill(&mut self) -> io::Result<()> {
        #[cfg(not(windows))]
        {
            self.inner.kill()
        }

        #[cfg(windows)]
        {
            use std::os::windows::io::AsRawHandle;
            use windows_sys::Win32::System::Threading::TerminateProcess;

            // SAFETY: process is an owned process handle returned by CreateProcessW.
            if unsafe { TerminateProcess(self.process.as_raw_handle(), 1) } == 0 {
                Err(io::Error::last_os_error())
            } else {
                Ok(())
            }
        }
    }

    pub(super) fn wait(&mut self) -> io::Result<ExitStatus> {
        #[cfg(not(windows))]
        {
            self.inner.wait()
        }

        #[cfg(windows)]
        {
            use std::os::windows::io::AsRawHandle;
            use windows_sys::Win32::{
                Foundation::WAIT_FAILED,
                System::Threading::{WaitForSingleObject, INFINITE},
            };

            // SAFETY: process is an owned process handle returned by CreateProcessW.
            if unsafe { WaitForSingleObject(self.process.as_raw_handle(), INFINITE) } == WAIT_FAILED
            {
                return Err(io::Error::last_os_error());
            }
            self.windows_exit_status()?.ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::Other,
                    "daemon process remained active after its wait completed",
                )
            })
        }
    }

    #[cfg(windows)]
    fn windows_exit_status(&self) -> io::Result<Option<ExitStatus>> {
        use std::os::windows::{io::AsRawHandle, process::ExitStatusExt};
        use windows_sys::Win32::{Foundation::STILL_ACTIVE, System::Threading::GetExitCodeProcess};

        let mut code = 0;
        // SAFETY: process is an owned process handle and code points to writable storage.
        if unsafe { GetExitCodeProcess(self.process.as_raw_handle(), &mut code) } == 0 {
            return Err(io::Error::last_os_error());
        }
        if code == STILL_ACTIVE as u32 {
            Ok(None)
        } else {
            Ok(Some(ExitStatus::from_raw(code)))
        }
    }
}

#[cfg(windows)]
fn spawn_windows(command: &Command) -> io::Result<DaemonChild> {
    use std::mem::size_of;
    use std::os::windows::{ffi::OsStrExt, io::FromRawHandle};
    use std::ptr;
    use windows_sys::Win32::{
        Foundation::CloseHandle,
        System::Threading::{
            CreateProcessW, CREATE_NEW_PROCESS_GROUP, CREATE_NO_WINDOW, PROCESS_INFORMATION,
            STARTUPINFOW,
        },
    };

    if command.get_envs().next().is_some() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "managed Windows daemon launch does not accept environment overrides",
        ));
    }

    let application = command
        .get_program()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();
    if application[..application.len().saturating_sub(1)].contains(&0) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "daemon executable path contains a NUL character",
        ));
    }

    let mut command_line = Vec::new();
    append_quoted_arg(&mut command_line, command.get_program())?;
    for argument in command.get_args() {
        command_line.push(b' ' as u16);
        append_quoted_arg(&mut command_line, argument)?;
    }
    command_line.push(0);

    let current_dir = command
        .get_current_dir()
        .map(|path| {
            let mut encoded = path.as_os_str().encode_wide().collect::<Vec<_>>();
            if encoded.contains(&0) {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "daemon working directory contains a NUL character",
                ));
            }
            encoded.push(0);
            Ok(encoded)
        })
        .transpose()?;
    let current_dir_ptr = current_dir
        .as_ref()
        .map_or(ptr::null(), |encoded| encoded.as_ptr());

    let startup = STARTUPINFOW {
        cb: size_of::<STARTUPINFOW>() as u32,
        ..STARTUPINFOW::default()
    };
    let mut process = PROCESS_INFORMATION::default();
    // SAFETY: all string buffers are NUL-terminated and remain alive for this call;
    // output structures point to writable storage. Handle inheritance is disabled so
    // a long-lived daemon cannot retain the launcher's captured pipes.
    let created = unsafe {
        CreateProcessW(
            application.as_ptr(),
            command_line.as_mut_ptr(),
            ptr::null(),
            ptr::null(),
            0,
            CREATE_NO_WINDOW | CREATE_NEW_PROCESS_GROUP,
            ptr::null(),
            current_dir_ptr,
            &startup,
            &mut process,
        )
    };
    if created == 0 {
        return Err(io::Error::last_os_error());
    }

    // SAFETY: CreateProcessW returned both handles and ownership is transferred here.
    unsafe {
        let _ = CloseHandle(process.hThread);
        Ok(DaemonChild {
            process: std::os::windows::io::OwnedHandle::from_raw_handle(process.hProcess),
            pid: process.dwProcessId,
        })
    }
}

#[cfg(windows)]
fn install_windows_stdio(log_file: Option<&Path>) -> io::Result<DetachedStdio> {
    use std::fs::OpenOptions;
    use std::os::windows::io::AsRawHandle;
    use windows_sys::Win32::System::Console::{
        SetStdHandle, STD_ERROR_HANDLE, STD_INPUT_HANDLE, STD_OUTPUT_HANDLE,
    };

    let stdin = OpenOptions::new().read(true).open(r"\\.\NUL")?;
    let stdout = OpenOptions::new().write(true).open(r"\\.\NUL")?;
    let stderr = match log_file {
        Some(path) => OpenOptions::new().create(true).append(true).open(path)?,
        None => OpenOptions::new().write(true).open(r"\\.\NUL")?,
    };

    for (id, handle) in [
        (STD_INPUT_HANDLE, stdin.as_raw_handle()),
        (STD_OUTPUT_HANDLE, stdout.as_raw_handle()),
        (STD_ERROR_HANDLE, stderr.as_raw_handle()),
    ] {
        // SAFETY: every handle is owned by this guard and retained until serving ends.
        if unsafe { SetStdHandle(id, handle) } == 0 {
            return Err(io::Error::last_os_error());
        }
    }

    Ok(DetachedStdio {
        _stdin: stdin,
        _stdout: stdout,
        _stderr: stderr,
    })
}

#[cfg(windows)]
fn append_quoted_arg(output: &mut Vec<u16>, argument: &std::ffi::OsStr) -> io::Result<()> {
    use std::os::windows::ffi::OsStrExt;

    let argument = argument.encode_wide().collect::<Vec<_>>();
    if argument.contains(&0) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "daemon argument contains a NUL character",
        ));
    }

    output.push(b'"' as u16);
    let mut backslashes = 0;
    for unit in argument {
        if unit == b'\\' as u16 {
            backslashes += 1;
            continue;
        }
        if unit == b'"' as u16 {
            output.extend(std::iter::repeat(b'\\' as u16).take(backslashes * 2 + 1));
        } else {
            output.extend(std::iter::repeat(b'\\' as u16).take(backslashes));
        }
        backslashes = 0;
        output.push(unit);
    }
    output.extend(std::iter::repeat(b'\\' as u16).take(backslashes * 2));
    output.push(b'"' as u16);
    Ok(())
}
