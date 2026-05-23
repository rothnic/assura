use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::Path;
use std::process::{Child, Command, Stdio};

fn write_project(root: &Path, file_name: &str) {
    fs::create_dir_all(root.join(".assura")).unwrap();
    fs::create_dir_all(root.join("src")).unwrap();
    write_config(&root.join(".assura/config.yml"), "kebab-case");
    fs::write(root.join("src").join(file_name), "").unwrap();
}

fn write_config(path: &Path, naming: &str) {
    fs::write(
        path,
        r#"
structure:
  ./:
    files:
      naming_patterns:
        "*.ts": NAMING
    directories:
      naming: NAMING
    children:
      .assura/:
        inherit: false
        files:
          naming: NAMING
exclude:
  - ".assura/**"
"#
        .replace("NAMING", naming),
    )
    .unwrap();
}

fn daemon_listen_arg(temp: &tempfile::TempDir, name: &str) -> String {
    #[cfg(unix)]
    {
        format!("unix:{}", temp.path().join(name).display())
    }
    #[cfg(not(unix))]
    {
        let _ = temp;
        let _ = name;
        "127.0.0.1:0".to_string()
    }
}

fn read_daemon_addr_or_skip(server: &mut Child) -> Option<String> {
    let stdout = server.stdout.take().unwrap();
    let mut reader = BufReader::new(stdout);
    let mut addr = String::new();
    reader.read_line(&mut addr).unwrap();
    let addr = addr.trim().to_string();
    if !addr.is_empty() {
        return Some(addr);
    }

    let mut stderr = String::new();
    if let Some(stderr_pipe) = server.stderr.take() {
        let mut reader = BufReader::new(stderr_pipe);
        let _ = reader.read_to_string(&mut stderr);
    }
    if stderr.contains("Operation not permitted") {
        let _ = server.wait();
        return None;
    }
    panic!("assura-checkd exited before publishing an address:\n{stderr}");
}

#[test]
fn hot_client_can_validate_one_changed_path_without_project_check() {
    let temp = tempfile::tempdir().unwrap();
    let project = temp.path().join("hot-project");
    write_project(&project, "bad_name.ts");
    fs::write(project.join("src").join("good-file.ts"), "").unwrap();

    let mut server = Command::new(env!("CARGO_BIN_EXE_assura-checkd"))
        .arg("--listen")
        .arg(daemon_listen_arg(&temp, "hot-project.sock"))
        .arg("--root")
        .arg(&project)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    let Some(addr) = read_daemon_addr_or_skip(&mut server) else {
        return;
    };

    let valid = Command::new(env!("CARGO_BIN_EXE_assura-check-client"))
        .arg(&addr)
        .arg(project.join("src").join("good-file.ts"))
        .output()
        .unwrap();
    assert!(
        valid.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&valid.stdout),
        String::from_utf8_lossy(&valid.stderr)
    );

    let invalid = Command::new(env!("CARGO_BIN_EXE_assura-check-client"))
        .arg(&addr)
        .arg(project.join("src").join("bad_name.ts"))
        .output()
        .unwrap();
    assert_eq!(invalid.status.code(), Some(1));

    let _ = server.kill();
    let _ = server.wait();
}

#[test]
fn hot_client_can_check_project_from_explicit_dirty_path() {
    let temp = tempfile::tempdir().unwrap();
    let project = temp.path().join("hot-dirty-project");
    write_project(&project, "valid-file.ts");

    let mut server = Command::new(env!("CARGO_BIN_EXE_assura-checkd"))
        .arg("--listen")
        .arg(daemon_listen_arg(&temp, "hot-dirty-project.sock"))
        .arg("--root")
        .arg(&project)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    let Some(addr) = read_daemon_addr_or_skip(&mut server) else {
        return;
    };

    let warm = Command::new(env!("CARGO_BIN_EXE_assura-check-client"))
        .arg(&addr)
        .output()
        .unwrap();
    assert!(
        warm.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&warm.stdout),
        String::from_utf8_lossy(&warm.stderr)
    );

    let dirty_path = project.join("src").join("bad_name.ts");
    fs::write(&dirty_path, "").unwrap();

    let invalid = Command::new(env!("CARGO_BIN_EXE_assura-check-client"))
        .arg(&addr)
        .arg("--dirty-project-path")
        .arg(&dirty_path)
        .output()
        .unwrap();
    assert_eq!(invalid.status.code(), Some(1));

    let _ = server.kill();
    let _ = server.wait();
}

