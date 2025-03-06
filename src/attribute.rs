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

use crate::{
    markdown::{parser::OptionKey, position::Position},
    option::{AttrOption, RawOption},
    xmltype::XMLType,
};
use serde::{de::Visitor, Deserialize, Serialize};
use std::{error::Error, fmt, str::FromStr};

#[cfg(feature = "python")]
use pyo3::{pyclass, pymethods};

#[cfg(feature = "wasm")]
use tsify_next::Tsify;

/// Represents an attribute with various properties and options.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "python", pyclass(get_all))]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi))]
pub struct Attribute {
    /// The name of the attribute.
    pub name: String,
    /// Indicates if the attribute is an array.
    #[serde(rename = "multiple")]
    pub is_array: bool,
    /// Is an identifier or not
    pub is_id: bool,
    /// Data types associated with the attribute.
    pub dtypes: Vec<String>,
    /// Documentation string for the attribute.
    pub docstring: String,
    /// List of additional options for the attribute.
    pub options: Vec<AttrOption>,
    /// Term associated with the attribute, if any.
    pub term: Option<String>,
    /// Indicates if the attribute is required.
    pub required: bool,
    /// Default value for the attribute.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<DataType>,
    /// XML type information for the attribute.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub xml: Option<XMLType>,
    /// Is an enumeration or not
    pub is_enum: bool,
    /// The line number of the attribute
    pub position: Option<Position>,
    /// The prefix of the attribute, if it is an import
    #[serde(skip_serializing_if = "Option::is_none")]
    pub import_prefix: Option<String>,
}

impl Attribute {
    /// Creates a new `Attribute` with the given name and required status.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the attribute.
    /// * `required` - Indicates if the attribute is required.
    pub fn new(name: String, required: bool) -> Self {
        Attribute {
            name: name.clone(),
            dtypes: Vec::new(),
            docstring: String::new(),
            options: Vec::new(),
            is_array: false,
            is_id: false,
            term: None,
            required,
            xml: Some(XMLType::from_str(name.as_str()).unwrap()),
            default: None,
            is_enum: false,
            position: None,
            import_prefix: None,
        }
    }

    /// Sets the documentation string for the attribute.
    ///
    /// # Arguments
    ///
    /// * `docstring` - The documentation string to set.
    pub fn set_docstring(&mut self, docstring: String) {
        self.docstring = docstring;
    }

    /// Sets the line number of the attribute.
    ///
    /// # Arguments
    ///
    /// * `position` - The position to set.
    pub fn set_position(&mut self, position: Position) {
        self.position = Some(position);
    }

    /// Adds an option to the attribute.
    ///
    /// # Arguments
    ///
    /// * `option` - The option to add.
    pub fn add_option(&mut self, option: RawOption) -> Result<(), Box<dyn Error>> {
        match OptionKey::from_str(option.key.as_str()) {
            OptionKey::Type => self.set_dtype(option.value)?,
            OptionKey::Term => self.term = Some(option.value),
            OptionKey::Description => self.docstring = option.value,
            OptionKey::Default => self.default = Some(DataType::from_str(&option.value)?),
            OptionKey::Multiple => self.is_array = option.value.to_lowercase() == "true",
            OptionKey::Other => self.options.push(option.try_into()?),
            OptionKey::Xml => {
                self.set_xml(XMLType::from_str(&option.value).expect("Invalid XML type"))
            }
        }

        Ok(())
    }

    /// Sets the data type for the attribute.
    ///
    /// # Arguments
    ///
    /// * `dtype` - The data type to set.
    pub(crate) fn set_dtype(&mut self, dtype: String) -> Result<(), Box<dyn Error>> {
        let mut dtypes = self.break_up_dtypes(&dtype);

        self.validate_dtypes(&dtypes)?;

        let mut new_dtypes = Vec::new();

        for dtype in dtypes.iter_mut() {
            *dtype = dtype.trim().to_string();
            if self.is_identifier(dtype) {
                *dtype = self.process_identifier(dtype);
            }

            if let Some((prefix, name)) = dtype.split_once('.') {
                self.import_prefix = Some(prefix.to_string());
                *dtype = name.to_string();
            }

            if dtype.ends_with("[]") {
                self.is_array = true;
            }

            new_dtypes.push(dtype.trim_end_matches("[]").to_string());
        }

        self.dtypes = new_dtypes;

        Ok(())
    }

