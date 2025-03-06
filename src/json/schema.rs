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

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::{BTreeMap, HashMap},
    str::FromStr,
};

use crate::attribute;

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum SchemaType {
    Object(SchemaObject),
    Enum(EnumObject),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SchemaObject {
    #[serde(rename = "$schema", skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,
    #[serde(rename = "$id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub title: String,
    #[serde(rename = "type")]
    pub dtype: DataType,
    #[serde(skip_serializing_if = "skip_empty_string")]
    pub description: Option<String>,
    pub properties: BTreeMap<String, Property>,
    #[serde(
        rename = "$defs",
        skip_serializing_if = "BTreeMap::is_empty",
        alias = "definitions"
    )]
    pub definitions: BTreeMap<String, SchemaType>,
    pub required: Vec<String>,
    #[serde(rename = "additionalProperties", default = "default_false")]
    pub additional_properties: bool,
}

impl SchemaObject {
    pub fn to_value(&self) -> Result<Value, serde_json::Error> {
        serde_json::to_value(self)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EnumObject {
    pub title: String,
    #[serde(rename = "type")]
    pub dtype: DataType,
    #[serde(skip_serializing_if = "skip_empty_string")]
    pub description: Option<String>,
    #[serde(rename = "enum")]
    pub enum_values: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Property {
    pub title: String,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub dtype: Option<DataType>,
    #[serde(rename = "default", skip_serializing_if = "Option::is_none")]
    pub default: Option<PrimitiveType>,
    #[serde(skip_serializing_if = "skip_empty_string")]
    pub description: Option<String>,
    #[serde(rename = "$term", skip_serializing_if = "skip_empty_string")]
    pub term: Option<String>,
    #[serde(rename = "$ref", skip_serializing_if = "Option::is_none")]
    pub reference: Option<String>,
    #[serde(flatten)]
    pub options: HashMap<String, PrimitiveType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Item>,
    #[serde(rename = "oneOf", skip_serializing_if = "skip_empty")]
    pub one_of: Option<Vec<Item>>,
    #[serde(skip_serializing_if = "skip_empty", rename = "enum")]
    pub enum_values: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Item {
    ReferenceItem(ReferenceItemType),
    OneOfItem(OneOfItemType),
    DataTypeItem(DataTypeItemType),
}

impl Serialize for Item {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Item::ReferenceItem(ref_item) => ref_item.serialize(serializer),
            Item::OneOfItem(one_of_item) => one_of_item.serialize(serializer),
            Item::DataTypeItem(data_type_item) => data_type_item.serialize(serializer),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ReferenceItemType {
    #[serde(rename = "$ref")]
    pub reference: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OneOfItemType {
    #[serde(rename = "oneOf")]
    pub one_of: Vec<Item>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DataTypeItemType {
    #[serde(rename = "type")]
    pub dtype: DataType,
}

/// Represents various data types that can be used in a JSON schema.
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum DataType {
    #[serde(rename = "string")]
    String,
    #[serde(rename = "integer")]
    Integer,
    #[serde(rename = "number")]
    Number,
    #[serde(rename = "boolean")]
    Boolean,
    #[serde(rename = "object")]
    Object,
    #[serde(rename = "array")]
    Array,
}

impl Default for DataType {
    /// Provides a default value for the DataType, which is `String`.
    fn default() -> Self {
        DataType::String
    }
}

impl FromStr for DataType {
    type Err = String;

    /// Converts a string representation of a data type into a `DataType` enum.
    ///
    /// # Errors
    ///
    /// Returns an error if the string is empty or does not match any known data type.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "string" => Ok(DataType::String),
            "number" => Ok(DataType::Number),
            "float" => Ok(DataType::Number),
            "integer" => Ok(DataType::Integer),
            "boolean" => Ok(DataType::Boolean),
            "object" => Ok(DataType::Object),
            "array" => Ok(DataType::Array),
            _ => Err(format!("Invalid data type: {}", s)),
        }
    }
}

impl TryFrom<&String> for DataType {
    type Error = String;

    fn try_from(s: &String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "string" => Ok(DataType::String),
            "number" => Ok(DataType::Number),
            "integer" => Ok(DataType::Integer),
            "boolean" => Ok(DataType::Boolean),
            "array" => Ok(DataType::Array),
            "float" => Ok(DataType::Number),
            _ => Ok(DataType::Object),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum PrimitiveType {
    String(String),
    Number(f64),
    Integer(i64),
    Boolean(bool),
}

impl From<&String> for PrimitiveType {
    /// Converts a string reference into a `PrimitiveType` enum.
    ///
    /// # Arguments
    ///
    /// * `s` - A reference to the string to be converted.
    ///
    /// # Returns
    ///
    /// A `PrimitiveType` enum variant corresponding to the parsed value.
    fn from(s: &String) -> Self {
        if let Ok(number) = s.parse::<f64>() {
            return PrimitiveType::Number(number);
        }

        if let Ok(boolean) = s.to_lowercase().parse::<bool>() {
            return PrimitiveType::Boolean(boolean);
        }

        if let Ok(integer) = s.parse::<i64>() {
            return PrimitiveType::Integer(integer);
        }

        PrimitiveType::String(s.clone())
    }
}

impl From<attribute::DataType> for PrimitiveType {
    fn from(dtype: attribute::DataType) -> Self {
        match dtype {
            attribute::DataType::String(s) => {
                PrimitiveType::String(s.trim_matches('"').to_string())
            }
            attribute::DataType::Integer(i) => PrimitiveType::Integer(i),
            attribute::DataType::Float(f) => PrimitiveType::Number(f),
            attribute::DataType::Boolean(b) => PrimitiveType::Boolean(b),
        }
    }
}

fn skip_empty<T>(option: &Option<Vec<T>>) -> bool {
    match option {
        Some(vec) => vec.is_empty(),
        None => true,
    }
}

fn skip_empty_string(option: &Option<String>) -> bool {
    match option {
        Some(string) => string.is_empty(),
        None => true,
    }
}

fn default_false() -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    /// Tests the conversion from string to DataType enum variants.
    /// It checks for correct parsing of basic types and custom references.
    fn test_from_str() {
        assert_eq!(DataType::from_str("string").unwrap(), DataType::String);
        assert_eq!(DataType::from_str("number").unwrap(), DataType::Number);
        assert_eq!(DataType::from_str("integer").unwrap(), DataType::Integer);
        assert_eq!(DataType::from_str("boolean").unwrap(), DataType::Boolean);
        assert_eq!(DataType::from_str("object").unwrap(), DataType::Object);
        assert_eq!(DataType::from_str("array").unwrap(), DataType::Array);
    }
}
