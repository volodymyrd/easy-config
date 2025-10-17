use crate::errors::ConfigError;
use crate::prelude::Validator;
use crate::{Password, impl_config_value_for_fromstr};
use indexmap::IndexMap;
use std::any::Any;
use std::collections::{HashMap, HashSet, LinkedList};

mod macros;

/// The central bridge between raw string configurations and strongly-typed Rust values.
///
/// This trait defines the essential contract for any type that can be used as a value
/// in a configuration struct derived with `EasyConfig`. It provides a symmetric pair
/// of operations: parsing from a string (`parse`) and converting back to a string
/// (`to_config_string`).
///
/// Configuration systems inherently deal with two different worlds:
/// 1.  **The Unstructured World:** Raw text from files, environment variables, or command-line
///     arguments. In this world, every value is a `String`.
/// 2.  **The Structured World:** Your Rust code, where you want type safety, correctness,
///     and performance (e.g., working with `u32`, `bool`, or custom structs).
///
/// `ConfigValue` acts as the adapter or translator between these two worlds. The `EasyConfig`
/// derive macro uses this trait to automatically generate the code that handles this
/// translation for each field in your configuration struct.
///
/// The flow for a user-provided value is:
/// **Raw String Input** -> **`ConfigValue::parse`** -> **Typed Rust Field**
///
pub trait ConfigValue: Sized {
    /// Parses a string slice into an instance of the implementing type.
    ///
    /// This method is the primary entry point for converting user-provided configuration
    /// data into a type-safe Rust value. The `EasyConfig` macro generates code that
    /// calls this method for every field that is being populated from the input properties.
    ///
    /// # Parameters
    ///
    /// *   `key`: The configuration key (name) associated with the value. This is provided
    ///     so that implementations can create rich, user-friendly error messages. For example,
    ///     if parsing fails, the error can state exactly which configuration key had the
    ///     invalid value (e.g., `"Failed to parse 'server.port': not a valid integer"`).
    ///
    /// *   `value_str`: The raw string value from the configuration source to be parsed.
    ///
    /// # Returns
    ///
    /// *   `Ok(Self)`: If the string is successfully parsed into a value of type `Self`.
    /// *   `Err(ConfigError)`: If parsing fails. The error should be descriptive.
    ///
    /// # Implementation Notes
    ///
    // Implementors should typically trim whitespace from `value_str` before parsing.
    // For types that implement the standard `FromStr` trait, this method can often be
    // a simple wrapper around the standard `parse()` method, mapping the error into a
    // `ConfigError`.
    fn parse(key: &str, value_str: &str) -> Result<Self, ConfigError>;

    /// Converts an instance of the type back into its canonical string representation.
    ///
    /// This method provides the reverse operation of `parse`. It takes a typed value
    /// and serializes it back into a `String`.
    ///
    /// The primary and most critical use case for this method is **validating default values**.
    ///
    /// The validation system is designed to operate consistently on raw strings, regardless
    /// of their origin. However, default values are defined directly in your Rust code with
    /// their correct type (e.g., `#[attr(default = 5, ...)] a: i32;`). Here, the default is
    /// an `i32`, not a `String`.
    ///
    /// To ensure that a default value is valid, according to its own rules, the system
    /// must be able to pass it to its validator. Since the validator expects a `&str`,
    /// `to_config_string` is called to bridge this type gap.
    ///
    /// The process is as follows:
    ///
    /// 1.  A field is defined with a strongly-typed default:
    ///     `#[attr(default = 5, validator = Range::between(0, 10))]`.
    /// 2.  The macro stores the value `5` as an `i32`.
    /// 3.  When processing the configuration, the system retrieves this `i32` value.
    /// 4.  To validate it, the system calls `to_config_string()` on the `i32`, which produces
    ///     the `String` `"5"`.
    /// 5.  This string `"5"` is then passed to the `Range` validator's `validate` method.
    ///
    /// This ensures that the same validation logic is applied consistently to both
    /// user-provided values and developer-provided defaults, preventing invalid
    /// default configurations.
    fn to_config_string(&self) -> String;
}