    /// Splits a data type string into a vector of strings based on commas.
    ///
    /// # Arguments
    ///
    /// * `dtype` - A string representing the data types, separated by commas.
    ///
    /// # Returns
    ///
    /// A vector of strings, each representing a separate data type.
    fn break_up_dtypes(&self, dtype: &str) -> Vec<String> {
        dtype.split(",").map(|s| s.to_string()).collect()
    }

    /// Validates a vector of data type strings to ensure consistency in array notation.
    ///
    /// # Arguments
    ///
    /// * `dtypes` - A reference to a vector of strings representing data types.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or an error if the validation fails.
    fn validate_dtypes(&self, dtypes: &[String]) -> Result<(), Box<dyn Error>> {
        let has_multiple_dtypes = dtypes.len() > 1;
        let contains_array_dtype = dtypes.iter().any(|dtype| dtype.ends_with("[]"));

        if has_multiple_dtypes && contains_array_dtype {
            return Err(
                "If more than one dtype is provided, none can be array valued by []. \
                Use the keyword 'Multiple' instead."
                    .into(),
            );
        }

        Ok(())
    }

    /// Checks if a data type string represents an identifier.
    ///
    /// # Arguments
    ///
    /// * `dtype` - A string representing a data type.
    ///
    /// # Returns
    ///
    /// `true` if the data type is an identifier, `false` otherwise.
    fn is_identifier(&self, dtype: &str) -> bool {
        dtype.to_lowercase().starts_with("identifier")
    }

    /// Processes a data type string to replace 'identifier' with 'string'.
    ///
    /// # Arguments
    ///
    /// * `dtype` - A string representing a data type.
    ///
    /// # Returns
    ///
    /// A new string with 'identifier' replaced by 'string'.
    fn process_identifier(&mut self, dtype: &str) -> String {
        self.is_id = true;
        // Regex replace identifier or Identifier with string
        let pattern = regex::Regex::new(r"[I|i]dentifier").unwrap();
        pattern.replace_all(dtype, "string").to_string()
    }

    /// Converts the attribute to a JSON schema.
    ///
    /// # Returns
    ///
    /// A JSON string representing the attribute schema.
    pub fn to_json_schema(&self) -> String {
        serde_json::to_string_pretty(&self).unwrap()
    }

    /// Checks if the attribute has an associated term.
    ///
    /// # Returns
    ///
    /// `true` if the attribute has a term, `false` otherwise.
    pub fn has_term(&self) -> bool {
        self.term.is_some()
    }

    /// Sets the XML type for the attribute.
    ///
    /// # Arguments
    ///
    /// * `xml` - The XML type to set.
    pub fn set_xml(&mut self, xml: XMLType) {
        self.xml = Some(xml);
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "python", pyclass)]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi))]
pub enum DataType {
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(String),
}

#[cfg_attr(feature = "python", pymethods)]
impl DataType {
    pub fn is_boolean(&self) -> bool {
        matches!(self, DataType::Boolean(_))
    }

    pub fn is_integer(&self) -> bool {
        matches!(self, DataType::Integer(_))
    }

    pub fn is_float(&self) -> bool {
        matches!(self, DataType::Float(_))
    }

    pub fn is_string(&self) -> bool {
        matches!(self, DataType::String(_))
    }

    pub fn as_boolean(&self) -> Option<bool> {
        if let DataType::Boolean(value) = self {
            Some(*value)
        } else {
            None
        }
    }

