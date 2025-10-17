use indexmap::IndexMap;
use prelude::*;
use std::collections::{HashMap, HashSet, LinkedList};
use std::fmt::Display;
use std::str::FromStr;
pub use types::password::Password;

pub mod prelude;

mod errors;
mod types;
mod validators;

pub trait FromConfigDef: Sized {
    fn from_props(props: &HashMap<String, String>) -> Result<Self, ConfigError>;
    // The contract for getting the schema.
    fn config_def() -> Result<&'static ConfigDef, ConfigError>;
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

#[derive(Debug, Clone)]
pub struct ConfigKey {
    pub name: &'static str,
    pub documentation: Option<String>,
    pub default_value: Option<String>,
    pub validator: Option<Box<dyn Validator>>,
    pub importance: Option<Importance>,
    pub group: Option<String>,
    // pub order_in_group: Option<usize>,
    // pub width: Width,
    // pub display_name: Option<&'static str>,
    // pub dependents: Vec<&'static str>,
    // pub recommender: Recommender,
    pub internal_config: bool,
    // pub alternative_string: Option<&'static str>,
}

#[derive(Default)]
pub struct ConfigDef {
    config_keys: IndexMap<&'static str, ConfigKey>,
    _groups: LinkedList<String>,
    _configs_with_no_parent: HashSet<String>,
}

impl ConfigDef {
    pub fn find_key(&self, name: &str) -> Option<&ConfigKey> {
        self.config_keys.get(name)
    }

    pub fn config_keys(&self) -> &IndexMap<&'static str, ConfigKey> {
        &self.config_keys
    }
}

impl TryFrom<Vec<ConfigKey>> for ConfigDef {
    type Error = ConfigError;

    /// Creates a `ConfigDef` from a vector of `ConfigKey`s, checking for duplicates.
    fn try_from(keys: Vec<ConfigKey>) -> Result<Self, Self::Error> {
        let mut config_keys = IndexMap::with_capacity(keys.len());
        let mut seen_groups = HashSet::new();

        for key in keys {
            if let Some(existing_key) = config_keys.insert(key.name, key) {
                return Err(ConfigError::ValidationFailed {
                    name: existing_key.name.to_string(),
                    message: format!(
                        "Configuration key '{}' is defined twice.",
                        existing_key.name
                    ),
                });
            }
        }

        let groups: LinkedList<String> = config_keys
            .values()
            .filter_map(|k| k.group.as_ref())
            .filter(|&g| seen_groups.insert(g))
            .map(String::from)
            .collect();

        Ok(ConfigDef {
            config_keys,
            _groups: groups,
            ..Default::default()
        })
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

impl ConfigValue for u8 {
    fn parse(key: &str, s: &str) -> Result<Self, ConfigError> {
        parse_config_value(key, s)
    }
}

impl ConfigValue for u16 {
    fn parse(key: &str, s: &str) -> Result<Self, ConfigError> {
        parse_config_value(key, s)
    }
}

impl ConfigValue for u32 {
    fn parse(key: &str, s: &str) -> Result<Self, ConfigError> {
        parse_config_value(key, s)
    }
}

impl ConfigValue for u64 {
    fn parse(key: &str, s: &str) -> Result<Self, ConfigError> {
        parse_config_value(key, s)
    }
}

impl ConfigValue for u128 {
    fn parse(key: &str, s: &str) -> Result<Self, ConfigError> {
        parse_config_value(key, s)
    }
}

impl ConfigValue for usize {
    fn parse(key: &str, s: &str) -> Result<Self, ConfigError> {
        parse_config_value(key, s)
    }
}

impl ConfigValue for i8 {
    fn parse(key: &str, s: &str) -> Result<Self, ConfigError> {
        parse_config_value(key, s)
    }
}

impl ConfigValue for i16 {
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

impl ConfigValue for i128 {
    fn parse(key: &str, s: &str) -> Result<Self, ConfigError> {
        parse_config_value(key, s)
    }
}

impl ConfigValue for isize {
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
        let s = s.trim();
        if s.is_empty() {
            return Ok(Vec::new());
        }
        Ok(s.split(',').map(|item| item.trim().to_string()).collect())
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
    use easy_config_macros::EasyConfig;
    use std::collections::HashMap;
    use std::fmt::Debug;

    const H: &str = "h";
    const DOC: &str = "Docs for 'a'. ";

    #[test]
    fn test_basic_types() {
        #[derive(Debug, PartialEq, EasyConfig)]
        struct TestConfig {
            #[attr(default = 5, validator = Range::between(0, 14), importance = Importance::HIGH,
            documentation = format!("{DOC} Must be between 0 and 14."))]
            a: i32,
            #[attr(importance = Importance::HIGH, documentation = "docs".to_string(),
            group = "group")]
            b: i64,
            #[attr(default = "hello", importance = Importance::HIGH, documentation = "docs")]
            c: String,
            #[attr(importance = Importance::HIGH, documentation = "docs")]
            d: Vec<String>,
            #[attr(importance = Importance::HIGH, documentation = "docs")]
            e: f64,
            #[attr(importance = Importance::HIGH, documentation = "docs")]
            f: String,
            #[attr(name = "prop.f", importance = Importance::HIGH, documentation = "docs")]
            f1: String,
            #[attr(importance = Importance::HIGH, documentation = "docs")]
            g: bool,
            #[attr(name=H, importance = Importance::HIGH, documentation = "docs")]
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
        props.insert("prop.f".to_string(), "prop_f_val".to_string());
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
        assert_eq!(config.f1, "prop_f_val");
        assert_eq!(config.g, true);
        assert_eq!(config.h, false);
        assert_eq!(config.i, true);
        assert_eq!(config.j, Password::new("password".to_string()));
        assert_eq!(config.j.to_string(), "[hidden]");
    }

