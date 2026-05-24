//! Release-binary profile labeling for performance reports.

use std::path::Path;
#[cfg(target_os = "linux")]
use std::process::{Command, Stdio};

pub(super) fn assura_binary_profile(binary_path: &Path) -> String {
    let profile = if binary_path
        .components()
        .any(|component| component.as_os_str() == "release")
    {
        "release".to_string()
    } else if binary_path
        .components()
        .any(|component| component.as_os_str() == "debug")
    {
        "debug".to_string()
    } else {
        "unknown".to_string()
    };

    if cfg!(target_os = "linux") && linux_binary_is_static(binary_path) {
        format!("{profile}-static-crt")
    } else {
        profile
    }
}

#[cfg(target_os = "linux")]
fn linux_binary_is_static(binary_path: &Path) -> bool {
    let Ok(output) = Command::new("ldd")
        .arg(binary_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
    else {
        return false;
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    stdout.contains("statically linked")
        || stderr.contains("statically linked")
        || stdout.contains("not a dynamic executable")
        || stderr.contains("not a dynamic executable")
}

#[cfg(not(target_os = "linux"))]
fn linux_binary_is_static(_binary_path: &Path) -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::assura_binary_profile;

    #[test]
    fn binary_profile_uses_release_path_component() {
        let path = std::path::Path::new("target/release/assura-check");

        assert!(assura_binary_profile(path).starts_with("release"));
    }
}
