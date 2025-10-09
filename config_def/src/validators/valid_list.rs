use crate::{ConfigError, ValidString, Validator};
use std::collections::HashSet;
use std::fmt;
use std::fmt::Display;

/// A stateful validator for comma-separated lists.
///
/// It can check for duplicate values, enforce a specific set of allowed values,
/// and control whether an empty list is permitted.
#[derive(Clone, Debug)]
pub struct ValidList {
    valid_string: ValidString,
    is_empty_allowed: bool,
}

impl ValidList {
    // Private constructor.
    fn new(valid_strings: Vec<String>, is_empty_allowed: bool) -> Self {
        Self {
            valid_string: ValidString::new(valid_strings),
            is_empty_allowed,
        }
    }

    /// Factory for creating a validator that allows any non-duplicate values.
    pub fn any_non_duplicate_values(is_empty_allowed: bool) -> Box<dyn Validator> {
        Box::new(Self::new(Vec::new(), is_empty_allowed))
    }

    /// Creates a validator that ensures all values are in the given set.
    /// Allows empty lists by default.
    pub fn in_list(valid_strings: &[&'static str]) -> Box<dyn Validator> {
        Box::new(Self::new(
            valid_strings.iter().map(|s| s.to_string()).collect(),
            true, // is_empty_allowed
        ))
    }

    /// A configurable factory that creates a validator for a specific set of values
    /// and allows specifying whether an empty list is valid.
    ///
    /// Panics if an empty list is disallowed but no valid strings are provided.
    pub fn in_list_allow_empty(
        is_empty_allowed: bool,
        valid_strings: &[&'static str],
    ) -> Box<dyn Validator> {
        if !is_empty_allowed && valid_strings.is_empty() {
            panic!("At least one valid string must be provided when empty values are not allowed");
        }
        Box::new(Self::new(
            valid_strings.iter().map(|s| s.to_string()).collect(),
            is_empty_allowed,
        ))
    }
}

impl Validator for ValidList {
    fn validate(&self, name: &str, value: &str) -> Result<(), ConfigError> {
        // Step 1: Parse the raw string into a vector of strings.
        // This handles cases like " a, , b " and results in `vec!["a", "b"]`.
        let values_str: Vec<&str> = value.trim().split(',').map(|s| s.trim()).collect();

        // If the input was empty or just whitespace/commas, `split` might produce `[""]`.
        // We want to treat this as a truly empty list for the `is_empty_allowed` check.
        let values: Vec<&str> = if values_str.len() == 1 && values_str[0].is_empty() {
            Vec::new()
        } else {
            values_str
        };

        // Step 2: Check if the list is empty.
        if !self.is_empty_allowed && values.is_empty() {
            let valid_values_str = if self.valid_string.valid_strings().is_empty() {
                "any non-empty value".to_string()
            } else {
                self.to_string()
            };
            return Err(ConfigError::ValidationFailed {
                name: name.to_string(),
                message: format!(
                    "Configuration '{}' must not be empty. Valid values include: {}",
                    name, valid_values_str
                ),
            });
        }

        // Step 3: Check for duplicates.
        let unique_values: HashSet<_> = values.iter().collect();
        if unique_values.len() != values.len() {
            return Err(ConfigError::ValidationFailed {
                name: name.to_string(),
                message: format!("Configuration '{}' values must not be duplicated.", name),
            });
        }

        // Step 4: Validate individual values against the allowed set (if any).
        for &val in &values {
            if val.is_empty() {
                return Err(ConfigError::ValidationFailed {
                    name: name.to_string(),
                    message: format!("Configuration '{}' values must not be empty.", name),
                });
            }
            if !self.valid_string.valid_strings().is_empty()
                && !self.valid_string.valid_strings().contains(&val.to_string())
            {
                return Err(ConfigError::ValidationFailed {
                    name: name.to_string(),
                    message: format!(
                        "Invalid value '{}' for configuration '{}': String must be one of: {}",
                        val,
                        name,
                        self.valid_string.valid_strings().join(", ")
                    ),
                });
            }
        }

        Ok(())
    }

    fn box_clone(&self) -> Box<dyn Validator> {
        Box::new(self.clone())
    }
}

impl Display for ValidList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} (empty config {})",
            self.valid_string,
            if self.is_empty_allowed {
                "empty config allowed"
            } else {
                "empty not allowed"
            }
        )
    }
}
