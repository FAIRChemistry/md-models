/*
 * Copyright (c) 2025 Jan Range
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in
 * all copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
 * THE SOFTWARE.
 *
 */

use std::str::FromStr;

use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

#[cfg(feature = "python")]
use pyo3::pyclass;

#[cfg(feature = "wasm")]
use tsify_next::Tsify;

/// Represents an option for an attribute in a data model.
///
/// This enum provides a strongly-typed representation of various attribute options
/// that can be used to configure and constrain attributes in a data model.
///
/// The options are grouped into several categories:
/// - JSON Schema validation options (e.g., minimum/maximum values, length constraints)
/// - SQL database options (e.g., primary key)
/// - LinkML specific options (e.g., readonly, recommended)
/// - Custom options via the `Other` variant
///
#[derive(Debug, Clone, PartialEq, EnumString, Display)]
#[cfg_attr(feature = "python", pyclass(get_all))]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi))]
#[derive(Serialize, Deserialize)]
#[serde(try_from = "RawOption")]
#[serde(into = "RawOption")]
pub enum AttrOption {
    // General options
    Example(String),

    // JSON Schema validation options
    /// Specifies the minimum value for a numeric attribute
    #[strum(serialize = "minimum")]
    MinimumValue(f64),
    /// Specifies the maximum value for a numeric attribute
    #[strum(serialize = "maximum")]
    MaximumValue(f64),
    /// Specifies the minimum number of items for an array attribute
    #[strum(serialize = "minitems")]
    MinItems(usize),
    /// Specifies the maximum number of items for an array attribute
    #[strum(serialize = "maxitems")]
    MaxItems(usize),
    /// Specifies the minimum length for a string attribute
    #[strum(serialize = "minlength")]
    MinLength(usize),
    /// Specifies the maximum length for a string attribute
    #[strum(serialize = "maxlength")]
    MaxLength(usize),
    /// Specifies a regular expression pattern that a string attribute must match
    #[strum(serialize = "pattern", serialize = "regex")]
    Pattern(String),
    /// Specifies whether array items must be unique
    #[strum(serialize = "unique")]
    Unique(bool),
    /// Specifies that a numeric value must be a multiple of this number
    #[strum(serialize = "multipleof")]
    MultipleOf(i32),
    /// Specifies an exclusive minimum value for a numeric attribute
    #[strum(serialize = "exclusiveminimum")]
    ExclusiveMinimum(f64),
    /// Specifies an exclusive maximum value for a numeric attribute
    #[strum(serialize = "exclusivemaximum")]
    ExclusiveMaximum(f64),

    // SQL database options
    /// Indicates whether the attribute is a primary key in a database
    #[strum(serialize = "pk")]
    PrimaryKey(bool),

    // LinkML specific options
    /// Indicates whether the attribute is read-only
    #[strum(serialize = "readonly")]
    ReadOnly(bool),
    /// Indicates whether the attribute is recommended to be included
    #[strum(serialize = "recommended")]
    Recommended(bool),

    // Custom options
    /// Represents a custom option not covered by the predefined variants
    Other {
        /// The key/name of the custom option
        key: String,
        /// The value of the custom option
        value: String,
    },
}

impl AttrOption {
    /// Creates a new `AttrOption` from a key-value pair.
    ///
    /// This method attempts to parse the key and value into an appropriate `AttrOption` variant.
    /// If the key matches a known option type, it will attempt to parse the value into the
    /// appropriate type. If the key is not recognized, it will create an `Other` variant.
    ///
    /// # Arguments
    ///
    /// * `key` - The string key identifying the option type
    /// * `value` - The string value to be parsed into the appropriate type
    ///
    /// # Returns
    ///
    /// A `Result` containing either the parsed `AttrOption` or an error if parsing fails
    pub fn from_pair(key: &str, value: &str) -> Result<Self, Box<dyn std::error::Error>> {
        match AttrOption::from_str(key) {
            Ok(option) => match option {
                AttrOption::MinimumValue(_) => Ok(AttrOption::MinimumValue(value.parse()?)),
                AttrOption::MaximumValue(_) => Ok(AttrOption::MaximumValue(value.parse()?)),
                AttrOption::MinItems(_) => Ok(AttrOption::MinItems(value.parse()?)),
                AttrOption::MaxItems(_) => Ok(AttrOption::MaxItems(value.parse()?)),
                AttrOption::MinLength(_) => Ok(AttrOption::MinLength(value.parse()?)),
                AttrOption::MaxLength(_) => Ok(AttrOption::MaxLength(value.parse()?)),
                AttrOption::Pattern(_) => Ok(AttrOption::Pattern(value.to_string())),
                AttrOption::Unique(_) => Ok(AttrOption::Unique(value.parse()?)),
                AttrOption::MultipleOf(_) => Ok(AttrOption::MultipleOf(value.parse()?)),
                AttrOption::ExclusiveMinimum(_) => Ok(AttrOption::ExclusiveMinimum(value.parse()?)),
                AttrOption::ExclusiveMaximum(_) => Ok(AttrOption::ExclusiveMaximum(value.parse()?)),
                AttrOption::PrimaryKey(_) => Ok(AttrOption::PrimaryKey(value.parse()?)),
                AttrOption::ReadOnly(_) => Ok(AttrOption::ReadOnly(value.parse()?)),
                AttrOption::Recommended(_) => Ok(AttrOption::Recommended(value.parse()?)),
                AttrOption::Example(_) => Ok(AttrOption::Example(value.to_string())),
                AttrOption::Other { .. } => unreachable!(),
            },
            Err(_) => Ok(AttrOption::Other {
                key: key.to_string(),
                value: value.to_string(),
            }),
        }
    }