    pub fn as_integer(&self) -> Option<i64> {
        if let DataType::Integer(value) = self {
            Some(*value)
        } else {
            None
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        if let DataType::Float(value) = self {
            Some(*value)
        } else {
            None
        }
    }

    pub fn as_string(&self) -> Option<String> {
        if let DataType::String(value) = self {
            Some(value.clone())
        } else {
            None
        }
    }
}

impl PartialEq for DataType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (DataType::Boolean(a), DataType::Boolean(b)) => a == b,
            (DataType::Integer(a), DataType::Integer(b)) => a == b,
            (DataType::Float(a), DataType::Float(b)) => a == b,
            (DataType::String(a), DataType::String(b)) => a == b,
            _ => false,
        }
    }
}

impl FromStr for DataType {
    type Err = String;

    /// Converts a string to a DataType (Boolean, Integer, Float, or String).
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lower_s = s.to_lowercase();
        let lower_s = lower_s.trim_matches('"');

        if let Ok(b) = lower_s.parse::<bool>() {
            Ok(DataType::Boolean(b))
        } else if let Ok(i) = lower_s.parse::<i64>() {
            Ok(DataType::Integer(i))
        } else if let Ok(f) = lower_s.parse::<f64>() {
            Ok(DataType::Float(f))
        } else if !lower_s.is_empty() {
            Ok(DataType::String(format!("\"{}\"", s)))
        } else {
            Err("Invalid data type".to_string())
        }
    }
}

impl Serialize for DataType {
    /// Serializes a DataType to a string.
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            DataType::Boolean(b) => serializer.serialize_bool(*b),
            DataType::Integer(i) => serializer.serialize_i64(*i),
            DataType::Float(f) => serializer.serialize_f64(*f),
            DataType::String(s) => serializer.serialize_str(s),
        }
    }
}

#[allow(clippy::needless_lifetimes)]
impl<'de> Deserialize<'de> for DataType {
    /// Deserializes a DataType from a string.
    fn deserialize<D>(deserializer: D) -> Result<DataType, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct DataTypeVisitor;
        impl<'de> Visitor<'de> for DataTypeVisitor {
            type Value = DataType;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a boolean, integer, float, or string")
            }

            fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E> {
                Ok(DataType::Boolean(v))
            }

            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E> {
                Ok(DataType::Integer(v))
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E> {
                Ok(DataType::Integer(v as i64))
            }

            fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E> {
                Ok(DataType::Float(v))
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> {
                Ok(DataType::String(v.to_string()))
            }
        }

        deserializer.deserialize_any(DataTypeVisitor)
    }
}

#[cfg(test)]
mod tests {
    use crate::xmltype::XMLType;
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_attribute_new() {
        let attr = Attribute::new("name".to_string(), false);
        assert_eq!(attr.name, "name");
        assert_eq!(attr.dtypes.len(), 0);
        assert_eq!(attr.docstring, "");
        assert_eq!(attr.options.len(), 0);
        assert_eq!(attr.is_array, false);
        assert_eq!(attr.term, None);
        assert_eq!(attr.required, false);
    }

    #[test]
    fn test_attribute_set_docstring() {
        let mut attr = Attribute::new("name".to_string(), false);
        attr.set_docstring("This is a test".to_string());
        assert_eq!(attr.docstring, "This is a test");
        assert_eq!(attr.required, false);
    }

    #[test]
    fn test_attribute_add_type_option() {
        let mut attr = Attribute::new("name".to_string(), false);
        let option = RawOption::new("type".to_string(), "string".to_string());
        attr.add_option(option).expect("Failed to add option");
        assert_eq!(attr.dtypes.len(), 1);
        assert_eq!(attr.dtypes[0], "string");
    }

    #[test]
    fn test_attribute_add_term_option() {
        let mut attr = Attribute::new("name".to_string(), false);
        let option = RawOption::new("term".to_string(), "string".to_string());
        attr.add_option(option).expect("Failed to add option");
        assert_eq!(attr.term, Some("string".to_string()));
    }

