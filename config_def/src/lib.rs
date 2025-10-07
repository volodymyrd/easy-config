pub use easy_config_macros::EasyConfig;
pub use errors::ConfigError;
pub use validators::{Validator, range::Range, valid_string::ValidString};

use indexmap::IndexMap;
use std::collections::{HashMap, HashSet, LinkedList};
use std::fmt::Display;
use std::str::FromStr;
pub use types::password::Password;

mod errors;
mod types;
mod validators;

pub trait FromConfigDef: Sized {
    fn from_props(props: &HashMap<String, String>) -> Result<Self, ConfigError>;
}

pub trait ConfigValue: Sized {
    fn parse(key: &str, value_str: &str) -> Result<Self, ConfigError>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Importance {
    HIGH,
    MEDIUM,
    LOW,
}

#[derive(Clone)]
pub struct ConfigKey {
    pub name: &'static str,
    pub documentation: Option<&'static str>,
    pub default_value: Option<&'static str>,
    pub validator: Option<Box<dyn Validator>>,
    pub importance: Option<Importance>,
    pub group: Option<&'static str>,
    // pub order_in_group: Option<usize>,
    // pub width: Width,
    // pub display_name: Option<&'static str>,
    // pub dependents: Vec<&'static str>,
    // pub recommender: Recommender,
    // pub internal_config: bool,
    // pub alternative_string: Option<&'static str>,
}

#[derive(Default)]
pub struct ConfigDef {
    config_keys: IndexMap<&'static str, ConfigKey>,
    _groups: LinkedList<String>,
    _configs_with_no_parent: HashSet<String>,
}

#[derive(Default)]
pub struct ConfigDefBuilder {
    config_keys: IndexMap<&'static str, ConfigKey>,
    groups: LinkedList<String>,
}

impl ConfigDef {
    pub fn builder() -> ConfigDefBuilder {
        ConfigDefBuilder::default()
    }

    pub fn find_key(&self, name: &str) -> Option<&ConfigKey> {
        self.config_keys.get(name)
    }
}

impl ConfigDefBuilder {
    pub fn define(&mut self, key: ConfigKey) -> &mut Self {
        if self.config_keys.contains_key(key.name) {
            panic!("Configuration key {} is defined twice", key.name);
        }

        if let Some(group_name) = key.group.as_ref() {
            let group_string = group_name.to_string();
            if !self.groups.contains(&group_string) {
                self.groups.push_back(group_string);
            }
        }

        self.config_keys.insert(key.name, key);
        self
    }