    /// Converts the option into a key-value pair.
    ///
    /// # Returns
    ///
    /// A tuple containing the option's key and value as strings
    pub fn to_pair(&self) -> (String, String) {
        (self.key(), self.value())
    }

    /// Gets the key/name of the option.
    ///
    /// For predefined options, this returns the serialized name.
    /// For custom options, this returns the custom key.
    ///
    /// # Returns
    ///
    /// The option's key as a String
    pub fn key(&self) -> String {
        match self {
            AttrOption::Other { key, .. } => key.to_string(),
            _ => self.to_string(),
        }
    }

    /// Gets the value of the option as a string.
    ///
    /// # Returns
    ///
    /// The option's value converted to a String
    pub fn value(&self) -> String {
        match self {
            AttrOption::Other { value, .. } => value.to_string(),
            AttrOption::MinimumValue(value) => value.to_string(),
            AttrOption::MaximumValue(value) => value.to_string(),
            AttrOption::MinItems(value) => value.to_string(),
            AttrOption::MaxItems(value) => value.to_string(),
            AttrOption::MinLength(value) => value.to_string(),
            AttrOption::MaxLength(value) => value.to_string(),
            AttrOption::Pattern(value) => value.to_string(),
            AttrOption::Unique(value) => value.to_string(),
            AttrOption::MultipleOf(value) => value.to_string(),
            AttrOption::ExclusiveMinimum(value) => value.to_string(),
            AttrOption::ExclusiveMaximum(value) => value.to_string(),
            AttrOption::PrimaryKey(value) => value.to_string(),
            AttrOption::ReadOnly(value) => value.to_string(),
            AttrOption::Recommended(value) => value.to_string(),
            AttrOption::Example(value) => value.to_string(),
        }
    }
}

/// A raw key-value representation of an attribute option.
///
/// This struct provides a simple string-based representation of options,
/// which is useful for serialization/deserialization and when working
/// with untyped data.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "python", pyclass(get_all))]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi))]
pub struct RawOption {
    /// The key/name of the option
    pub key: String,
    /// The string value of the option
    pub value: String,
}

impl RawOption {
    /// Creates a new `RawOption` with the given key and value.
    ///
    /// The key is automatically converted to lowercase for consistency.
    ///
    /// # Arguments
    ///
    /// * `key` - The key/name of the option
    /// * `value` - The value of the option
    pub fn new(key: String, value: String) -> Self {
        Self {
            key: key.to_lowercase(),
            value,
        }
    }

    /// Gets a reference to the option's key.
    ///
    /// # Returns
    ///
    /// A string slice containing the option's key
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Gets a reference to the option's value.
    ///
    /// # Returns
    ///
    /// A string slice containing the option's value
    pub fn value(&self) -> &str {
        &self.value
    }
}

impl TryFrom<RawOption> for AttrOption {
    type Error = Box<dyn std::error::Error>;

    fn try_from(option: RawOption) -> Result<Self, Self::Error> {
        AttrOption::from_pair(&option.key, &option.value)
    }
}

