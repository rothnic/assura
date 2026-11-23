//! Render findings and replay guidance for Git snapshot checks.

use super::{Action, Report};
use std::path::Path;

impl Report {
    pub(super) fn print_text(&self) {
        println!(
            "{} max_lines check: {} → {} ({}, {} files checked)",
            self.scope,
            self.baseline.tree,
            self.selected.tree,
            if self.success { "passed" } else { "failed" },
            self.files_checked,
        );
        for source in &self.policies {
            println!(
                "  {} policy: {:?} ({:?}, tree {:?}, sha256 {})",
                source.layer, source.path, source.origin, source.tree, source.sha256
            );
        }
        for exemption in &self.exemptions {
            println!(
                "  {} exemption {:?} applies to {:?}",
                exemption.layer, exemption.pattern, exemption.path
            );
        }
        for finding in &self.findings {
            println!(
                "[{}/{}] {:?}: {} lines exceeds the allowed {} (configured limit {}). Lines to cut or move: {}.",
                finding.layer, finding.severity, finding.path, finding.actual, finding.allowed_lines, finding.limit,
                finding.overage,
            );
            if let Some(message) = &finding.repair_message {
                println!("  Policy message: {}", message.escape_debug());
            }
        }
        for instruction in &self.guidance {
            println!("  {instruction}");
        }
        for action in &self.actions {
            println!("  {}\n    {}", action.id, action.command);
        }
    }

    pub(super) fn add_guidance(&mut self, cwd: &Path, root: &Path, alternate_index: bool) {
        self.guidance.push(
            "Split the file or move lines to fit. Ask the policy owner about a legitimate exception.",
        );
        if self
            .policies
            .iter()
            .any(|policy| policy.layer == "scoped_owner")
        {
            self.guidance.push(
                "Select the matching scoped owner policy through Git config (assura.ownerPolicy).",
            );
        }
        self.guidance.push(
            "For CI, match the reported policy layers and pin each SHA-256. Update policy paths for the CI checkout and provision external files separately.",
        );
        if let Some(replay) = self.action(
            "replay",
            cwd,
            root,
            &format!(
                "--base {} --head {}",
                self.baseline.tree, self.selected.tree
            ),
        ) {
            self.actions.push(replay);
        }
        if self.scope == "worktree" {
            self.guidance
                .push("Edit the working files and recheck against the same base commit.");
            if let Some(base) = &self.baseline.commit {
                if let Some(recheck) =
                    self.action("recheck", cwd, root, &format!("--worktree {base}"))
                {
                    self.actions.push(recheck);
                }
            }
        } else if self.scope != "staged" {
            self.guidance.push(
                "After committing the repair, run check-change with the reported baseline tree as --base and the repaired commit as --head.",
            );
        } else if alternate_index {
            self.guidance.push(
                "After repairing, repeat the Git operation or index setup that supplied the active index. For a path-limited commit, repeat the same path selection.",
            );
        } else {
            self.guidance.push(
                "Stage the repaired files and run check-change --staged before advancing HEAD.",
            );
            if let Some(recheck) = self.action("recheck", cwd, root, "--staged") {
                self.actions.push(recheck);
            }
        }
    }

    fn action(
        &self,
        id: &'static str,
        cwd: &Path,
        root: &Path,
        comparison: &str,
    ) -> Option<Action> {
        #[cfg(unix)]
        fn quote(path: &Path) -> Option<String> {
            let path_text = path.to_str()?;
            if path_text.chars().any(char::is_control) {
                return None;
            }
            Some(format!("'{}'", path_text.replace('\'', "'\\''")))
        }
        #[cfg(not(unix))]
        fn quote(_path: &Path) -> Option<String> {
            None
        }

        let mut command = format!("(cd {} && assura check-change {comparison}", quote(cwd)?);
        for policy in &self.policies {
            match policy.layer {
                "user_baseline" => {
                    command.push_str(&format!(" --owner-baseline {}", quote(&policy.path)?))
                }
                "project" => {
                    command.push_str(&format!(" --config {}", quote(&root.join(&policy.path))?))
                }
                _ => {}
            }
            command.push_str(&format!(
                " --require-policy-digest {}={}",
                policy.layer, policy.sha256
            ));
        }
        command.push(')');
        Some(Action {
            id,
            cwd: cwd.to_path_buf(),
            command,
        })
    }
}