/// A uniform, type-erased interface for configuration key metadata.
///
///
/// This trait is the cornerstone of the `EasyConfig` system's type safety and flexibility.
/// A configuration struct can contain fields of many different types (e.g., `i32`, `String`,
/// `Vec<String>`).
/// The metadata for each field is stored in a `ConfigKey<T>` struct, where `T` is the field's type.
/// This means a `ConfigKey<i32>` and a `ConfigKey<String>` are two completely different,
/// incompatible Rust types.
///
/// The system needs a single, central place (`ConfigDef`) to store the metadata for *all*
/// fields together in one collection. This trait provides a **common interface** that
/// "erases" the specific generic type `T`. By using a trait object (`Box<dyn ConfigKeyTrait>`),
/// we can store the metadata for any `ConfigKey<T>` in the same collection, treating them
/// all identically.
///
/// This allows for powerful introspection, where code can iterate over all configuration
/// keys to generate documentation or UIs without needing to know the concrete type of each key's
/// value.
///
/// The `Send + Sync` bounds are required because the collected metadata is stored in a
/// global `static` variable (`once_cell::sync::OnceCell`), which must be safe to access from any
/// thread.
pub trait ConfigKeyTrait: Send + Sync {
    fn name(&self) -> &'static str;
    fn documentation(&self) -> Option<&String>;
    fn default_value_any(&self) -> Option<&dyn Any>;
    fn validator(&self) -> Option<&dyn Validator>;
    fn importance(&self) -> Option<Importance>;
    fn group(&self) -> Option<&String>;
    fn internal_config(&self) -> bool;
    /// Clones the underlying concrete `ConfigKey<T>` and returns it as a new trait object.
    ///
    /// Trait objects (`dyn Trait`) are "unsized" and cannot implement `Clone` directly.
    /// This method is a standard Rust pattern to enable cloning of trait objects by boxing
    /// the cloned concrete instance.
    fn clone_box(&self) -> Box<dyn ConfigKeyTrait>;
    /// Returns the underlying `ConfigKey<T>` as a `&dyn Any`.
    ///
    /// This provides an escape hatch for advanced use cases where code might need to
    /// determine the original concrete type of the `ConfigKey<T>` itself.
    fn as_any(&self) -> &dyn Any;
}

/// This struct holds all the metadata for a single configuration field within your
/// `EasyConfig`-derived struct. It is generic over `T`, where `T` is the actual type
/// of the field's value (e.g., for `a: i32`, the metadata is a `ConfigKey<i32>`).
///
/// The `EasyConfig` derive macro constructs an instance of this struct for each
/// field it processes, gathering the information from the `#[attr(...)]` attributes.
#[derive(Debug, Clone)]
pub struct ConfigKey<T: 'static + Clone + Send + Sync + ConfigValue> {
    pub name: &'static str,
    pub documentation: Option<String>,
    pub default_value: Option<T>,
    pub validator: Option<Box<dyn Validator>>,
    pub importance: Option<Importance>,
    pub group: Option<String>,
    pub internal_config: bool,
}

/// This struct acts as the central repository or "single source of truth" for all
/// metadata related to a configuration struct. It is generated once by the `EasyConfig`
/// macro, then stored in a `static` variable for efficient, repeated access.
///
/// Its primary role is to hold a collection of all configuration keys, abstracted
/// as `Box<dyn ConfigKeyTrait>` objects. This allows the `from_props` method to
/// look up the rules (like name, type, default value, and validator) for any given key.
#[derive(Default, Clone)]
pub struct ConfigDef {
    /// A map of all configuration keys defined in the struct.
    ///
    /// An `IndexMap` is used because it preserves the original insertion order of the keys.
    /// This is highly desirable for user-facing applications, such as generating help text
    /// or documentation, where the order of fields should match their declaration in the struct.
    /// It also provides efficient `O(1)` average-case lookup by key name.
    ///
    /// Box<dyn ConfigKeyTrait>` this is where the type erasure from `ConfigKeyTrait` is put to use.
    /// Storing the metadata as trait objects allows this single map to hold the definitions for keys
    /// of many different value types (`i32`, `String`, etc.) together
    config_keys: IndexMap<&'static str, Box<dyn ConfigKeyTrait>>,
    _groups: LinkedList<String>,
}