impl From<AttrOption> for RawOption {
    fn from(option: AttrOption) -> Self {
        RawOption::new(option.key(), option.value())
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::prelude::DataModel;
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_from_pair_basic() {
        let cases = vec![
            ("minimum", "10.5", AttrOption::MinimumValue(10.5)),
            ("maximum", "100.0", AttrOption::MaximumValue(100.0)),
            ("minitems", "5", AttrOption::MinItems(5)),
            ("maxitems", "10", AttrOption::MaxItems(10)),
            ("minlength", "3", AttrOption::MinLength(3)),
            ("maxlength", "20", AttrOption::MaxLength(20)),
            (
                "pattern",
                "^[a-z]+$",
                AttrOption::Pattern("^[a-z]+$".to_string()),
            ),
            (
                "regex",
                "^[a-z]+$",
                AttrOption::Pattern("^[a-z]+$".to_string()),
            ),
            ("unique", "true", AttrOption::Unique(true)),
            ("multipleof", "3", AttrOption::MultipleOf(3)),
            ("exclusiveminimum", "0.5", AttrOption::ExclusiveMinimum(0.5)),
            (
                "exclusivemaximum",
                "99.9",
                AttrOption::ExclusiveMaximum(99.9),
            ),
            ("pk", "true", AttrOption::PrimaryKey(true)),
            ("readonly", "false", AttrOption::ReadOnly(false)),
            ("recommended", "true", AttrOption::Recommended(true)),
        ];

        for (key, value, expected) in cases {
            let result = AttrOption::from_pair(key, value).unwrap();
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_from_pair_other() {
        let result = AttrOption::from_pair("custom_option", "value").unwrap();
        assert_eq!(
            result,
            AttrOption::Other {
                key: "custom_option".to_string(),
                value: "value".to_string()
            }
        );
    }

    #[test]
    fn test_from_pair_invalid_values() {
        // Test invalid number formats
        assert!(AttrOption::from_pair("minimum", "not_a_number").is_err());
        assert!(AttrOption::from_pair("minitems", "-1").is_err());
        assert!(AttrOption::from_pair("multipleof", "3.5").is_err());

        // Test invalid boolean formats
        assert!(AttrOption::from_pair("unique", "not_a_bool").is_err());
        assert!(AttrOption::from_pair("pk", "invalid").is_err());
    }

    #[test]
    fn test_to_pair() {
        let cases = vec![
            (
                AttrOption::MinimumValue(10.5),
                ("minimum".to_string(), "10.5".to_string()),
            ),
            (
                AttrOption::Pattern("^test$".to_string()),
                ("pattern".to_string(), "^test$".to_string()),
            ),
            (
                AttrOption::Other {
                    key: "custom".to_string(),
                    value: "test".to_string(),
                },
                ("custom".to_string(), "test".to_string()),
            ),
        ];

        for (option, expected) in cases {
            assert_eq!(option.to_pair(), expected);
        }
    }

    #[test]
    fn test_raw_option_conversion() {
        let raw = RawOption::new("minimum".to_string(), "10.5".to_string());
        let attr: AttrOption = raw.try_into().unwrap();
        assert_eq!(attr, AttrOption::MinimumValue(10.5));

        let raw_back: RawOption = attr.into();
        assert_eq!(raw_back.key(), "minimum");
        assert_eq!(raw_back.value(), "10.5");
    }

    #[test]
    fn test_raw_option_case_sensitivity() {
        let raw = RawOption::new("MINIMUM".to_string(), "10.5".to_string());
        let attr: AttrOption = raw.try_into().unwrap();
        assert_eq!(attr, AttrOption::MinimumValue(10.5));
    }

    #[test]
    fn test_raw_option_serialize() {
        let raw = RawOption::new("minimum".to_string(), "10.5".to_string());
        let serialized = serde_json::to_string(&raw).unwrap();
        assert_eq!(serialized, r#"{"key":"minimum","value":"10.5"}"#);
    }

    #[test]
    fn test_raw_option_deserialize() {
        let serialized = r#"{"key":"minimum","value":"10.5"}"#;
        let deserialized: RawOption = serde_json::from_str(serialized).unwrap();
        assert_eq!(deserialized.key(), "minimum");
        assert_eq!(deserialized.value(), "10.5");
    }

    #[test]
    fn test_attr_option_from_str() {
        let path = PathBuf::from("tests/data/model_options.md");
        let model = DataModel::from_markdown(&path).expect("Failed to parse markdown file");
        let attr = model.objects.first().unwrap();
        let attribute = attr.attributes.first().unwrap();
        let options = attribute
            .options
            .iter()
            .map(|o| o.key())
            .collect::<Vec<_>>();

        let expected = vec![
            "minimum",
            "maximum",
            "minitems",
            "maxitems",
            "minlength",
            "maxlength",
            "pattern",
            "unique",
            "multipleof",
            "exclusiveminimum",
            "exclusivemaximum",
            "primarykey",
            "readonly",
            "recommended",
        ];

        let mut missing = Vec::new();
        for expected_option in expected {
            if !options.contains(&expected_option.to_string()) {
                missing.push(expected_option);
            }
        }
        assert!(
            missing.is_empty(),
            "Expected options \n[{}]\nnot found in \n[{}]",
            missing.join(", "),
            options.join(", ")
        );

        // Assert that the content of the options is correct
        let expected_options = vec![
            AttrOption::Example("test".to_string()),
            AttrOption::MinimumValue(0.0),
            AttrOption::MaximumValue(100.0),
            AttrOption::MinItems(1),
            AttrOption::MaxItems(10),
            AttrOption::MinLength(1),
            AttrOption::MaxLength(100),
            AttrOption::Pattern("^[a-zA-Z0-9]+$".to_string()),
            AttrOption::Pattern("^[a-zA-Z0-9]+$".to_string()),
            AttrOption::Unique(true),
            AttrOption::MultipleOf(2),
            AttrOption::ExclusiveMinimum(0.0),
            AttrOption::ExclusiveMaximum(100.0),
            AttrOption::PrimaryKey(true),
            AttrOption::ReadOnly(true),
            AttrOption::Recommended(true),
        ];

        for expected_option in expected_options.iter() {
            for option in attribute.options.iter() {
                if option.key() == expected_option.key() {
                    assert_eq!(option, expected_option);
                }
            }
        }
    }
}
