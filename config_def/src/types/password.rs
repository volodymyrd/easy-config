/// A wrapper for passwords to hide them while logging a config.
use std::fmt;

const HIDDEN: &str = "[hidden]";

#[derive(Clone, Eq, PartialEq)]
pub struct Password(String);

impl Password {
    pub fn new(s: String) -> Self {
        Password(s)
    }

    /// Returns a real password string.
    pub fn password(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for Password {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Password({HIDDEN})")
    }
}
impl fmt::Display for Password {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{HIDDEN}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_creation_and_retrieval() {
        let password_str = "my_secret_password".to_string();
        let password = Password::new(password_str.clone());
        assert_eq!(password.password(), password_str);
    }

    #[test]
    fn test_password_debug_format() {
        let password = Password::new("my_secret_password".to_string());
        let debug_output = format!("{:?}", password);
        assert_eq!(debug_output, "Password([hidden])");
    }

    #[test]
    fn test_password_display_format() {
        let password = Password::new("my_secret_password".to_string());
        let display_output = format!("{}", password);
        assert_eq!(display_output, "[hidden]");
    }

    #[test]
    fn test_password_clone_and_equality() {
        let password_str = "my_secret_password".to_string();
        let password = Password::new(password_str.clone());
        let password_clone = password.clone();
        assert_eq!(password, password_clone);
    }
}
