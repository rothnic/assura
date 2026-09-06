use std::fs;
use std::process::Command;

use tempfile::TempDir;

#[cfg(unix)]
#[test]
fn minimal_launcher_distinguishes_good_missing_and_unlaunchable_companions() {
    let bundle = TempDir::new().expect("bundle directory");
    let target = TempDir::new().expect("minimal build target");
    let build = Command::new(env!("CARGO"))
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .args([
            "build",
            "--bin",
            "assura",
            "--no-default-features",
            "--features",
            "json-output,yaml-config",
            "--target-dir",
        ])
        .arg(target.path())
        .status()
        .expect("build minimal launcher");
    assert!(build.success(), "minimal launcher build failed: {build}");

    let launcher = bundle.path().join("assura");
    let companion = bundle.path().join("assura-full");
    fs::copy(target.path().join("debug/assura"), &launcher).expect("copy minimal launcher");

    fs::copy(env!("CARGO_BIN_EXE_assura-full"), &companion).expect("copy full companion");
    assert!(Command::new(&launcher)
        .arg("--version")
        .status()
        .unwrap()
        .success());

    fs::remove_file(&companion).expect("remove companion");
    let missing = Command::new(&launcher).arg("agent").output().unwrap();
    assert_eq!(missing.status.code(), Some(127));
    assert!(String::from_utf8_lossy(&missing.stderr).contains("was not found"));

    fs::write(&companion, "not executable\n").expect("write invalid companion");
    let unlaunchable = Command::new(&launcher).arg("agent").output().unwrap();
    assert_eq!(unlaunchable.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&unlaunchable.stderr).contains("failed to launch companion"));

    assert!(Command::new(&launcher)
        .args(["check", "--help"])
        .status()
        .unwrap()
        .success());
}
