#![cfg(unix)]

use std::fs;
use std::process::Command;
use tempfile::TempDir;

#[test]
fn max_lines_feedback_commands_replay_the_selected_policy() {
    let project = TempDir::new().unwrap();
    fs::create_dir(project.path().join(".assura")).unwrap();
    fs::write(project.path().join(".assura/config.yml"), "structure: {}\n").unwrap();
    let config = ".assura/team budget.yml";
    fs::write(
        project.path().join(config),
        "structure:\n  ./:\n    files:\n      max_lines: 2\nexclude:\n  - .assura/**\n",
    )
    .unwrap();
    let file = "worker's notes.md";
    fs::write(project.path().join(file), "one\ntwo\nthree\nfour\n").unwrap();
    fs::write(project.path().join("another file.md"), "one\ntwo\nthree\n").unwrap();
    let check = |args: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_assura"))
            .args(["check", "--config", config])
            .args(args)
            .current_dir(project.path())
            .output()
            .unwrap()
    };

    let agent = check(&["--format", "agent", file]);
    assert_eq!(agent.status.code(), Some(1));
    let payload: serde_json::Value = serde_json::from_slice(&agent.stdout).unwrap();
    let finding = &payload["feedback"][0]["messages"][0];
    assert_eq!(finding["path"], file);
    assert_eq!(finding["line_budget"]["actual"], 4);
    assert_eq!(finding["line_budget"]["limit"], 2);
    assert_eq!(finding["line_budget"]["overage"], 2);
    let actions = finding["line_budget"]["actions"].as_array().unwrap();
    let [explain, recheck] = actions.as_slice() else {
        panic!("expected exactly two actions: {actions:?}");
    };
    assert_eq!(explain["id"], "explain");
    assert_eq!(recheck["id"], "recheck");
    let explain_command = explain["command"].as_str().unwrap();
    let recheck_command = recheck["command"].as_str().unwrap();

    for format in ["text", "advice", "agent"] {
        let mut args = vec!["--format", format, file];
        if format == "agent" {
            args.extend(["--agent", "codex", "--warn"]);
        }
        let output = check(&args);
        assert_eq!(output.status.code(), Some(i32::from(format != "agent")));
        let text = if format == "agent" {
            let hook: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
            hook["hookSpecificOutput"]["additionalContext"]
                .as_str()
                .unwrap()
                .to_string()
        } else {
            String::from_utf8(output.stdout).unwrap()
        };
        assert!(text.contains("has 4 lines, exceeding limit 2"));
        assert!(text.contains("Lines to cut or move: 2"));
        assert!(text.contains("ask the policy owner"));
        assert!(text.find(explain_command).unwrap() < text.find(recheck_command).unwrap());
    }
    let status = check(&["--format", "status", file]);
    let status = String::from_utf8(status.stdout).unwrap();
    assert_eq!(status.lines().count(), 1);
    assert!(!status.contains("assura explain"));

    let verbose = check(&["--verbose", file]);
    assert_eq!(verbose.status.code(), Some(1));
    let verbose = String::from_utf8_lossy(&verbose.stdout);
    assert!(verbose.find(explain_command).unwrap() < verbose.find(recheck_command).unwrap());
    let raw = check(&["--format", "json", file]);
    assert_eq!(raw.status.code(), Some(1));
    let raw: serde_json::Value = serde_json::from_slice(&raw.stdout).unwrap();
    assert_eq!(raw["violations"][0]["metadata"], finding["line_budget"]);

    let bin_dir = std::path::Path::new(env!("CARGO_BIN_EXE_assura"))
        .parent()
        .unwrap();
    let search_path = std::env::join_paths(
        std::iter::once(bin_dir.to_path_buf())
            .chain(std::env::split_paths(&std::env::var_os("PATH").unwrap())),
    )
    .unwrap();
    let replay = |action: &serde_json::Value| {
        Command::new("sh")
            .args(["-c", action["command"].as_str().unwrap()])
            .current_dir(action["cwd"].as_str().unwrap())
            .env("PATH", &search_path)
            .output()
            .unwrap()
    };
    let explained = replay(explain);
    assert!(explained.status.success(), "{explained:?}");
    assert!(String::from_utf8_lossy(&explained.stdout).contains(file));
    let failing = replay(recheck);
    assert_eq!(failing.status.code(), Some(1), "{failing:?}");
    assert!(String::from_utf8_lossy(&failing.stdout).contains("exceeding limit 2"));
    assert!(!String::from_utf8_lossy(&failing.stdout).contains("another file.md"));
    fs::write(project.path().join(file), "one\ntwo\n").unwrap();
    let passing = replay(recheck);
    assert!(passing.status.success(), "{passing:?}");
}
