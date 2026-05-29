//! Counterexample regression fixtures for LS-Lint performance evidence.

use super::fixture_io::{write_file, write_lslint_compatible_configs};
use std::fs;
use std::path::Path;

pub(super) fn create_multipart_extension_regression_project(root: &Path) -> Result<(), String> {
    write_lslint_compatible_configs(
        root,
        r#"
ignore:
  - .assura
ls:
  .a.b.c.d.e.f.g.h.i.j.k.js: kebabcase
"#,
    )?;

    fs::create_dir(root.join("src")).map_err(|error| format!("create src: {error}"))?;
    for index in 0..1500 {
        write_file(
            root.join(format!("src/file-{index:04}.a.b.c.d.e.f.g.h.i.j.k.js")),
            "",
        )?;
    }
    Ok(())
}

pub(super) fn create_many_configured_scopes_regression_project(root: &Path) -> Result<(), String> {
    let mut ls_lint_config = String::from(
        r#"
ignore:
  - .assura
ls:
  .dir: kebab-case
"#,
    );
    for index in 0..800 {
        ls_lint_config.push_str(&format!("  pkg-{index:04}:\n    .js: kebab-case\n"));
    }
    write_lslint_compatible_configs(root, &ls_lint_config)?;

    for index in 0..800 {
        let dir = root.join(format!("pkg-{index:04}"));
        fs::create_dir(&dir).map_err(|error| format!("create {}: {error}", dir.display()))?;
        write_file(dir.join("file-name.js"), "")?;
    }
    Ok(())
}
