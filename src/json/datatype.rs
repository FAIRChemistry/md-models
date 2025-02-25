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

use std::{fmt::Display, str::FromStr};

#[derive(Debug, PartialEq, Clone)]
pub enum DataType {
    String,
    Integer,
    Number,
    Boolean,
    Array,
}

impl Display for DataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataType::String => write!(f, "string"),
            DataType::Integer => write!(f, "integer"),
            DataType::Number => write!(f, "number"),
            DataType::Boolean => write!(f, "boolean"),
            DataType::Array => write!(f, "array"),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_type_from_str() {
        let data_type = "string".parse::<DataType>().unwrap();
        assert_eq!(data_type, DataType::String);
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
    }
}