#[cfg(unix)]
#[test]
fn unix_hot_client_uses_compact_project_check_protocol() {
    let temp = tempfile::tempdir().unwrap();
    let project = temp.path().join("hot-unix-project");
    let socket = temp.path().join("assura-checkd.sock");
    write_project(&project, "valid-file.ts");

    let mut server = Command::new(env!("CARGO_BIN_EXE_assura-checkd"))
        .arg("--listen")
        .arg(format!("unix:{}", socket.display()))
        .arg("--root")
        .arg(&project)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    let Some(addr) = read_daemon_addr_or_skip(&mut server) else {
        return;
    };
    assert_eq!(addr, format!("unix:{}", socket.display()));

    let valid = Command::new(env!("CARGO_BIN_EXE_assura-check-unix-client"))
        .arg(&addr)
        .output()
        .unwrap();
    assert!(
        valid.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&valid.stdout),
        String::from_utf8_lossy(&valid.stderr)
    );

    let dirty_path = project.join("src").join("bad_name.ts");
    fs::write(&dirty_path, "").unwrap();
    let invalid = Command::new(env!("CARGO_BIN_EXE_assura-check-unix-client"))
        .arg(&addr)
        .arg("--dirty-project-path")
        .arg(&dirty_path)
        .output()
        .unwrap();
    assert_eq!(invalid.status.code(), Some(1));

    let _ = server.kill();
    let _ = server.wait();

    let invalid_project = temp.path().join("hot-unix-invalid-project");
    let invalid_socket = temp.path().join("assura-checkd-invalid.sock");
    write_project(&invalid_project, "bad_name.ts");

    let mut server = Command::new(env!("CARGO_BIN_EXE_assura-checkd"))
        .arg("--listen")
        .arg(format!("unix:{}", invalid_socket.display()))
        .arg("--root")
        .arg(&invalid_project)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    let Some(addr) = read_daemon_addr_or_skip(&mut server) else {
        return;
    };
    assert_eq!(addr, format!("unix:{}", invalid_socket.display()));

    let invalid = Command::new(env!("CARGO_BIN_EXE_assura-check-unix-client"))
        .arg(&addr)
        .output()
        .unwrap();
    assert_eq!(invalid.status.code(), Some(1));

    let _ = server.kill();
    let _ = server.wait();
}

#[test]
fn session_client_reuses_one_cli_process_for_dirty_project_checks() {
    let temp = tempfile::tempdir().unwrap();
    let project = temp.path().join("hot-session-project");
    write_project(&project, "valid-file.ts");

    let mut server = Command::new(env!("CARGO_BIN_EXE_assura-checkd"))
        .arg("--listen")
        .arg(daemon_listen_arg(&temp, "hot-session-project.sock"))
        .arg("--root")
        .arg(&project)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    let Some(addr) = read_daemon_addr_or_skip(&mut server) else {
        return;
    };

    let mut session = Command::new(env!("CARGO_BIN_EXE_assura-check-session"))
        .arg(&addr)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdin = session.stdin.take().unwrap();
    let stdout = session.stdout.take().unwrap();
    let mut session_reader = BufReader::new(stdout);

    writeln!(stdin, "CHECK").unwrap();
    stdin.flush().unwrap();
    let mut response = String::new();
    session_reader.read_line(&mut response).unwrap();
    assert_eq!(response.trim(), "OK 0");

    let dirty_path = project.join("src").join("bad_name.ts");
    fs::write(&dirty_path, "").unwrap();
    writeln!(
        stdin,
        "DIRTY-PROJECT-PATH\t{}",
        dirty_path.to_string_lossy()
    )
    .unwrap();
    stdin.flush().unwrap();
    response.clear();
    session_reader.read_line(&mut response).unwrap();
    assert_eq!(response.trim(), "OK 1");

    writeln!(stdin, "QUIT").unwrap();
    stdin.flush().unwrap();
    assert!(session.wait().unwrap().success());

    let _ = server.kill();
    let _ = server.wait();
}

#[test]
fn session_check_reloads_external_config_without_watcher_event() {
    let temp = tempfile::tempdir().unwrap();
    let project = temp.path().join("hot-session-external-config-project");
    let external_config = temp.path().join("external-config.yml");
    let status_file = temp.path().join("assura-check.status");
    write_project(&project, "bad_name.ts");
    write_config(&external_config, "kebab-case");

    let mut server = Command::new(env!("CARGO_BIN_EXE_assura-checkd"))
        .arg("--listen")
        .arg(daemon_listen_arg(
            &temp,
            "hot-session-external-config-project.sock",
        ))
        .arg("--root")
        .arg(&project)
        .arg("--config")
        .arg(&external_config)
        .arg("--status-file")
        .arg(&status_file)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    let Some(addr) = read_daemon_addr_or_skip(&mut server) else {
        return;
    };

    let mut session = Command::new(env!("CARGO_BIN_EXE_assura-check-session"))
        .arg(&addr)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdin = session.stdin.take().unwrap();
    let stdout = session.stdout.take().unwrap();
    let mut session_reader = BufReader::new(stdout);
    let mut response = String::new();

    writeln!(stdin, "CHECK").unwrap();
    stdin.flush().unwrap();
    session_reader.read_line(&mut response).unwrap();
    assert_eq!(response.trim(), "OK 1");

    write_config(&external_config, "snake_case");
    writeln!(
        stdin,
        "PATH\t{}",
        project.join("src").join("bad_name.ts").display()
    )
    .unwrap();
    stdin.flush().unwrap();
    response.clear();
    session_reader.read_line(&mut response).unwrap();
    assert_eq!(response.trim(), "OK 0");

    let dirty_status = Command::new(env!("CARGO_BIN_EXE_assura-check-status"))
        .arg(&status_file)
        .output()
        .unwrap();
    assert_eq!(dirty_status.status.code(), Some(3));

    writeln!(stdin, "CHECK").unwrap();
    stdin.flush().unwrap();
    response.clear();
    session_reader.read_line(&mut response).unwrap();
    assert_eq!(response.trim(), "OK 0");

    let clean_status = Command::new(env!("CARGO_BIN_EXE_assura-check-status"))
        .arg(&status_file)
        .output()
        .unwrap();
    assert!(clean_status.status.success());

    writeln!(stdin, "QUIT").unwrap();
    stdin.flush().unwrap();
    assert!(session.wait().unwrap().success());

    let _ = server.kill();
    let _ = server.wait();
}
