use crate::{ConfigError, Validator};
use std::fmt::{self, Display};

/// A stateful validator for numeric ranges.
#[derive(Clone, Debug)]
pub struct Range {
    min: Option<f64>,
    max: Option<f64>,
}

impl Range {
    // This private constructor is idiomatic Rust for enforcing creation via factories.
    fn new(min: Option<f64>, max: Option<f64>) -> Self {
        Self { min, max }
    }

    /// Factory for a range with a lower bound. Returns a trait object.
    pub fn at_least(min: impl Into<f64>) -> Box<dyn Validator> {
        Box::new(Self::new(Some(min.into()), None))
    }

    /// Factory for a range with an upper and lower bound. Returns a trait object.
    pub fn between(min: impl Into<f64>, max: impl Into<f64>) -> Box<dyn Validator> {
        Box::new(Self::new(Some(min.into()), Some(max.into())))
    }
}

impl Validator for Range {
    fn validate(&self, name: &str, value: &str) -> Result<(), ConfigError> {
        let n: f64 = value
            .trim()
            .parse()
            .map_err(|_| ConfigError::InvalidValue {
                name: name.to_string(),
                message: "Value is not a valid number".to_string(),
            })?;

        if let Some(min) = self.min
            && n < min
        {
            return Err(ConfigError::ValidationFailed {
                name: name.to_string(),
                message: format!("Value {} must be at least {}", n, min),
            });
        }

        if let Some(max) = self.max
            && n > max
        {
            return Err(ConfigError::ValidationFailed {
                name: name.to_string(),
                message: format!("Value {} must be no more than {}", n, max),
            });
        }

        Ok(())
    }

    fn box_clone(&self) -> Box<dyn Validator> {
        Box::new(self.clone())
    }
}

impl Display for Range {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self.min, self.max) {
            (None, None) => write!(f, "[...]"),
            (None, Some(max)) => write!(f, "[..., {}]", max),
            (Some(min), None) => write!(f, "[{}, ...]", min),
            (Some(min), Some(max)) => write!(f, "[{}, ..., {}]", min, max),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_range_debug_output() {
        // Test the derived Debug impl on the struct itself
        let range_struct = Range::new(Some(10.0), Some(100.0));
        let struct_debug = format!("{:?}", range_struct);
        assert_eq!(struct_debug, "Range { min: Some(10.0), max: Some(100.0) }");

        // Test the Debug impl on the Box<dyn Validator> which should use the Display impl
        let at_least_validator = Range::at_least(0);
        let at_least_debug = format!("{:?}", at_least_validator);
        assert_eq!(at_least_debug, "Validator([0, ...])");

        let between_validator = Range::between(10, 20);
        let between_debug = format!("{:?}", between_validator);
        assert_eq!(between_debug, "Validator([10, ..., 20])");
    }
}