    #[test]
    fn test_attribute_add_option() {
        let mut attr = Attribute::new("name".to_string(), false);
        let option = RawOption::new("description".to_string(), "This is a test".to_string());
        attr.add_option(option).expect("Failed to add option");
        let option = RawOption::new("something".to_string(), "something".to_string());
        attr.add_option(option).expect("Failed to add option");

        assert_eq!(attr.options.len(), 1);

        if let Some(option) = attr.options.first() {
            if let AttrOption::Other { key, value } = option {
                assert_eq!(key, "something");
                assert_eq!(value, "something");
            } else {
                panic!("Option is not an AttributeOption::Other");
            }
        } else {
            panic!("Option not found");
        }

        assert_eq!(attr.docstring, "This is a test");
    }

    #[test]
    fn test_attribute_set_dtype() {
        let mut attr = Attribute::new("name".to_string(), false);
        attr.set_dtype("string".to_string())
            .expect("Failed to set dtype");
        assert_eq!(attr.dtypes.len(), 1);
        assert_eq!(attr.dtypes[0], "string");
        assert_eq!(attr.is_array, false);
    }

    #[test]
    fn test_attribute_set_array_dtype() {
        let mut attr = Attribute::new("name".to_string(), false);
        attr.set_dtype("string[]".to_string())
            .expect("Failed to set dtype");
        assert_eq!(attr.dtypes.len(), 1);
        assert_eq!(attr.dtypes[0], "string");
        assert_eq!(attr.is_array, true);
    }

    #[test]
    fn test_attribute_set_xml_attr() {
        let mut attr = Attribute::new("name".to_string(), false);
        let xml = XMLType::from_str("@name").expect("Could not parse XMLType");
        attr.set_xml(xml);
        assert_eq!(
            attr.xml.expect("Could not find XML option"),
            XMLType::Attribute {
                is_attr: true,
                name: "name".to_string(),
            },
            "XMLType is not correct. Expected an attribute type."
        );
    }

    #[test]
    fn test_attribute_set_xml_element() {
        let mut attr = Attribute::new("name".to_string(), false);
        let xml = XMLType::from_str("name").expect("Could not parse XMLType");
        attr.set_xml(xml);
        assert_eq!(
            attr.xml.expect("Could not find XML option"),
            XMLType::Element {
                is_attr: false,
                name: "name".to_string(),
            },
            "XMLType is not correct. Expected an element type."
        );
    }

    #[test]
    fn test_default_xml_type() {
        let attr = Attribute::new("name".to_string(), false);
        assert_eq!(
            attr.xml.unwrap(),
            XMLType::Element {
                is_attr: false,
                name: "name".to_string(),
            }
        );
    }

    #[test]
    fn test_serialize_data_type() {
        // Test string
        let dt = DataType::String("string".to_string());
        let serialized = serde_json::to_string(&dt).expect("Failed to serialize DataType");
        assert_eq!(serialized, "\"string\"");

        // Test integer
        let dt = DataType::Integer(1);
        let serialized = serde_json::to_string(&dt).expect("Failed to serialize DataType");
        assert_eq!(serialized, "1");

        // Test float
        let dt = DataType::Float(1.0);
        let serialized = serde_json::to_string(&dt).expect("Failed to serialize DataType");
        assert_eq!(serialized, "1.0");

        // Test boolean
        let dt = DataType::Boolean(true);
        let serialized = serde_json::to_string(&dt).expect("Failed to serialize DataType");
        assert_eq!(serialized, "true");
    }

