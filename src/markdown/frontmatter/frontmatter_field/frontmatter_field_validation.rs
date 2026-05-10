//! FieldValidator runtime validation methods.

use regex::Regex;

use super::{FieldType, FieldValidator};
use crate::markdown::error::MarkdownValidationError;

impl FieldValidator {
    /// Validate a value against this validator
    pub fn validate(
        &self,
        field_name: &str,
        value: &serde_yaml::Value,
        path: &std::path::Path,
    ) -> Result<(), MarkdownValidationError> {
        // First, check the type
        match self.field_type {
            FieldType::String => self.validate_string(field_name, value, path)?,
            FieldType::Integer => self.validate_integer(field_name, value, path)?,
            FieldType::Float => self.validate_float(field_name, value, path)?,
            FieldType::Boolean => self.validate_boolean(field_name, value, path)?,
            FieldType::Array => self.validate_array(field_name, value, path)?,
            FieldType::Object => self.validate_object(field_name, value, path)?,
            FieldType::Date => self.validate_date(field_name, value, path)?,
            FieldType::DateTime => self.validate_datetime(field_name, value, path)?,
            FieldType::Email => self.validate_email(field_name, value, path)?,
            FieldType::Url => self.validate_url(field_name, value, path)?,
        }

        // Check allowed values
        if let Some(ref allowed) = self.allowed_values {
            if !allowed.contains(value) {
                return Err(MarkdownValidationError::new(
                    "field_validation",
                    path,
                    self.message.clone().unwrap_or_else(|| {
                        format!(
                            "Field '{}' has invalid value. Allowed: {:?}",
                            field_name, allowed
                        )
                    }),
                ));
            }
        }

        Ok(())
    }

    fn validate_string(
        &self,
        field_name: &str,
        value: &serde_yaml::Value,
        path: &std::path::Path,
    ) -> Result<(), MarkdownValidationError> {
        let s = match value {
            serde_yaml::Value::String(s) => s,
            _ => {
                return Err(MarkdownValidationError::new(
                    "field_type",
                    path,
                    format!("Field '{}' must be a string", field_name),
                ));
            }
        };

        // Check pattern
        if let Some(ref pattern) = self.pattern {
            let regex = Regex::new(pattern).map_err(|_| {
                MarkdownValidationError::new(
                    "invalid_pattern",
                    path,
                    format!("Invalid regex pattern for field '{}'", field_name),
                )
            })?;
            if !regex.is_match(s) {
                return Err(MarkdownValidationError::new(
                    "field_pattern",
                    path,
                    self.message.clone().unwrap_or_else(|| {
                        format!("Field '{}' does not match pattern: {}", field_name, pattern)
                    }),
                ));
            }
        }

        // Check length constraints
        if let Some(ref min) = self.min {
            if let Some(min_len) = min.as_u64() {
                if s.len() < min_len as usize {
                    return Err(MarkdownValidationError::new(
                        "field_min_length",
                        path,
                        format!(
                            "Field '{}' must be at least {} characters",
                            field_name, min_len
                        ),
                    ));
                }
            }
        }

        if let Some(ref max) = self.max {
            if let Some(max_len) = max.as_u64() {
                if s.len() > max_len as usize {
                    return Err(MarkdownValidationError::new(
                        "field_max_length",
                        path,
                        format!(
                            "Field '{}' must be at most {} characters",
                            field_name, max_len
                        ),
                    ));
                }
            }
        }

        Ok(())
    }

    fn validate_integer(
        &self,
        field_name: &str,
        value: &serde_yaml::Value,
        path: &std::path::Path,
    ) -> Result<(), MarkdownValidationError> {
        let n = match value {
            serde_yaml::Value::Number(n) => n.as_i64().ok_or_else(|| {
                MarkdownValidationError::new(
                    "field_type",
                    path,
                    format!("Field '{}' must be an integer", field_name),
                )
            })?,
            _ => {
                return Err(MarkdownValidationError::new(
                    "field_type",
                    path,
                    format!("Field '{}' must be an integer", field_name),
                ));
            }
        };

        // Check min/max
        if let Some(ref min) = self.min {
            if let Some(min_val) = min.as_i64() {
                if n < min_val {
                    return Err(MarkdownValidationError::new(
                        "field_min_value",
                        path,
                        format!("Field '{}' must be >= {}", field_name, min_val),
                    ));
                }
            }
        }

        if let Some(ref max) = self.max {
            if let Some(max_val) = max.as_i64() {
                if n > max_val {
                    return Err(MarkdownValidationError::new(
                        "field_max_value",
                        path,
                        format!("Field '{}' must be <= {}", field_name, max_val),
                    ));
                }
            }
        }

        Ok(())
    }

    fn validate_float(
        &self,
        field_name: &str,
        value: &serde_yaml::Value,
        path: &std::path::Path,
    ) -> Result<(), MarkdownValidationError> {
        let n = match value {
            serde_yaml::Value::Number(n) => n.as_f64().ok_or_else(|| {
                MarkdownValidationError::new(
                    "field_type",
                    path,
                    format!("Field '{}' must be a number", field_name),
                )
            })?,
            _ => {
                return Err(MarkdownValidationError::new(
                    "field_type",
                    path,
                    format!("Field '{}' must be a number", field_name),
                ));
            }
        };

        // Check min/max
        if let Some(ref min) = self.min {
            if let Some(min_val) = min.as_f64() {
                if n < min_val {
                    return Err(MarkdownValidationError::new(
                        "field_min_value",
                        path,
                        format!("Field '{}' must be >= {}", field_name, min_val),
                    ));
                }
            }
        }

        if let Some(ref max) = self.max {
            if let Some(max_val) = max.as_f64() {
                if n > max_val {
                    return Err(MarkdownValidationError::new(
                        "field_max_value",
                        path,
                        format!("Field '{}' must be <= {}", field_name, max_val),
                    ));
                }
            }
        }

        Ok(())
    }

