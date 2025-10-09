#[derive(thiserror::Error, Debug, PartialEq)]
pub enum ConfigError {
    #[error("Missing required configuration name: '{0}'")]
    MissingName(String),
    #[error("Failed to parse name '{name}': {message}")]
    InvalidValue { name: String, message: String },
    #[error("Validation failed for name '{name}': {message}")]
    ValidationFailed { name: String, message: String },
}