/// The primary trait implemented by structs that derive `EasyConfig`.
///
/// This trait provides the main entry points for working with a configuration struct.
/// It connects the struct's definition (`ConfigDef`) with the logic for parsing
/// raw properties into a strongly-typed instance of the struct.
pub trait FromConfigDef: Sized {
    /// Parses a map of raw string properties into an instance of the struct.
    fn from_props(props: &HashMap<String, String>) -> Result<Self, ConfigError>;

    /// Provides access to the static configuration schema (`ConfigDef`).
    fn config_def() -> Result<&'static ConfigDef, ConfigError>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Importance {
    HIGH,
    MEDIUM,
    LOW,
}

impl Clone for Box<dyn ConfigKeyTrait> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

impl<T: 'static + Clone + Send + Sync + ConfigValue> ConfigKeyTrait for ConfigKey<T> {
    fn name(&self) -> &'static str {
        self.name
    }
    fn documentation(&self) -> Option<&String> {
        self.documentation.as_ref()
    }
    fn default_value_any(&self) -> Option<&dyn Any> {
        self.default_value.as_ref().map(|v| v as &dyn Any)
    }
    fn validator(&self) -> Option<&dyn Validator> {
        self.validator.as_deref()
    }
    fn importance(&self) -> Option<Importance> {
        self.importance
    }
    fn group(&self) -> Option<&String> {
        self.group.as_ref()
    }
    fn internal_config(&self) -> bool {
        self.internal_config
    }
    fn clone_box(&self) -> Box<dyn ConfigKeyTrait> {
        Box::new(self.clone())
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl ConfigDef {
    pub fn find_key(&self, name: &str) -> Option<&dyn ConfigKeyTrait> {
        self.config_keys.get(name).map(|k| k.as_ref())
    }

    pub fn config_keys(&self) -> &IndexMap<&'static str, Box<dyn ConfigKeyTrait>> {
        &self.config_keys
    }
}

impl TryFrom<Vec<Box<dyn ConfigKeyTrait>>> for ConfigDef {
    type Error = ConfigError;

    fn try_from(keys: Vec<Box<dyn ConfigKeyTrait>>) -> Result<Self, Self::Error> {
        let mut config_keys = IndexMap::with_capacity(keys.len());
        for key in keys {
            if let Some(existing_key) = config_keys.insert(key.name(), key) {
                return Err(ConfigError::ValidationFailed {
                    name: existing_key.name().to_string(),
                    message: format!(
                        "Configuration key '{}' is defined twice.",
                        existing_key.name()
                    ),
                });
            }
        }

        let mut seen_groups = HashSet::new();
        let groups: LinkedList<String> = config_keys
            .values()
            .filter_map(|k| k.group())
            .filter(|g| seen_groups.insert((*g).clone()))
            .cloned()
            .collect();

        Ok(ConfigDef {
            config_keys,
            _groups: groups,
        })
    }
}

impl_config_value_for_fromstr!(
    bool, u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64
);

impl ConfigValue for String {
    fn parse(_key: &str, s: &str) -> Result<Self, ConfigError> {
        Ok(s.trim().to_string())
    }
    fn to_config_string(&self) -> String {
        self.clone()
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
    fn to_config_string(&self) -> String {
        self.join(",")
    }
}

impl ConfigValue for Password {
    fn parse(_key: &str, s: &str) -> Result<Self, ConfigError> {
        Ok(Password::new(s.trim().to_string()))
    }
    fn to_config_string(&self) -> String {
        self.password().to_string()
    }
}
