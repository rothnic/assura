//! Platform-specific process spawning for the managed local daemon.

use std::process::{Child, Command};

#[cfg(windows)]
use windows_sys::Win32::{
    Foundation::{
        GetHandleInformation, SetHandleInformation, HANDLE, HANDLE_FLAG_INHERIT,
        INVALID_HANDLE_VALUE,
    },
    System::Console::{GetStdHandle, STD_ERROR_HANDLE, STD_INPUT_HANDLE, STD_OUTPUT_HANDLE},
};

pub(super) fn without_inherited_parent_stdio(command: &mut Command) -> std::io::Result<Child> {
    #[cfg(windows)]
    let _guard = ParentStdioInheritanceGuard::disable()?;

    command.spawn()
}

#[cfg(windows)]
struct ParentStdioInheritanceGuard {
    inherited: Vec<HANDLE>,
}

#[cfg(windows)]
impl ParentStdioInheritanceGuard {
    fn disable() -> std::io::Result<Self> {
        let mut guard = Self {
            inherited: Vec::new(),
        };
        for id in [STD_INPUT_HANDLE, STD_OUTPUT_HANDLE, STD_ERROR_HANDLE] {
            // SAFETY: GetStdHandle returns a process-owned handle that remains valid for
            // the lifetime of this short guard. The handle is never closed here.
            let handle = unsafe { GetStdHandle(id) };
            if handle.is_null() || handle == INVALID_HANDLE_VALUE {
                continue;
            }
            let mut flags = 0;
            // SAFETY: flags points to writable storage and handle was checked above.
            if unsafe { GetHandleInformation(handle, &mut flags) } == 0 {
                return Err(std::io::Error::last_os_error());
            }
            if flags & HANDLE_FLAG_INHERIT != 0 {
                // SAFETY: this changes only the inheritance bit on a valid process handle.
                if unsafe { SetHandleInformation(handle, HANDLE_FLAG_INHERIT, 0) } == 0 {
                    return Err(std::io::Error::last_os_error());
                }
                guard.inherited.push(handle);
            }
        }
        Ok(guard)
    }
}

#[cfg(windows)]
impl Drop for ParentStdioInheritanceGuard {
    fn drop(&mut self) {
        for handle in &self.inherited {
            // SAFETY: each handle came from GetStdHandle and is still process-owned.
            let _ =
                unsafe { SetHandleInformation(*handle, HANDLE_FLAG_INHERIT, HANDLE_FLAG_INHERIT) };
        }
    }
}