    pub fn build(self) -> ConfigDef {
        ConfigDef {
            config_keys: self.config_keys,
            _groups: self.groups,
            _configs_with_no_parent: HashSet::new(),
        }
    }
}

fn parse_config_value<T>(key: &str, s: &str) -> Result<T, ConfigError>
where
    T: ConfigValue + Copy + FromStr + 'static, // The type must be parsable from a string.
    <T as FromStr>::Err: Display,              // The error it produces must be printable
{
    s.trim()
        .to_lowercase()
        .parse()
        .map_err(|e: <T as FromStr>::Err| ConfigError::InvalidValue {
            name: key.to_string(),
            message: e.to_string(),
        })
}

impl ConfigValue for bool {
    fn parse(key: &str, s: &str) -> Result<Self, ConfigError> {
        parse_config_value(key, s)
    }
}

impl ConfigValue for i32 {
    fn parse(key: &str, s: &str) -> Result<Self, ConfigError> {
        parse_config_value(key, s)
    }
}

impl ConfigValue for i64 {
    fn parse(key: &str, s: &str) -> Result<Self, ConfigError> {
        parse_config_value(key, s)
    }
}

impl ConfigValue for f32 {
    fn parse(key: &str, s: &str) -> Result<Self, ConfigError> {
        parse_config_value(key, s)
    }
}

impl ConfigValue for f64 {
    fn parse(key: &str, s: &str) -> Result<Self, ConfigError> {
        parse_config_value(key, s)
    }
}

impl ConfigValue for String {
    fn parse(_key: &str, s: &str) -> Result<Self, ConfigError> {
        Ok(s.trim().to_string())
    }
}

impl ConfigValue for Vec<String> {
    fn parse(_key: &str, s: &str) -> Result<Self, ConfigError> {
        Ok(s.trim()
            .split(',')
            .map(|item| item.trim().to_string())
            .collect())
    }
}

impl ConfigValue for Password {
    fn parse(_key: &str, s: &str) -> Result<Self, ConfigError> {
        Ok(Password::new(s.trim().to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::fmt::Debug;

    #[test]
    fn test_basic_types() {
        #[derive(Debug, PartialEq, EasyConfig)]
        struct TestConfig {
            #[attr(default = "5", validator=Range::between(0, 14), importance = Importance::HIGH,
            documentation = "docs")]
            a: i32,
            #[attr(importance = Importance::HIGH, documentation = "docs")]
            b: i64,
            #[attr(default = "hello", importance = Importance::HIGH, documentation = "docs")]
            c: String,
            #[attr(importance = Importance::HIGH, documentation = "docs")]
            d: Vec<String>,
            #[attr(importance = Importance::HIGH, documentation = "docs")]
            e: f64,
            #[attr(importance = Importance::HIGH, documentation = "docs")]
            f: String,
            #[attr(importance = Importance::HIGH, documentation = "docs")]
            g: bool,
            #[attr(importance = Importance::HIGH, documentation = "docs")]
            h: bool,
            #[attr(importance = Importance::HIGH, documentation = "docs")]
            i: bool,
            #[attr(importance = Importance::HIGH, documentation = "docs")]
            j: Password,
        }

        // Arrange: Set up the raw string properties.
        let mut props = HashMap::new();
        props.insert("a".to_string(), "1   ".to_string());
        props.insert("b".to_string(), "2".to_string());
        // "c" is omitted to test the default value.
        props.insert("d".to_string(), " a , b, c".to_string());
        props.insert("e".to_string(), "42.5".to_string());
        props.insert("f".to_string(), "java.lang.String".to_string());
        props.insert("g".to_string(), "true".to_string());
        props.insert("h".to_string(), "FalSE".to_string());
        props.insert("i".to_string(), "TRUE".to_string());
        props.insert("j".to_string(), "password".to_string());

        // Act: Parse the properties into the strongly typed struct.
        let config = TestConfig::from_props(&props).unwrap();

        // Assert: Check the final parsed values.
        assert_eq!(config.a, 1);
        assert_eq!(config.b, 2);
        assert_eq!(config.c, "hello"); // Correctly uses the default
        assert_eq!(config.d, vec!["a", "b", "c"]);
        assert_eq!(config.e, 42.5);
        assert_eq!(config.f, "java.lang.String");
        assert_eq!(config.g, true);
        assert_eq!(config.h, false);
        assert_eq!(config.i, true);
        assert_eq!(config.j, Password::new("password".to_string()));
        assert_eq!(config.j.to_string(), "[hidden]");
    }

    #[test]
    fn test_invalid_default() {
        #[derive(Debug, EasyConfig)]
        struct TestConfig {
            #[attr(default = "hello")] // "hello" is not a valid i32
            _a: i32,
        }

        let result = TestConfig::from_props(&HashMap::new());

        match result {
            Err(ConfigError::InvalidValue { name, message }) => {
                assert_eq!(name, "_a");
                // The exact error message from `ParseIntError` can be a bit volatile,
                // so checking `contains` is more robust than a direct equality check.
                assert!(message.contains("invalid digit found in string"));
            }
            _ => {
                // If we get `Ok` or a different `Err` variant, fail the test.
                panic!("Expected InvalidValue error, but got {:?}", result);
            }
        }
    }

    #[test]
    fn test_null_default() {
        #[derive(EasyConfig, Debug, PartialEq)]
        struct TestConfig {
            // This field is optional and has no default.
            #[attr(documentation = "docs")]
            a: Option<i32>,
        }

        let config = TestConfig::from_props(&HashMap::new()).unwrap();

        assert_eq!(config.a, None);
    }

    #[test]
    fn test_missing_required() {
        #[derive(EasyConfig)]
        struct TestConfig {
            // This field is required (not an Option, no default).
            #[attr(importance = Importance::HIGH, documentation = "docs")]
            _a: i32,
        }

        let config = TestConfig::from_props(&HashMap::new());

        assert!(matches!(config, Err(ConfigError::MissingName(s)) if s == "_a"));
    }

    #[test]
    fn test_parsing_empty_default_value_for_string_field_should_succeed() {
        #[derive(EasyConfig)]
        struct TestConfig {
            // This field is required empty by default.
            #[attr(default="", importance = Importance::HIGH, documentation = "docs")]
            _a: String,
        }

        let _ = TestConfig::from_props(&HashMap::new()).expect("parsing should succeed");
    }

    macro_rules! test_bad_inputs {
        // The macro takes a test name, the type to test, and a slice of bad values.
        ($test_name:ident, $type:ty, $bad_values:expr) => {
            #[test]
            fn $test_name() {
                #[derive(EasyConfig, Debug)]
                struct TestConfig { _name: $type }

                for &value in $bad_values {
                    let mut props = HashMap::new();
                    props.insert("_name".to_string(), value.to_string());

                    let result = TestConfig::from_props(&props);

                    assert!(
                        matches!(&result, Err(ConfigError::InvalidValue { name, .. }) if name == "_name"),
                        "Expected InvalidValue error for type '{}' with input '{}', but got {:?}",
                        stringify!($type),
                        value,
                        result
                    );
                }
            }
        };
    }

    test_bad_inputs!(
        test_bad_inputs_for_int,
        i32,
        &["hello", "42.5", "9223372036854775807"]
    );

    test_bad_inputs!(
        test_bad_inputs_for_long,
        i64,
        &["hello", "42.5", "922337203685477580700"]
    );

    test_bad_inputs!(test_bad_inputs_for_double, f64, &["hello", "not-a-number"]);

    test_bad_inputs!(
        test_bad_inputs_for_boolean,
        bool,
        &["hello", "truee", "fals", "0", "1"]
    );

    #[test]
    fn test_invalid_default_range() {
        #[derive(Debug, EasyConfig)]
        struct TestConfig {
            #[attr(default="-1", validator=Range::between(0, 10),
            importance = Importance::HIGH, documentation = "docs")]
            _a: i32,
        }

