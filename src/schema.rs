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

use crate::attribute;
use crate::attribute::AttrOption;
use crate::datamodel::DataModel;
use crate::object::{self, Enumeration};
use crate::primitives::PrimitiveTypes;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashSet;
use std::fmt::Display;
use std::hash::Hash;

static DEFINITIONS_KEY: &str = "definitions";
static SCHEMA_VERSION: &str = "http://json-schema.org/draft-07/schema";

#[derive(PartialEq, Eq, Debug)]
enum RefType {
    Object(String),
    Enum(String),
}

impl Hash for RefType {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            RefType::Object(name) => name.hash(state),
            RefType::Enum(name) => name.hash(state),
        }
    }
}

impl Display for RefType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RefType::Object(name) => write!(f, "{}", name),
            RefType::Enum(name) => write!(f, "{}", name),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct JSONSchema {
    #[serde(rename = "$schema")]
    schema: String,
    #[serde(flatten)]
    definitions: serde_json::Value,
}

/// Converts a data model to a JSON schema.
///
/// # Arguments
/// * `name` - The name of the object to convert.
/// * `model` - The data model containing the objects and enums.
///
/// # Returns
/// A JSON string representing the schema.
pub fn to_json_schema(name: &String, model: &DataModel) -> String {
    let objects = &model.objects;
    let obj = objects.iter().find(|o| o.name == *name).unwrap();
    let (mut schema, used_refs) = process_class(obj, model);

    // Get unique used_refs
    let used_refs = used_refs.into_iter().collect::<HashSet<RefType>>();

    for reference in used_refs {
        match reference {
            RefType::Object(name) => {
                let sub_obj = objects.iter().find(|o| o.name == name).unwrap();
                let (properties, _) = process_class(sub_obj, model);
                schema[DEFINITIONS_KEY][name] = properties;
            }
            RefType::Enum(name) => {
                let sub_enum = model.enums.iter().find(|e| e.name == name).unwrap();
                let properties = process_enum(sub_enum);
                schema[DEFINITIONS_KEY][name] = properties;
            }
        }
    }

    let schema = JSONSchema {
        schema: SCHEMA_VERSION.to_string(),
        definitions: schema,
    };

    serde_json::to_string_pretty(&schema).unwrap()
}

/// Processes a class object to generate its JSON schema and collect references.
///
/// # Arguments
/// * `object` - The object to process.
/// * `model` - The data model containing the objects and enums.
///
/// # Returns
/// A tuple containing the JSON schema and a set of references.
fn process_class(
    object: &object::Object,
    model: &DataModel,
) -> (serde_json::Value, HashSet<RefType>) {
    // Retrieve all object and enum names
    let object_names = model
        .objects
        .iter()
        .map(|o| o.name.clone())
        .collect::<HashSet<String>>();
    let enum_names = model
        .enums
        .iter()
        .map(|e| e.name.clone())
        .collect::<HashSet<String>>();

    // Initialize the schema and references
    let mut all_refs = HashSet::new();
    let mut schema = json!({
        "title": object.name,
        "type": "object",
        "properties": {},
    });

    if !object.docstring.is_empty() {
        schema["description"] = json!(object.docstring);
    }

    if object.term.is_some() {
        schema["term"] = json!(object.term.as_ref().unwrap());
    }

    for attribute in &object.attributes {
        let (primitives, references) = extract_primitives_and_refs(&attribute.dtypes);

        for primitive in primitives {
            process_primitive(&mut schema["properties"], attribute, &primitive);
        }

        for reference in references {
            if enum_names.contains(&reference) {
                all_refs.insert(RefType::Enum(reference.clone()));
                process_enum_reference(
                    &attribute.name,
                    &mut schema["properties"],
                    reference.as_str(),
                );
            } else if object_names.contains(&reference) {
                all_refs.insert(RefType::Object(reference.clone()));
                process_reference(&mut schema["properties"], attribute, &reference);
            } else {
                panic!("Reference {} not found in the markdown file", reference);
            }
        }
    }

    (schema, all_refs)
}

fn process_enum(enumeration: &Enumeration) -> serde_json::Value {
    let values = enumeration
        .mappings
        .values()
        .cloned()
        .collect::<Vec<String>>();

    json!({
        "title": enumeration.name,
        "type": "string",
        "enum": values,
    })
}

