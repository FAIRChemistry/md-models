/*
 * Copyright (c) 2024 Jan Range
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

use std::{fmt::Display, str::FromStr};

#[derive(Debug, PartialEq)]
pub enum DataType {
    String,
    Integer,
    Number,
    Boolean,
    Object { properties: serde_json::Value },
    Array,
    Enum { values: Vec<String> },
    Reference { reference: String },
}

impl Display for DataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataType::String => write!(f, "string"),
            DataType::Integer => write!(f, "integer"),
            DataType::Number => write!(f, "number"),
            DataType::Boolean => write!(f, "boolean"),
            DataType::Array => write!(f, "array"),
            DataType::Reference { reference } => write!(f, "reference [{}]", reference),
            DataType::Object { properties } => {
                let properties = properties
                    .as_object()
                    .unwrap()
                    .keys()
                    .map(|k| k.to_string())
                    .collect::<Vec<String>>()
                    .join(", ");
                write!(f, "object [{}]", properties)
            }
            DataType::Enum { values } => {
                let values = values.join(", ");
                write!(f, "enum [{}]", values)
            }
        }
    }
}

impl FromStr for DataType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "string" => Ok(DataType::String),
            "integer" => Ok(DataType::Integer),
            "number" => Ok(DataType::Number),
            "boolean" => Ok(DataType::Boolean),
            "array" => Ok(DataType::Array),
            _ => Err(format!("Unknown data type: {}", s)),
        }
    }
}

impl DataType {
    pub fn from_object(value: &serde_json::Value) -> Self {
        if let Some(reference) = value.get("$ref") {
            return DataType::Reference {
                reference: reference
                    .as_str()
                    .unwrap()
                    .split('/')
                    .last()
                    .unwrap()
                    .to_string(),
            };
        } else if let Some(values) = value.get("enum") {
            let values = values
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_str().unwrap().to_string())
                .collect();
            return DataType::Enum { values };
        } else if let Some(data_type) = value.get("type") {
            let data_type = data_type.as_str().unwrap();
            if data_type == "object" {
                let properties = value.get("properties").unwrap();
                return DataType::Object {
                    properties: properties.clone(),
                };
            } else {
                return data_type.parse().expect(
                    "Could not parse the data type. Make sure the data type is a valid type.",
                );
            }
        } else {
            panic!("Could not find a data type in the JSON schema");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_type_from_str() {
        let data_type = "string".parse::<DataType>().unwrap();
        assert_eq!(data_type, DataType::String);
    }

    #[test]
    fn test_data_type_from_object() {
        let data_type = DataType::from_object(&serde_json::json!({
            "type": "string"
        }));
        assert_eq!(data_type, DataType::String);
    }

    #[test]
    fn test_data_type_from_object_with_enum() {
        let data_type = DataType::from_object(&serde_json::json!({
            "enum": ["one", "two"]
        }));
        assert_eq!(
            data_type,
            DataType::Enum {
                values: vec!["one".to_string(), "two".to_string()]
            }
        );
    }

    #[test]
    fn test_data_type_from_object_with_reference() {
        let data_type = DataType::from_object(&serde_json::json!({
            "$ref": "#/definitions/Person"
        }));
        assert_eq!(
            data_type,
            DataType::Reference {
                reference: "Person".to_string()
            }
        );
    }

    #[test]
    fn test_data_type_from_object_with_object() {
        let data_type = DataType::from_object(&serde_json::json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string"
                }
            }
        }));
        assert_eq!(
            data_type,
            DataType::Object {
                properties: serde_json::json!({
                    "name": {
                        "type": "string"
                    }
                })
            }
        );
    }

    #[test]
    fn test_data_type_display() {
        let data_type = DataType::String;
        assert_eq!(data_type.to_string(), "string");

        let data_type = DataType::Integer;
        assert_eq!(data_type.to_string(), "integer");

        let data_type = DataType::Number;
        assert_eq!(data_type.to_string(), "number");

        let data_type = DataType::Boolean;
        assert_eq!(data_type.to_string(), "boolean");

        let data_type = DataType::Array;
        assert_eq!(data_type.to_string(), "array");

        let data_type = DataType::Reference {
            reference: "Person".to_string(),
        };
        assert_eq!(data_type.to_string(), "reference [Person]");

        let data_type = DataType::Object {
            properties: serde_json::json!({
                "name": {
                    "type": "string"
                }
            }),
        };
        assert_eq!(data_type.to_string(), "object [name]");

        let data_type = DataType::Enum {
            values: vec!["one".to_string(), "two".to_string()],
        };
        assert_eq!(data_type.to_string(), "enum [one, two]");
    }

    #[test]
    #[should_panic]
    fn test_display_panic() {
        let data_type = DataType::from_object(&serde_json::json!({}));
        data_type.to_string();
    }
}
