//! Case-convention definitions and validation.

/// Case conventions for naming
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CaseConvention {
    /// lowercase (e.g., filename)
    LowerCase,
    /// UPPERCASE (e.g., FILENAME)
    UpperCase,
    /// snake_case (e.g., file_name)
    #[default]
    SnakeCase,
    /// camelCase (e.g., fileName)
    CamelCase,
    /// PascalCase (e.g., FileName)
    PascalCase,
    /// kebab-case (e.g., file-name)
    KebabCase,
    /// SCREAMING_SNAKE_CASE (e.g., FILE_NAME)
    ScreamingSnakeCase,
    /// dot.case (e.g., file.name)
    DotCase,
    /// flatcase (e.g., filename) - lowercase, no separators
    FlatCase,
    /// FLATCASE (e.g., FILENAME) - UPPERCASE, no separators
    ScreamingFlatCase,
    /// COBOL-CASE (e.g., FILE-NAME) - UPPERCASE with hyphens
    CobolCase,
    /// Train-Case (e.g., File-Name) - Title-Case with hyphens
    TrainCase,
}

impl CaseConvention {
    /// Get the name of this convention
    pub fn name(self) -> &'static str {
        match self {
            CaseConvention::LowerCase => "lowercase",
            CaseConvention::UpperCase => "UPPERCASE",
            CaseConvention::SnakeCase => "snake_case",
            CaseConvention::CamelCase => "camelCase",
            CaseConvention::PascalCase => "PascalCase",
            CaseConvention::KebabCase => "kebab-case",
            CaseConvention::ScreamingSnakeCase => "SCREAMING_SNAKE_CASE",
            CaseConvention::DotCase => "dot.case",
            CaseConvention::FlatCase => "flatcase",
            CaseConvention::ScreamingFlatCase => "FLATCASE",
            CaseConvention::CobolCase => "COBOL-CASE",
            CaseConvention::TrainCase => "Train-Case",
        }
    }

    /// Validate a string against this case convention
    pub fn validate(&self, name: &str) -> bool {
        if name.is_empty() {
            return false;
        }

        match self {
            CaseConvention::LowerCase => name
                .chars()
                .all(|c| c.is_lowercase() || c.is_numeric() || c == '_'),
            CaseConvention::UpperCase => name
                .chars()
                .all(|c| c.is_uppercase() || c.is_numeric() || c == '_'),
            CaseConvention::SnakeCase => {
                // Must be lowercase with underscores, no consecutive underscores
                if name.starts_with('_') || name.ends_with('_') || name.contains("__") {
                    return false;
                }
                name.chars()
                    .all(|c| c.is_lowercase() || c.is_numeric() || c == '_')
            }
            CaseConvention::CamelCase => {
                // Must start with lowercase, can have uppercase in middle
                if !name
                    .chars()
                    .next()
                    .map(|c| c.is_lowercase())
                    .unwrap_or(false)
                {
                    return false;
                }
                // No underscores, no consecutive uppercase
                let mut prev_upper = false;
                for c in name.chars() {
                    if c == '_' || c == '-' {
                        return false;
                    }
                    if c.is_uppercase() {
                        if prev_upper {
                            return false;
                        }
                        prev_upper = true;
                    } else {
                        prev_upper = false;
                    }
                }
                true
            }
            CaseConvention::PascalCase => {
                // Must start with uppercase, rest follows camelCase rules
                if !name
                    .chars()
                    .next()
                    .map(|c| c.is_uppercase())
                    .unwrap_or(false)
                {
                    return false;
                }
                // No underscores, no consecutive uppercase
                let mut prev_upper = false;
                for c in name.chars() {
                    if c == '_' || c == '-' {
                        return false;
                    }
                    if c.is_uppercase() {
                        if prev_upper {
                            return false;
                        }
                        prev_upper = true;
                    } else {
                        prev_upper = false;
                    }
                }
                true
            }
            CaseConvention::KebabCase => {
                // Must be lowercase with hyphens, no consecutive hyphens
                if name.starts_with('-') || name.ends_with('-') || name.contains("--") {
                    return false;
                }
                name.chars()
                    .all(|c| c.is_lowercase() || c.is_numeric() || c == '-')
            }
            CaseConvention::ScreamingSnakeCase => {
                // Must be uppercase with underscores, no consecutive underscores
                if name.starts_with('_') || name.ends_with('_') || name.contains("__") {
                    return false;
                }
                name.chars()
                    .all(|c| c.is_uppercase() || c.is_numeric() || c == '_')
            }
            CaseConvention::DotCase => {
                // Must be lowercase with dots, no consecutive dots
                if name.starts_with('.') || name.ends_with('.') || name.contains("..") {
                    return false;
                }
                name.chars()
                    .all(|c| c.is_lowercase() || c.is_numeric() || c == '.')
            }
            CaseConvention::FlatCase => {
                // Must be all lowercase letters and numbers, no separators
                if name.is_empty() {
                    return false;
                }
                name.chars().all(|c| c.is_lowercase() || c.is_numeric())
            }
            CaseConvention::ScreamingFlatCase => {
                // Must be all uppercase letters and numbers, no separators
                if name.is_empty() {
                    return false;
                }
                name.chars().all(|c| c.is_uppercase() || c.is_numeric())
            }
            CaseConvention::CobolCase => {
                // Must be uppercase with hyphens, no consecutive hyphens
                if name.starts_with('-') || name.ends_with('-') || name.contains("--") {
                    return false;
                }
                name.chars()
                    .all(|c| c.is_uppercase() || c.is_numeric() || c == '-')
            }
            CaseConvention::TrainCase => {
                // Must start with uppercase, then alternating lowercase and uppercase with hyphens
                // Pattern: Word-Word-Word (each word starts with uppercase, rest lowercase)
                if name.starts_with('-') || name.ends_with('-') || name.contains("--") {
                    return false;
                }
                // Check first character is uppercase
                if !name
                    .chars()
                    .next()
                    .map(|c| c.is_uppercase())
                    .unwrap_or(false)
                {
                    return false;
                }
                // Check pattern: uppercase followed by lowercase, then hyphen, repeat
                let parts: Vec<&str> = name.split('-').collect();
                for part in parts {
                    if part.is_empty() {
                        return false;
                    }
                    // Check if part is all numeric (allowed in Train-Case)
                    if part.chars().all(|c| c.is_numeric()) {
                        continue;
                    }
                    // Each part must start with uppercase
                    if !part
                        .chars()
                        .next()
                        .map(|c| c.is_uppercase())
                        .unwrap_or(false)
                    {
                        return false;
                    }
                    // Rest must be lowercase or numeric
                    for c in part.chars().skip(1) {
                        if !c.is_lowercase() && !c.is_numeric() {
                            return false;
                        }
                    }
                }
                true
            }
        }
    }

    /// Get an example for this convention
    pub fn example(self) -> &'static str {
        match self {
            CaseConvention::LowerCase => "filename",
            CaseConvention::UpperCase => "FILENAME",
            CaseConvention::SnakeCase => "file_name",
            CaseConvention::CamelCase => "fileName",
            CaseConvention::PascalCase => "FileName",
            CaseConvention::KebabCase => "file-name",
            CaseConvention::ScreamingSnakeCase => "FILE_NAME",
            CaseConvention::DotCase => "file.name",
            CaseConvention::FlatCase => "filename",
            CaseConvention::ScreamingFlatCase => "FILENAME",
            CaseConvention::CobolCase => "FILE-NAME",
            CaseConvention::TrainCase => "File-Name",
        }
    }

    /// Get a description of what's valid
    pub fn description(self) -> &'static str {
        match self {
            CaseConvention::LowerCase => "all lowercase letters and numbers",
            CaseConvention::UpperCase => "all uppercase letters and numbers",
            CaseConvention::SnakeCase => "lowercase with underscores between words",
            CaseConvention::CamelCase => "starts with lowercase, capitalizes word boundaries",
            CaseConvention::PascalCase => "starts with uppercase, capitalizes word boundaries",
            CaseConvention::KebabCase => "lowercase with hyphens between words",
            CaseConvention::ScreamingSnakeCase => "uppercase with underscores between words",
            CaseConvention::DotCase => "lowercase with dots between words",
            CaseConvention::FlatCase => "all lowercase letters and numbers, no separators",
            CaseConvention::ScreamingFlatCase => "all uppercase letters and numbers, no separators",
            CaseConvention::CobolCase => "uppercase with hyphens between words",
            CaseConvention::TrainCase => "title case words separated by hyphens",
        }
    }
}

impl std::fmt::Display for CaseConvention {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}
