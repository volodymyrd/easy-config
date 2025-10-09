use crate::ConfigError;
use std::fmt::Display;

pub(crate) mod range;
pub(crate) mod valid_list;
pub(crate) mod valid_string;

/// A trait for any stateful validation logic.
/// It must be `Send + Sync` to be stored in a static `Lazy` cell.
/// The `box_clone` method is a standard pattern for making trait objects cloneable.
pub trait Validator: Display + Send + Sync {
    /// The core validation method. It operates on the raw string value.
    fn validate(&self, name: &str, value: &str) -> Result<(), ConfigError>;

    fn box_clone(&self) -> Box<dyn Validator>;
}

/// Implement `Clone` for any `Box<dyn Validator>`.
impl Clone for Box<dyn Validator> {
    fn clone(&self) -> Self {
        self.box_clone()
    }
}
