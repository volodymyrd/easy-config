#[macro_export]
/// Macro to reduce boilerplate for types implementing FromStr.
macro_rules! impl_config_value_for_fromstr {
    ($($t:ty),*) => {
        $(
            impl ConfigValue for $t {
                fn parse(key: &str, s: &str) -> Result<Self, ConfigError> {
                    s.trim()
                    .to_lowercase()
                    .parse()
                    .map_err(|e| ConfigError::InvalidValue {
                        name: key.to_string(),
                        message: format!("{}", e),
                    })
                }
                fn to_config_string(&self) -> String {
                    self.to_string()
                }
            }
        )*
    };
}