        let config = TestConfig::from_props(&HashMap::new());

        assert!(
            matches!(&config, Err(ConfigError::ValidationFailed{name, message})
            if name == "_a" && message.contains("Value -1 must be at least 0")
            ),
            "Expected ValidationFailed error, but got {:?}",
            &config
        );

        println!("Received expected error: {:?}", &config.unwrap_err());
    }

    #[test]
    fn test_invalid_default_string() {
        #[derive(Debug, EasyConfig)]
        struct TestConfig {
            #[attr(default="bad", validator=ValidString::in_list(&["valid", "values"]),
            importance = Importance::HIGH, documentation = "docs")]
            _a: String,
        }

        let config = TestConfig::from_props(&HashMap::new());

        assert!(
            matches!(
                &config,
                Err(ConfigError::ValidationFailed { name, message })
                    if name == "_a" && message.contains("must be one of: valid, values")
            ),
            "Expected ValidationFailed error, but got {:?}",
            &config
        );

        println!("Received expected error: {:?}", &config.unwrap_err());
    }

    // TODO: Add support for pluggable components
    //     @Test
    //     public void testNestedClass() {
    //         // getName(), not getSimpleName() or getCanonicalName(), is the version that should be able to locate the class
    //         Map<String, Object> props = Collections.singletonMap("name", NestedClass.class.getName());
    //         new ConfigDef().define("name", Type.CLASS, Importance.HIGH, "docs").parse(props);
    //     }
    //
}
