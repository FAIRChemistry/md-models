use crate::attribute;
use crate::attribute::AttrOption;
use crate::datamodel::DataModel;
use crate::object::{self, Enumeration};
use crate::primitives::PrimitiveTypes;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashSet;

static DEFINITIONS_KEY: &str = "definitions";
static SCHEMA_VERSION: &str = "http://json-schema.org/draft-07/schema";

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

    for reference in used_refs {
        let sub_obj = objects.iter().find(|o| o.name == reference).unwrap();
        let (properties, _) = process_class(sub_obj, model);

        schema[DEFINITIONS_KEY][reference] = properties;
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
) -> (serde_json::Value, HashSet<String>) {
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
                let enumeration = model.enums.iter().find(|e| e.name == reference).unwrap();
                process_enum_reference(&attribute.name, &mut schema["properties"], enumeration);
            } else if object_names.contains(&reference) {
                all_refs.insert(reference.clone());
                process_reference(&mut schema["properties"], attribute, &reference);
            } else {
                panic!("Reference {} not found in the markdown file", reference);
            }
        }
    }

    (schema, all_refs)
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
fn is_numeric(value: &String) -> bool {
    match value.parse::<f64>() {
        Ok(_) => true,
        Err(_) => false,
    }
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
    set_options(properties, &attribute.options);
}

/// Processes an enum reference attribute and adds it to the properties.
///
/// # Arguments
/// * `name` - The name of the attribute.
/// * `properties` - The properties JSON object.
/// * `enumeration` - The enumeration object.
fn process_enum_reference(
    name: &String,
    properties: &mut serde_json::Value,
    enumeration: &Enumeration,
) {
    properties[name] = create_property(name);
    properties[name]["type"] = json!("string");
    properties[name]["enum"] = json!(enumeration
        .mappings
        .values()
        .map(|v| v.to_string())
        .collect::<Vec<String>>());
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