/// Extracts primitive types and references from a list of data types.
///
/// # Arguments
/// * `dtypes` - The list of data types to process.
///
/// # Returns
/// A tuple containing lists of primitive types and references.
fn extract_primitives_and_refs(dtypes: &Vec<String>) -> (Vec<String>, Vec<String>) {
    let primitives = PrimitiveTypes::new();
    let references = primitives.filter_non_primitives(dtypes);
    let primitives = primitives.filter_primitive(dtypes);

    (primitives, references)
}

/// Creates a JSON property with a capitalized title.
///
/// # Arguments
/// * `name` - The name of the property.
///
/// # Returns
/// A JSON value representing the property.
fn create_property(name: &String) -> serde_json::Value {
    json!({
        "title": name,
    })
}

/// Processes a primitive attribute and adds it to the properties.
///
/// # Arguments
/// * `properties` - The properties JSON object.
/// * `attribute` - The attribute to process.
/// * `primitive` - The primitive type of the attribute.
fn process_primitive(
    properties: &mut serde_json::Value,
    attribute: &attribute::Attribute,
    primitive: &String,
) {
    let name = &attribute.name;
    properties[name] = create_property(name);

    if !attribute.docstring.is_empty() {
        properties[name]["description"] = json!(attribute.docstring);
    }

    if let Some(ref term) = attribute.term {
        properties[name]["term"] = json!(term);
    }

    set_primitive_dtype(properties, attribute, primitive);
    set_options(&mut properties[name], &attribute.options);
}

/// Sets the data type of a primitive attribute.
///
/// # Arguments
/// * `properties` - The properties JSON object.
/// * `attribute` - The attribute to process.
/// * `primitive` - The primitive type of the attribute.
fn set_primitive_dtype(
    properties: &mut serde_json::Value,
    attribute: &attribute::Attribute,
    primitive: &String,
) {
    let is_array = attribute.is_array;
    let name = &attribute.name;
    let primitives = PrimitiveTypes::new();
    let json_dtype = primitives.dtype_to_json(primitive);

    if is_array {
        properties[name]["type"] = json!("array");
        properties[name]["items"] = json!({
            "type": json_dtype
        });

        return;
    }

    properties[name]["type"] = json!(json_dtype);
}

/// Sets additional options for a JSON property.
///
/// # Arguments
/// * `property` - The property JSON object.
/// * `options` - The list of attribute options.
fn set_options(property: &mut serde_json::Value, options: &Vec<AttrOption>) {
    for option in options {
        match is_numeric(&option.value) {
            true => {
                property[option.key()] = json!(option.value().parse::<f64>().unwrap());
            }
            false => {
                property[option.key()] = json!(option.value());
            }
        }
    }
}

/// Checks if a value is numeric or a string.
///
/// # Arguments
/// * `value` - The value to check.
fn is_numeric(value: &str) -> bool {
    value.parse::<f64>().is_ok()
}

/// Processes a reference attribute and adds it to the properties.
///
/// # Arguments
/// * `properties` - The properties JSON object.
/// * `attribute` - The attribute to process.
/// * `reference` - The reference type of the attribute.
fn process_reference(
    properties: &mut serde_json::Value,
    attribute: &attribute::Attribute,
    reference: &String,
) {
    let name = &attribute.name;
    if let Some(ref term) = attribute.term {
        properties[name]["term"] = json!(term);
    }

    set_ref_dtype(properties, attribute, reference);
    set_options(&mut properties[name], &attribute.options);
}

/// Processes an enum reference attribute and adds it to the properties.
///
/// # Arguments
/// * `name` - The name of the attribute.
/// * `properties` - The properties JSON object.
/// * `enumeration` - The enumeration object.
fn process_enum_reference(name: &String, properties: &mut serde_json::Value, reference: &str) {
    properties[name] = create_property(name);
    let def_path = format!("#/{}/{}", DEFINITIONS_KEY, reference);
    properties[name]["$ref"] = json!(def_path);
}

/// Sets the data type of a reference attribute.
///
/// # Arguments
/// * `properties` - The properties JSON object.
/// * `attribute` - The attribute to process.
/// * `reference` - The reference type of the attribute.
fn set_ref_dtype(
    properties: &mut serde_json::Value,
    attribute: &attribute::Attribute,
    reference: &String,
) {
    let name = &attribute.name;
    let def_path = format!("#/{}/{}", DEFINITIONS_KEY, reference);
    if attribute.is_array {
        properties[name]["type"] = json!("array");
        properties[name]["items"] = json!({
            "$ref": json!(def_path)
        });

        return;
    }

    properties[name]["$ref"] = json!(def_path);
}
