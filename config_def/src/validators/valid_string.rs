use crate::{ConfigError, Validator};
use std::fmt::{self, Display};

/// A stateful validator that checks if a string is in a predefined set.
#[derive(Clone, Debug)]
pub struct ValidString {
    valid_strings: Vec<String>,
}

impl ValidString {
    // Private constructor.
    fn new(valid_strings: Vec<String>) -> Self {
        Self { valid_strings }
    }

    /// Factory for creating a `ValidString` validator.
    ///
    /// It takes a slice of string slices and returns a trait object.
    /// Example: `ValidString::in_list(&["a", "b", "c"])`
    pub fn in_list(valid_strings: &[&'static str]) -> Box<dyn Validator> {
        Box::new(Self::new(
            valid_strings.iter().map(|s| s.to_string()).collect(),
        ))
    }
}

impl Validator for ValidString {
    fn validate(&self, name: &str, value: &str) -> Result<(), ConfigError> {
        let s = value.trim();
        if !self.valid_strings.contains(&s.to_string()) {
            Err(ConfigError::ValidationFailed {
                name: name.to_string(),
                message: format!("String must be one of: {}", self.valid_strings.join(", ")),
            })
        } else {
            Ok(())
        }
    }

    fn box_clone(&self) -> Box<dyn Validator> {
        Box::new(self.clone())
    }
}

impl Display for ValidString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}]", self.valid_strings.join(", "))
    }
}
