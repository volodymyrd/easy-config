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
