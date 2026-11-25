use std::fs;
use std::path::Path;
use std::process::Command;

#[test]
fn portable_self_directory_counts_match_live_checks() {
    let mut mismatches = Vec::new();
    for scope in ["plugin-core/", "plugin-*/"] {
        for expected in [0, 1] {
            let project = tempfile::tempdir().unwrap();
            let root = project.path();
            fs::create_dir(root.join(".assura")).unwrap();
            fs::write(
                root.join(".assura/config.yml"),
                format!(
                    r#"
structure:
  ./:
    extra: true
    children:
      "{scope}":
        self_directory:
          exists:
            "*": "{expected}"
"#
                ),
            )
            .unwrap();
            let compile = Command::new(env!("CARGO_BIN_EXE_assura-check-compile-config"))
                .args([
                    "--config",
                    ".assura/config.yml",
                    "--output",
                    ".assura/check-config.bin",
                ])
                .current_dir(root)
                .output()
                .unwrap();
            assert!(compile.status.success(), "compiler: {compile:?}");

            let directory = root.join("plugin-core");
            for state in ["absent", "empty", "populated"] {
                let actual = match state {
                    "absent" => 0,
                    "empty" => {
                        fs::create_dir(&directory).unwrap();
                        1
                    }
                    _ => {
                        fs::write(directory.join("entry.rs"), "").unwrap();
                        1
                    }
                };
                let should_fail = actual != expected;
                for (mode, binary, args) in [
                    ("live", env!("CARGO_BIN_EXE_assura-check"), &[][..]),
                    (
                        "portable",
                        env!("CARGO_BIN_EXE_assura-check-compiled"),
                        &[][..],
                    ),
                    (
                        "artifact-only",
                        env!("CARGO_BIN_EXE_assura-check-compiled"),
                        &["--compiled-config", ".assura/check-config.bin"][..],
                    ),
                ] {
                    let (status, counts, report) = check_counts(binary, args, root);
                    if status != Some(i32::from(should_fail))
                        || counts.len() != usize::from(should_fail)
                        || counts.iter().any(|line| {
                            !line.contains(&format!("exists {actual} times, expected {expected}"))
                        })
                    {
                        mismatches.push(format!(
                            "{scope}, expected {expected}, {state}, {mode}, exit {status:?}:\n{report}"
                        ));
                    }
                }
            }
        }
    }

    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}

fn check_counts(binary: &str, args: &[&str], root: &Path) -> (Option<i32>, Vec<String>, String) {
    let check = Command::new(binary)
        .args(args)
        .current_dir(root)
        .output()
        .unwrap();
    let stdout = String::from_utf8(check.stdout).unwrap();
    let counts = stdout
        .lines()
        .filter(|line| line.contains(":exists_count]"))
        .map(str::to_owned)
        .collect();
    let stderr = String::from_utf8_lossy(&check.stderr);
    (check.status.code(), counts, format!("{stdout}{stderr}"))
}