    #[test]
    fn test_deserialize_data_type() {
        // Test string
        let deserialized: DataType =
            serde_json::from_str("\"string\"").expect("Failed to deserialize string DataType");
        assert_eq!(deserialized, DataType::String("string".to_string()));

        // Test integer
        let deserialized: DataType =
            serde_json::from_str("1").expect("Failed to deserialize integer DataType");
        assert_eq!(deserialized, DataType::Integer(1));

        // Test float
        let deserialized: DataType =
            serde_json::from_str("1.0").expect("Failed to deserialize float DataType");
        assert_eq!(deserialized, DataType::Float(1.0));

        // Test boolean
        let deserialized: DataType =
            serde_json::from_str("true").expect("Failed to deserialize bool DataType");
        assert_eq!(deserialized, DataType::Boolean(true));
    }

    #[test]
    fn is_boolean_returns_true_for_boolean() {
        let dt = DataType::Boolean(true);
        assert!(dt.is_boolean());
    }

    #[test]
    fn is_boolean_returns_false_for_non_boolean() {
        let dt = DataType::Integer(1);
        assert!(!dt.is_boolean());
    }

    #[test]
    fn is_integer_returns_true_for_integer() {
        let dt = DataType::Integer(1);
        assert!(dt.is_integer());
    }

    #[test]
    fn is_integer_returns_false_for_non_integer() {
        let dt = DataType::Boolean(true);
        assert!(!dt.is_integer());
    }

    #[test]
    fn is_float_returns_true_for_float() {
        let dt = DataType::Float(1.0);
        assert!(dt.is_float());
    }

    #[test]
    fn is_float_returns_false_for_non_float() {
        let dt = DataType::String("string".to_string());
        assert!(!dt.is_float());
    }

    #[test]
    fn is_string_returns_true_for_string() {
        let dt = DataType::String("string".to_string());
        assert!(dt.is_string());
    }

    #[test]
    fn is_string_returns_false_for_non_string() {
        let dt = DataType::Float(1.0);
        assert!(!dt.is_string());
    }

    #[test]
    fn as_boolean_returns_some_for_boolean() {
        let dt = DataType::Boolean(true);
        assert_eq!(dt.as_boolean(), Some(true));
    }

    #[test]
    fn as_boolean_returns_none_for_non_boolean() {
        let dt = DataType::Integer(1);
        assert_eq!(dt.as_boolean(), None);
    }

    #[test]
    fn as_integer_returns_some_for_integer() {
        let dt = DataType::Integer(1);
        assert_eq!(dt.as_integer(), Some(1));
    }

    #[test]
    fn as_integer_returns_none_for_non_integer() {
        let dt = DataType::Boolean(true);
        assert_eq!(dt.as_integer(), None);
    }

    #[test]
    fn as_float_returns_some_for_float() {
        let dt = DataType::Float(1.0);
        assert_eq!(dt.as_float(), Some(1.0));
    }

    #[test]
    fn as_float_returns_none_for_non_float() {
        let dt = DataType::String("string".to_string());
        assert_eq!(dt.as_float(), None);
    }

    #[test]
    fn as_string_returns_some_for_string() {
        let dt = DataType::String("string".to_string());
        assert_eq!(dt.as_string(), Some("string".to_string()));
    }

    #[test]
    fn as_string_returns_none_for_non_string() {
        let dt = DataType::Float(1.0);
        assert_eq!(dt.as_string(), None);
    }

    #[test]
    fn from_str_parses_boolean() {
        let dt = DataType::from_str("true").unwrap();
        assert_eq!(dt, DataType::Boolean(true));
    }

    #[test]
    fn from_str_parses_integer() {
        let dt = DataType::from_str("42").unwrap();
        assert_eq!(dt, DataType::Integer(42));
    }

    #[test]
    fn from_str_parses_float() {
        let dt = DataType::from_str("3.5").unwrap();
        assert_eq!(dt, DataType::Float(3.5));
    }

    #[test]
    fn from_str_parses_string() {
        let dt = DataType::from_str("hello").unwrap();
        assert_eq!(dt, DataType::String("\"hello\"".to_string()));
    }

    #[test]
    fn from_str_returns_error_for_invalid_data_type() {
        let dt = DataType::from_str("");
        assert!(dt.is_err());
    }
}
