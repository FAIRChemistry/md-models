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

use crate::{
    attribute::{AttrOption, Attribute},
    datamodel::DataModel,
    markdown::frontmatter::FrontMatter,
    object::{Enumeration, Object},
};
use convert_case::{Case, Casing};
use reqwest::Url;
use std::{error::Error, path::Path};

use super::datatype::DataType;

static PROP_KEYS: [&str; 10] = [
    "type", "format", "enum", "minimum", "maximum", "minItems", "maxItems", "title", "items",
    "$ref",
];

/// Parse a JSON schema into an MD-Models data model
pub fn parse_json_schema(path: &Path) -> Result<DataModel, Box<dyn Error>> {
    let schema = read_json_schema(path).expect(
        "Could not read the JSON schema file. Make sure the file is a valid JSON schema file.",
    );

    // Create a new data model
    let name = schema
        .get("title")
        .expect("Could not find title in the JSON schema")
        .as_str()
        .expect("Title is not a string")
        .to_string();
    let mut model = DataModel::new(Some(name), None);
    model.config = Some(FrontMatter::default());

    // Create the root object
    let object = create_object(&schema);
    model.objects.push(object);

    // Create the rest of the objects and enums
    let definitions = schema.get("definitions").unwrap();
    for (key, value) in definitions.as_object().unwrap() {
        let data_type = DataType::from_object(value);

        match data_type {
            DataType::Object { properties: _ } => {
                let object = create_object(value);
                model.objects.push(object);
            }
            DataType::Enum { values } => {
                let enumeration = create_enum(key, &values);
                model.enums.push(enumeration);
            }
            _ => {}
        }
    }

    Ok(model)
}

/// Read JSON schema from a file
fn read_json_schema(path: &Path) -> Result<serde_json::Value, serde_json::Error> {
    let content = std::fs::read_to_string(path).expect("Could not read the JSON schema file");
    serde_json::from_str(&content)
}

fn create_enum(name: &str, values: &[String]) -> Enumeration {
    // Create a generic mapping for the enum
    let mappings = values
        .iter()
        .map(|v| (create_enum_alias(v), v.to_string()))
        .collect();

    Enumeration {
        name: name.to_string(),
        mappings,
        docstring: "".to_string(),
    }
}

fn create_enum_alias(name: &str) -> String {
    // If it is a URL, get the last part of the URL and part before the .org/.com
    let name = if let Ok(url) = Url::parse(name) {
        url_to_enum_alias(url)
    } else {
        remove_special_characters(name)
    };

    name.to_case(Case::Snake).to_uppercase()
}

fn remove_special_characters(input: &str) -> String {
    input.chars().filter(|c| c.is_alphanumeric()).collect()
}

fn url_to_enum_alias(url: Url) -> String {
    // Get the host and path
    let host = url.host_str().unwrap_or("");
    let path = url.path();

    // Remove the 'www.' prefix from the host if present
    let host = host.strip_prefix("www.").unwrap_or(host);

    // Replace dot and slash with underscore
    let mut result = host.replace('.', "_");
    result.push('_');
    result.push_str(&path.replace('/', "_"));

    // Trim trailing underscore
    result.trim_end_matches('_').to_string()
}

/// Extract properties from a JSON schema
fn create_object(schema: &serde_json::Value) -> Object {
    let name = schema
        .get("title")
        .expect("Could not find title in the JSON schema")
        .as_str()
        .expect("Title is not a string");
    let properties = schema
        .get("properties")
        .expect("Could not find properties in the JSON schema")
        .as_object()
        .expect("Properties is not an object");

    let mut object = Object::new(name.to_string(), None);

    for (key, value) in properties {
        let data_type = DataType::from_object(value);

        let mut attribute = match data_type {
            DataType::Object { properties } => process_object(key, &properties),
            DataType::Array => process_array(key, value),
            DataType::Enum { values: _ } => process_enum(key),
            DataType::Reference { reference } => process_reference(key, reference),
            _ => process_primitive(key, value),
        };

        // Add all other keys as options
        for (key, value) in value.as_object().unwrap() {
            if !PROP_KEYS.contains(&key.as_str()) {
                attribute
                    .add_option(AttrOption::new(
                        key.to_string(),
                        value.as_str().unwrap().to_string(),
                    ))
                    .expect("Failed to add option");
            }
        }

        object.attributes.push(attribute);
    }

    object
}

fn process_array(name: &str, value: &serde_json::Value) -> Attribute {
    // Prepare attribute
    let mut attribute = Attribute::new(name.to_string(), false);
    attribute.is_array = true;

    // Get the items
    let items = value
        .get("items")
        .expect("Could not find items in the array");

    // Check whether the items is a ref or any other type
    let data_type = DataType::from_object(items);

    // Set the data type
    attribute.dtypes = match data_type {
        DataType::Reference { reference } => vec![reference],
        _ => vec![data_type.to_string()],
    };

    attribute
}

fn process_primitive(name: &str, value: &serde_json::Value) -> Attribute {
    let mut attribute = Attribute::new(name.to_string(), false);
    let data_type = value
        .get("type")
        .expect("Could not find type in the property")
        .as_str()
        .expect("Type is not a string")
        .to_string();

    attribute.dtypes = vec![data_type];

    attribute
}

fn process_reference(name: &str, reference: String) -> Attribute {
    let mut attribute = Attribute::new(name.to_string(), false);
    attribute.dtypes = vec![reference];
    attribute
}

fn process_object(_name: &str, _value: &serde_json::Value) -> Attribute {
    panic!("Nested object type is not supported yet");
}

fn process_enum(_name: &str) -> Attribute {
    panic!("Property enums are currently only allowed as reference");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_json_schema() {
        // Arrange
        let path = Path::new("tests/data/expected_json_schema.json");

        // Act
        let model = parse_json_schema(path).unwrap();

        // Assert
        assert_eq!(model.objects.len(), 2);
        assert_eq!(model.enums.len(), 1);

        let object = &model.objects[0];
        assert_eq!(object.name, "Test");
        assert_eq!(object.attributes.len(), 4);

        let object = &model.objects[1];
        assert_eq!(object.name, "Test2");
        assert_eq!(object.attributes.len(), 2);

        let enumeration = &model.enums[0];
        assert_eq!(enumeration.name, "Ontology");
        assert_eq!(enumeration.mappings.len(), 3);
    }
}