    fn validate_boolean(
        &self,
        field_name: &str,
        value: &serde_yaml::Value,
        path: &std::path::Path,
    ) -> Result<(), MarkdownValidationError> {
        match value {
            serde_yaml::Value::Bool(_) => Ok(()),
            _ => Err(MarkdownValidationError::new(
                "field_type",
                path,
                format!("Field '{}' must be a boolean", field_name),
            )),
        }
    }

    fn validate_array(
        &self,
        field_name: &str,
        value: &serde_yaml::Value,
        path: &std::path::Path,
    ) -> Result<(), MarkdownValidationError> {
        let arr = match value {
            serde_yaml::Value::Sequence(arr) => arr,
            _ => {
                return Err(MarkdownValidationError::new(
                    "field_type",
                    path,
                    format!("Field '{}' must be an array", field_name),
                ));
            }
        };

        // Check length constraints
        if let Some(ref min) = self.min {
            if let Some(min_len) = min.as_u64() {
                if arr.len() < min_len as usize {
                    return Err(MarkdownValidationError::new(
                        "field_min_length",
                        path,
                        format!(
                            "Field '{}' must have at least {} items",
                            field_name, min_len
                        ),
                    ));
                }
            }
        }

        if let Some(ref max) = self.max {
            if let Some(max_len) = max.as_u64() {
                if arr.len() > max_len as usize {
                    return Err(MarkdownValidationError::new(
                        "field_max_length",
                        path,
                        format!("Field '{}' must have at most {} items", field_name, max_len),
                    ));
                }
            }
        }

        Ok(())
    }

    fn validate_object(
        &self,
        field_name: &str,
        value: &serde_yaml::Value,
        path: &std::path::Path,
    ) -> Result<(), MarkdownValidationError> {
        match value {
            serde_yaml::Value::Mapping(_) => Ok(()),
            _ => Err(MarkdownValidationError::new(
                "field_type",
                path,
                format!("Field '{}' must be an object", field_name),
            )),
        }
    }

    fn validate_date(
        &self,
        field_name: &str,
        value: &serde_yaml::Value,
        path: &std::path::Path,
    ) -> Result<(), MarkdownValidationError> {
        let s = match value {
            serde_yaml::Value::String(s) => s,
            _ => {
                return Err(MarkdownValidationError::new(
                    "field_type",
                    path,
                    format!("Field '{}' must be a date string (YYYY-MM-DD)", field_name),
                ));
            }
        };

        // Validate date format YYYY-MM-DD
        let date_regex = Regex::new(r"^\d{4}-\d{2}-\d{2}$").unwrap();
        if !date_regex.is_match(s) {
            return Err(MarkdownValidationError::new(
                "field_format",
                path,
                format!("Field '{}' must be in format YYYY-MM-DD", field_name),
            ));
        }

        Ok(())
    }

    fn validate_datetime(
        &self,
        field_name: &str,
        value: &serde_yaml::Value,
        path: &std::path::Path,
    ) -> Result<(), MarkdownValidationError> {
        let s = match value {
            serde_yaml::Value::String(s) => s,
            _ => {
                return Err(MarkdownValidationError::new(
                    "field_type",
                    path,
                    format!(
                        "Field '{}' must be a datetime string (ISO 8601)",
                        field_name
                    ),
                ));
            }
        };

        // Validate ISO 8601 datetime format
        let datetime_regex =
            Regex::new(r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(\.\d+)?(Z|[+-]\d{2}:\d{2})?$")
                .unwrap();
        if !datetime_regex.is_match(s) {
            return Err(MarkdownValidationError::new(
                "field_format",
                path,
                format!("Field '{}' must be in ISO 8601 datetime format", field_name),
            ));
        }

        Ok(())
    }

    fn validate_email(
        &self,
        field_name: &str,
        value: &serde_yaml::Value,
        path: &std::path::Path,
    ) -> Result<(), MarkdownValidationError> {
        let s = match value {
            serde_yaml::Value::String(s) => s,
            _ => {
                return Err(MarkdownValidationError::new(
                    "field_type",
                    path,
                    format!("Field '{}' must be an email string", field_name),
                ));
            }
        };

        // Simple email validation regex
        let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
        if !email_regex.is_match(s) {
            return Err(MarkdownValidationError::new(
                "field_format",
                path,
                format!("Field '{}' must be a valid email address", field_name),
            ));
        }

        Ok(())
    }

    fn validate_url(
        &self,
        field_name: &str,
        value: &serde_yaml::Value,
        path: &std::path::Path,
    ) -> Result<(), MarkdownValidationError> {
        let s = match value {
            serde_yaml::Value::String(s) => s,
            _ => {
                return Err(MarkdownValidationError::new(
                    "field_type",
                    path,
                    format!("Field '{}' must be a URL string", field_name),
                ));
            }
        };

        // Simple URL validation
        let url_regex = Regex::new(r"^https?://[^\s/$.?#].[^\s]*$").unwrap();
        if !url_regex.is_match(s) {
            return Err(MarkdownValidationError::new(
                "field_format",
                path,
                format!("Field '{}' must be a valid URL (http/https)", field_name),
            ));
        }

        Ok(())
    }
}