    #[test]
    fn test_can_add_internal_config() {
        const CONFIG_NAME: &'static str = "internal.config";
        #[derive(Debug, PartialEq, EasyConfig)]
        struct TestConfig {
            #[attr(name = CONFIG_NAME, importance = Importance::LOW)]
            val: String,
        }

        let mut props = HashMap::new();
        props.insert(CONFIG_NAME.to_string(), "value".to_string());

        let config = TestConfig::from_props(&props).unwrap();

        assert_eq!(config.val, "value");
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
            #[attr(default=-1, validator=Range::between(0, 10),
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

    macro_rules! test_validators {
        // The macro takes a test name, type, validator, default, slice of ok values,
        // slice of bad values.
        ($test_name:ident, $type:ty, $default:expr, $validator:expr, $ok_values:expr, $bad_values:expr) => {
            #[test]
            fn $test_name() {
                #[derive(Debug, EasyConfig)]
                struct TestConfig {
                    #[attr(default = $default, validator = $validator, importance = Importance::HIGH,
                    documentation = "docs")]
                    name: $type,
                }

                for &value in $ok_values {
                    let mut props = HashMap::new();
                    props.insert("name".to_string(), value.to_string());

                    let config = TestConfig::from_props(&props).unwrap_or_else(|e| {
                        panic!("Expected success for input '{}', but got error: {}", value, e)
                    });

                    let expected_val = <$type as ConfigValue>::parse("name", value).unwrap();
                    assert_eq!(config.name, expected_val);
                }

                for &value in $bad_values {
                    let mut props = HashMap::new();
                    props.insert("name".to_string(), value.to_string());

                    let result = TestConfig::from_props(&props);

                    assert!(
                        matches!(&result, Err(ConfigError::ValidationFailed { name, .. }) if name == "name"),
                        "Expected ValidationFailed error for type '{}' with input '{}', but got {:?}",
                        stringify!($type),
                        value,
                        result
                    );
                }
            }
        };
    }

    test_validators!(
        test_range_validator,
        i32,
        1,
        Range::between(0, 10),
        &["1", "5", "9"],
        &["-1", "11"]
    );

    test_validators!(
        test_string_validator,
        String,
        "default",
        ValidString::in_list(&["good", "values", "default"]),
        &["good", "values", "default"],
        &["bad", "inputs", "DEFAULT"]
    );

    test_validators!(
        test_list_validator,
        Vec<String>,
        "1",
        ValidList::in_list(&["1", "2", "3"]),
        &["1", "2", "3"],
        &["4", "5", "6"]
    );

    #[test]
    fn test_list_validator_any_non_duplicate_values() {
        let allow_any_non_duplicate_values = ValidList::any_non_duplicate_values(true);

        allow_any_non_duplicate_values
            .validate("test.config", "a, b, c")
            .unwrap();
        allow_any_non_duplicate_values
            .validate("test.config", "")
            .unwrap();

        // Test the "null allowed" case at the `from_props` level.
        #[derive(EasyConfig, Debug)]
        struct TestConfig {
            #[attr(validator = ValidList::any_non_duplicate_values(true))]
            v: Option<Vec<String>>, // `Option` makes it "null allowed"
        }
        let config = TestConfig::from_props(&HashMap::new()).unwrap();
        assert_eq!(config.v, None);

        let res = allow_any_non_duplicate_values.validate("test.config", "a, a");
        assert!(
            matches!(&res, Err(ConfigError::ValidationFailed{..}) if res.as_ref().unwrap_err().to_string()
                .eq("Validation failed for name 'test.config': \
                Configuration 'test.config' values must not be duplicated.")),
            "Expected ValidationFailed error but got {:?}",
            &res
        );

        let res = allow_any_non_duplicate_values.validate("test.config", "a,,b"); // Contains an empty string
        assert!(
            matches!(&res, Err(ConfigError::ValidationFailed{..})
                if res.as_ref().unwrap_err().to_string().eq("Validation failed for name 'test.config': \
                Configuration 'test.config' values must not be empty.")),
            "Expected ValidationFailed error but got {:?}",
            &res
        );

        let allow_any_non_duplicate_values = ValidList::any_non_duplicate_values(false);

        allow_any_non_duplicate_values
            .validate("test.config", "a, b, c")
            .unwrap();

        let res = allow_any_non_duplicate_values.validate("test.config", "");
        assert!(
            matches!(&res, Err(ConfigError::ValidationFailed{..}) if res.as_ref().unwrap_err().to_string()
                .eq("Validation failed for name 'test.config': \
                Configuration 'test.config' must not be empty. Valid values include: any non-empty value")),
            "Expected ValidationFailed error but got {:?}",
            &res
        );

        let res = allow_any_non_duplicate_values.validate("test.config", "a, a");
        assert!(
            matches!(&res, Err(ConfigError::ValidationFailed{..}) if res.as_ref().unwrap_err().to_string()
                .eq("Validation failed for name 'test.config': \
                Configuration 'test.config' values must not be duplicated.")),
            "Expected ValidationFailed error but got {:?}",
            &res
        );

        let res = allow_any_non_duplicate_values.validate("test.config", "a,,b"); // Contains an empty string
        assert!(
            matches!(&res, Err(ConfigError::ValidationFailed{..})
                if res.as_ref().unwrap_err().to_string().eq("Validation failed for name 'test.config': \
                Configuration 'test.config' values must not be empty.")),
            "Expected ValidationFailed error but got {:?}",
            &res
        );
    }

    #[test]
    fn test_list_validator_in() {
        let allow_empty_validator = ValidList::in_list(&["a", "b", "c"]);

        allow_empty_validator
            .validate("test.config", "a, b")
            .unwrap();
        allow_empty_validator.validate("test.config", "").unwrap();

        let res = allow_empty_validator.validate("test.config", "d");
        assert!(
            matches!(&res, Err(ConfigError::ValidationFailed{..}) if res.as_ref().unwrap_err().to_string()
                .eq("Validation failed for name 'test.config': \
                Invalid value 'd' for configuration 'test.config': String must be one of: a, b, c")),
            "Expected ValidationFailed error but got {:?}",
            &res
        );

        let res = allow_empty_validator.validate("test.config", "a, a");
        assert!(
            matches!(&res, Err(ConfigError::ValidationFailed{..}) if res.as_ref().unwrap_err().to_string()
                .eq("Validation failed for name 'test.config': \
                Configuration 'test.config' values must not be duplicated.")),
            "Expected ValidationFailed error but got {:?}",
            &res
        );

        let res = allow_empty_validator.validate("test.config", "a,,b"); // Contains an empty string
        assert!(
            matches!(&res, Err(ConfigError::ValidationFailed{..})
                if res.as_ref().unwrap_err().to_string().eq("Validation failed for name 'test.config': \
                Configuration 'test.config' values must not be empty.")),
            "Expected ValidationFailed error but got {:?}",
            &res
        );

        let not_allow_empty_validator = ValidList::in_list_allow_empty(false, &["a", "b", "c"]);

        not_allow_empty_validator
            .validate("test.config", "a, b")
            .unwrap();

        let res = not_allow_empty_validator.validate("test.config", "");
        assert!(
            matches!(&res, Err(ConfigError::ValidationFailed{..}) if res.as_ref().unwrap_err().to_string()
                .eq("Validation failed for name 'test.config': \
                Configuration 'test.config' must not be empty. Valid values include: [a, b, c] (empty config empty not allowed)")),
            "Expected ValidationFailed error but got {:?}",
            &res
        );

        let res = not_allow_empty_validator.validate("test.config", "a, a");
        assert!(
            matches!(&res, Err(ConfigError::ValidationFailed{..}) if res.as_ref().unwrap_err().to_string()
                .eq("Validation failed for name 'test.config': \
                Configuration 'test.config' values must not be duplicated.")),
            "Expected ValidationFailed error but got {:?}",
            &res
        );

        let res = not_allow_empty_validator.validate("test.config", "d");
        assert!(
            matches!(&res, Err(ConfigError::ValidationFailed{..}) if res.as_ref().unwrap_err().to_string()
                .eq("Validation failed for name 'test.config': \
                Invalid value 'd' for configuration 'test.config': String must be one of: a, b, c")),
            "Expected ValidationFailed error but got {:?}",
            &res
        );

        let res = not_allow_empty_validator.validate("test.config", "a,,b"); // Contains an empty string
        assert!(
            matches!(&res, Err(ConfigError::ValidationFailed{..})
                if res.as_ref().unwrap_err().to_string().eq("Validation failed for name 'test.config': \
                Configuration 'test.config' values must not be empty.")),
            "Expected ValidationFailed error but got {:?}",
            &res
        );
    }

    #[test]
    fn test_merge() {
        mod test_conf1 {
            use super::prelude::*;

            #[derive(Debug, PartialEq, EasyConfig)]
            pub struct TestConfig1 {
                #[attr(default = 5, validator=Range::between(0, 14),
                importance = Importance::HIGH, documentation = "docs", getter)]
                a1: i32,
                #[attr(default = "hello", importance = Importance::HIGH, documentation = "docs",
                getter)]
                b1: String,
            }
        }

        mod test_conf2 {
            use super::prelude::*;

            const A2_DEF_VAL: i32 = 5;
            #[derive(Debug, PartialEq, EasyConfig)]
            pub struct TestConfig2 {
                #[attr(default = A2_DEF_VAL, validator=Range::between(0, 14),
                importance = Importance::HIGH, documentation = "docs", getter)]
                a2: i32,
                #[attr(importance = Importance::HIGH, documentation = "docs", getter)]
                b2: String,
            }
        }

        #[derive(Debug, PartialEq, EasyConfig)]
        struct MergeTestConfig {
            #[merge]
            config1: test_conf1::TestConfig1,
            #[merge]
            config2: test_conf2::TestConfig2,
        }

        let mut props = HashMap::new();
        props.insert("a1".to_string(), "1   ".to_string());
        props.insert("a2".to_string(), " 2 ".to_string());
        // "b1" is omitted to test the default value.
        props.insert("b2".to_string(), "value2".to_string());

        // Act: Parse the properties into the strongly typed struct.
        let config = MergeTestConfig::from_props(&props).unwrap();

        // Assert: Check the final parsed values.
        assert_eq!(config.config1.a1(), &1);
        assert_eq!(config.config2.a2(), &2);
        assert_eq!(config.config1.b1(), "hello");
        assert_eq!(config.config2.b2(), "value2");
    }
}
