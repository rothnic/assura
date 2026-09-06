//! Built-in, editable language-layout policy recipes.

pub const RUST_LIBRARY_STARTER_CONFIG: &str = r#"version: "2.0"

structure:
  ./:
    Cargo.toml: exists:1
    src/: exists:1
    tests/: exists:1
    .rs: snake_case
  src/:
    lib.rs: exists:1
    .rs: snake_case
  tests/:
    .rs: snake_case
"#;

pub const TYPESCRIPT_BUN_UTILITY_STARTER_CONFIG: &str = r#"version: "2.0"

structure:
  ./:
    package.json: exists:1
    src/: exists:1
    test/: exists:1
  src/:
    .ts: snake_case
    components/:
      exists: 0-1
      .tsx: PascalCase
  test/:
    .test.ts: snake_case
"#;

pub const PYTHON_PYTEST_STARTER_CONFIG: &str = r#"version: "2.0"

structure:
  ./:
    pyproject.toml: exists:1
    src/: exists:1
    tests/: exists:1
  src/:
    .py: snake_case
    __init__.py: exists:0-1
    ./*/:
      __init__.py: exists:0-1
  tests/:
    .py: snake_case
"#;

#[cfg(test)]
mod tests {
    use crate::cli::args::InitRecipe;
    use crate::cli::init_support::starter_config;
    use crate::config::config::ConfigLoader;

    #[test]
    fn selected_recipes_materialize_valid_project_owned_yaml() {
        for recipes in [
            vec![InitRecipe::AgenticCore],
            vec![InitRecipe::StructureHealth],
            vec![InitRecipe::AgenticCore, InitRecipe::StructureHealth],
            vec![InitRecipe::RustLibrary],
            vec![InitRecipe::TypescriptBunUtility],
            vec![InitRecipe::PythonPytest],
        ] {
            let source = starter_config(false, &recipes, None).unwrap();
            let config = ConfigLoader::parse(&source).unwrap();
            assert!(config.structure.contains_key("./"));
            assert!(!source.contains("$agentic-project"));
        }
    }
}
