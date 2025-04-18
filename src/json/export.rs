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

use std::{
    collections::{BTreeMap, HashMap, HashSet},
    str::FromStr,
};

use crate::{
    attribute::{self, Attribute},
    datamodel::DataModel,
    markdown::frontmatter::FrontMatter,
    object::{Enumeration, Object},
    option::AttrOption,
    validation::BASIC_TYPES,
};

use super::schema::{self, PrimitiveType};

const SCHEMA: &str = "https://json-schema.org/draft/2020-12/schema";

/// Converts a `DataModel` into a JSON schema representation.
///
/// # Arguments
///
/// * `model` - A reference to the `DataModel` to be converted.
/// * `root` - The root object name in the model.
/// * `openai` - A boolean flag indicating whether to use the OpenAI schema.
///
/// # Returns
///
/// A `Result` containing the `SchemaObject` or an error message.
pub fn to_json_schema(
    model: &DataModel,
    root: &str,
    openai: bool,
) -> Result<schema::SchemaObject, String> {
    let root_object = retrieve_object(model, root)?;

    let mut schema_object = schema::SchemaObject::try_from(root_object)?;
    let mut used_types = HashSet::new();
    let mut used_enums = HashSet::new();

    collect_definitions(root_object, model, &mut used_types, &mut used_enums)?;

    let definitions = collect_definitions_from_model(model, &used_types, &used_enums)?;

    schema_object.schema = Some(SCHEMA.to_string());
    schema_object.definitions = definitions;

    if let Some(config) = model.config.clone() {
        post_process_schema(&mut schema_object, &config, openai);
    }

    Ok(schema_object)
}

/// Retrieves an object from the `DataModel` by name.
///
/// # Arguments
///
/// * `model` - A reference to the `DataModel`.
/// * `name` - The name of the object to retrieve.
///
/// # Returns
///
/// A `Result` containing a reference to the `Object` or an error message.
fn retrieve_object<'a>(model: &'a DataModel, name: &'a str) -> Result<&'a Object, String> {
    model
        .objects
        .iter()
        .find(|obj| obj.name == name)
        .ok_or(format!("Object {} not found", name))
}

/// Retrieves an enumeration from the `DataModel` by name.
///
/// # Arguments
///
/// * `model` - A reference to the `DataModel`.
/// * `name` - The name of the enumeration to retrieve.
///
/// # Returns
///
/// A `Result` containing a reference to the `EnumObject` or an error message.
fn retrieve_enum<'a>(model: &'a DataModel, name: &'a str) -> Result<&'a Enumeration, String> {
    model
        .enums
        .iter()
        .find(|e| e.name == name)
        .ok_or(format!("Enum {} not found", name))
}

/// Collects definitions from the `DataModel` based on used types and enums.
///
/// # Arguments
///
/// * `model` - A reference to the `DataModel`.
/// * `used_types` - A reference to a set of used type names.
/// * `used_enums` - A reference to a set of used enum names.
///
/// # Returns
///
/// A `Result` containing a `BTreeMap` of schema definitions or an error message.
fn collect_definitions_from_model(
    model: &DataModel,
    used_types: &HashSet<String>,
    used_enums: &HashSet<String>,
) -> Result<BTreeMap<String, schema::SchemaType>, String> {
    let mut definitions = BTreeMap::new();

    for obj_name in used_types {
        let obj = retrieve_object(model, obj_name)?;
        definitions.insert(obj_name.clone(), schema::SchemaType::try_from(obj)?);
    }

    for enum_name in used_enums {
        let enum_object = retrieve_enum(model, enum_name)?;
        definitions.insert(
            enum_name.clone(),
            schema::SchemaType::try_from(enum_object)?,
        );
    }

    Ok(definitions)
}

/// Collects definitions from an object and updates the used types and enums sets.
///
/// # Arguments
///
/// * `object` - A reference to the `Object`.
/// * `model` - A reference to the `DataModel`.
/// * `used_types` - A mutable reference to a set of used type names.
/// * `used_enums` - A mutable reference to a set of used enum names.
///
/// # Returns
///
/// A `Result` indicating success or an error message.
fn collect_definitions(
    object: &Object,
    model: &DataModel,
    used_types: &mut HashSet<String>,
    used_enums: &mut HashSet<String>,
) -> Result<(), String> {
    for attr in object.attributes.iter() {
        for dtype in attr.dtypes.iter() {
            if BASIC_TYPES.contains(&dtype.as_str()) || used_types.contains(dtype) {
                continue;
            }

            let object = model.objects.iter().find(|obj| obj.name == *dtype);
            let enumeration = model.enums.iter().find(|e| e.name == *dtype);

            if let Some(object) = object {
                used_types.insert(dtype.clone());
                collect_definitions(object, model, used_types, used_enums)?;
            } else if let Some(enumeration) = enumeration {
                used_enums.insert(enumeration.name.clone());
            } else {
                return Err(format!("Object or enumeration {} not found", dtype));
            }
        }
    }

    Ok(())
}

/// Resolves prefixes in the schema properties using the provided prefixes map.
///
/// # Arguments
///
/// * `schema` - A mutable reference to the `SchemaObject`.
/// * `prefixes` - A reference to a map containing prefix-to-URI mappings.
fn resolve_prefixes(schema: &mut schema::SchemaObject, prefixes: &HashMap<String, String>) {
    for (_, property) in schema.properties.iter_mut() {
        if let Some(reference) = property.term.clone() {
            let (prefix, term) = reference.split_once(":").unwrap_or(("", ""));
            if let Some(prefix) = prefixes.get(prefix) {
                property.term = Some(format!("{}{}", prefix, term));
            }
        }
    }
}

/// Removes options from the schema properties.
///
/// # Arguments
///
/// * `schema` - A mutable reference to the `SchemaObject`.
fn remove_options(schema: &mut schema::SchemaObject) {
    for (_, property) in schema.properties.iter_mut() {
        property.options = HashMap::new();
    }
}

/// Post-processes the schema object by setting its ID, resolving prefixes, and optionally removing options.
///
/// # Arguments
///
/// * `schema_object` - A mutable reference to the `SchemaObject` to be post-processed.
/// * `config` - A reference to the `FrontMatter` configuration containing repository and prefix information.
/// * `no_options` - A boolean flag indicating whether to remove options from the schema properties.
fn post_process_schema(
    schema_object: &mut schema::SchemaObject,
    config: &FrontMatter,
    openai: bool,
) {
    schema_object.id = Some(config.repo.clone());
    post_process_object(schema_object, config, openai);

    for (_, definition) in schema_object.definitions.iter_mut() {
        if let schema::SchemaType::Object(definition) = definition {
            post_process_object(definition, config, openai);
        }
    }
}

fn post_process_object(object: &mut schema::SchemaObject, config: &FrontMatter, openai: bool) {
    if let Some(prefixes) = &config.prefixes {
        resolve_prefixes(object, prefixes);
    }
    if openai {
        remove_options(object);
    }
}

impl TryFrom<&Enumeration> for schema::SchemaType {
    type Error = String;

    /// Attempts to convert an `Enumeration` into a `SchemaType`.
    ///
    /// # Arguments
    ///
    /// * `enumeration` - A reference to the `Enumeration`.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `SchemaType` or an error message.
    fn try_from(enumeration: &Enumeration) -> Result<Self, Self::Error> {
        Ok(schema::SchemaType::Enum(schema::EnumObject::try_from(
            enumeration,
        )?))
    }
}

impl TryFrom<&Object> for schema::SchemaType {
    type Error = String;

    /// Attempts to convert an `Object` into a `SchemaType`.
    ///
    /// # Arguments
    ///
    /// * `obj` - A reference to the `Object`.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `SchemaType` or an error message.
    fn try_from(obj: &Object) -> Result<Self, Self::Error> {
        Ok(schema::SchemaType::Object(schema::SchemaObject::try_from(
            obj,
        )?))
    }
}

impl TryFrom<&Object> for schema::SchemaObject {
    type Error = String;

    /// Attempts to convert an `Object` into a `SchemaObject`.
    ///
    /// # Arguments
    ///
    /// * `obj` - A reference to the `Object`.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `SchemaObject` or an error message.
    fn try_from(obj: &Object) -> Result<Self, Self::Error> {
        let properties: Result<BTreeMap<String, schema::Property>, String> = obj
            .attributes
            .iter()
            .map(|attr| -> Result<(String, schema::Property), String> {
                Ok((attr.name.clone(), schema::Property::try_from(attr)?))
            })
            .collect();

        let required: Vec<String> = obj
            .attributes
            .iter()
            .filter(|attr| attr.required)
            .map(|attr| attr.name.clone())
            .collect();

        Ok(schema::SchemaObject {
            title: obj.name.clone(),
            dtype: schema::DataType::Object,
            description: Some(obj.docstring.clone()),
            properties: properties?,
            definitions: BTreeMap::new(),
            required,
            schema: None,
            id: None,
            additional_properties: false,
        })
    }
}

impl TryFrom<&Enumeration> for schema::EnumObject {
    type Error = String;

    /// Attempts to convert an `Enumeration` into an `EnumObject`.
    ///
    /// # Arguments
    ///
    /// * `enumeration` - A reference to the `Enumeration`.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `EnumObject` or an error message.
    fn try_from(enumeration: &Enumeration) -> Result<Self, Self::Error> {
        let values = enumeration
            .mappings
            .values()
            .cloned()
            .collect::<Vec<String>>();

        Ok(schema::EnumObject {
            title: enumeration.name.clone(),
            dtype: schema::DataType::String,
            description: Some(enumeration.docstring.clone()),
            enum_values: values,
        })
    }
}

impl TryFrom<&Attribute> for schema::Property {
    type Error = String;

    /// Attempts to convert an `Attribute` into a `Property`.
    ///
    /// # Arguments
    ///
    /// * `attr` - A reference to the `Attribute`.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `Property` or an error message.
    fn try_from(attr: &Attribute) -> Result<Self, Self::Error> {
        let mut dtype = (!attr.is_enum)
            .then(|| schema::DataType::try_from(attr))
            .transpose()?;

        let options: HashMap<String, PrimitiveType> = attr
            .options
            .iter()
            .map(|o| -> Result<(String, PrimitiveType), String> {
                Ok((o.key().to_string(), o.try_into()?))
            })
            .collect::<Result<HashMap<String, PrimitiveType>, String>>()?;

        let reference: Option<String> =
            if attr.is_enum || matches!(dtype, Some(schema::DataType::Object)) {
                Some(format!("#/$defs/{}", attr.dtypes[0]))
            } else {
                None
            };

        let items: Option<schema::Item> = attr.into();
        let one_of = (!attr.is_array).then(|| attr.into());
        let description = (!attr.docstring.is_empty()).then(|| attr.docstring.clone());
        let enum_values = if attr.is_enum { Some(Vec::new()) } else { None };

        if attr.dtypes.len() > 1 && !attr.is_array {
            // If there are multiple types, we need to use the AnyOf case
            dtype = None;
        }

        // Make sure that the default matches the datatype
        let default: Option<PrimitiveType> = if let Some(default) = attr.default.clone() {
            process_default(default, &dtype)
        } else {
            None
        };

        Ok(schema::Property {
            title: attr.name.clone(),
            dtype,
            default,
            description,
            term: attr.term.clone(),
            reference,
            options,
            one_of,
            items,
            enum_values,
        })
    }
}

/// Processes the default value of an attribute.
///
/// # Arguments
///
/// * `default` - A reference to the default value of the attribute.
/// * `dtype` - A reference to the data type of the attribute.
///
/// # Returns
///
/// A `Result` containing the processed default value or an error message.
fn process_default(
    default: attribute::DataType,
    dtype: &Option<schema::DataType>,
) -> Option<PrimitiveType> {
    if matches!(dtype, Some(schema::DataType::String)) {
        default
            .as_string()
            .map(|d| PrimitiveType::String(d.trim_matches('"').to_string()))
    } else {
        Some(default.into())
    }
}

impl TryFrom<&Attribute> for schema::DataType {
    type Error = String;

    /// Attempts to convert an `Attribute` into a `DataType`.
    ///
    /// # Arguments
    ///
    /// * `attr` - A reference to the `Attribute`.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `DataType` or an error message.
    ///
    /// # Errors
    ///
    /// Returns an error if the `dtypes` vector in the attribute is empty.
    fn try_from(attr: &Attribute) -> Result<Self, Self::Error> {
        if attr.is_array {
            return Ok(schema::DataType::Array);
        }

        schema::DataType::try_from(
            attr.dtypes
                .first()
                .ok_or(format!("No data types found for attribute: {}", attr.name))?,
        )
    }
}

/// Specific case for the `items` field in the JSON schema.
impl From<&Attribute> for Option<schema::Item> {
    /// Converts an `Attribute` into an `Option<Item>`.
    ///
    /// # Arguments
    ///
    /// * `attr` - A reference to the `Attribute`.
    ///
    /// # Returns
    ///
    /// An `Option<Item>` representing the attribute's items.
    fn from(attr: &Attribute) -> Self {
        if !attr.is_array {
            // No need for 'items' when the attr is not
            // an array type
            return None;
        }

        // Check if it is an OneOf case
        let one_of: Vec<schema::Item> = attr.into();

        if one_of.is_empty() {
            // There is just a single type
            Some(process_dtype(&attr.dtypes[0]))
        } else {
            Some(schema::Item::OneOfItem(schema::OneOfItemType { one_of }))
        }
    }
}

impl From<&Attribute> for Vec<schema::Item> {
    /// Converts an `Attribute` into a `Vec<Item>`.
    ///
    /// # Arguments
    ///
    /// * `attr` - A reference to the `Attribute`.
    ///
    /// # Returns
    ///
    /// A `Vec<Item>` representing the attribute's items.
    fn from(attr: &Attribute) -> Self {
        if attr.dtypes.len() == 1 {
            return Vec::new();
        }

        let mut items = Vec::new();
        for dtype in attr.dtypes.iter() {
            items.push(process_dtype(dtype));
        }

        items
    }
}

/// Processes a data type string and returns an `Item`.
///
/// # Arguments
///
/// * `dtype` - A reference to the data type string.
///
/// # Returns
///
/// An `Item` representing the data type.
fn process_dtype(dtype: &str) -> schema::Item {
    match schema::DataType::from_str(dtype) {
        Ok(basic_type) => {
            schema::Item::DataTypeItem(schema::DataTypeItemType { dtype: basic_type })
        }
        Err(_) => schema::Item::ReferenceItem(schema::ReferenceItemType {
            reference: format!("#/$defs/{}", dtype),
        }),
    }
}

impl TryFrom<&AttrOption> for PrimitiveType {
    type Error = String;

    fn try_from(option: &AttrOption) -> Result<Self, Self::Error> {
        let value = option.value();

        // Try parsing in order: f64, boolean, i64, string
        if let Ok(float_val) = value.parse::<f64>() {
            return Ok(PrimitiveType::Number(float_val));
        }

        if let Ok(bool_val) = value.parse::<bool>() {
            return Ok(PrimitiveType::Boolean(bool_val));
        }

        if let Ok(int_val) = value.parse::<i64>() {
            return Ok(PrimitiveType::Integer(int_val));
        }

        // If all other parses fail, treat as string
        Ok(PrimitiveType::String(value))
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{json, Value};

    use super::*;
    use crate::attribute::Attribute;

    #[test]
    fn test_attribute_with_multiple_types() {
        let attr = Attribute {
            name: "test_attribute".to_string(),
            is_array: false,
            is_id: false,
            dtypes: vec!["string".to_string(), "RefType".to_string()],
            docstring: "".to_string(),
            options: vec![],
            term: None,
            required: false,
            default: None,
            xml: None,
            is_enum: false,
            position: None,
            import_prefix: None,
        };

        let property: schema::Property =
            schema::Property::try_from(&attr).expect("Failed to convert Attribute to Property");

        let serialized_property =
            serde_json::to_value(&property).expect("Failed to serialize Property to JSON");

        let expected_json = json!({
            "title": "test_attribute",
            "oneOf": [
                {"type": "string"},
                {"$ref": "#/$defs/RefType"},
            ]
        });

        assert_eq!(serialized_property, expected_json);
    }

    #[test]
    fn test_array_attribute() {
        let attr = Attribute {
            name: "test_attribute".to_string(),
            is_array: true,
            is_id: false,
            dtypes: vec!["string".to_string(), "RefType".to_string()],
            docstring: "".to_string(),
            options: vec![],
            term: None,
            required: false,
            default: None,
            xml: None,
            is_enum: false,
            position: None,
            import_prefix: None,
        };

        let property: schema::Property =
            schema::Property::try_from(&attr).expect("Failed to convert Attribute to Property");
        let serialized_property: Value =
            serde_json::to_value(&property).expect("Failed to serialize Property to JSON");

        let expected_json = json!({
            "title": "test_attribute",
            "type": "array",
            "items": {
                "oneOf": [
                    {"type": "string"},
                    {"$ref": "#/$defs/RefType"}
                ]
            }
        });

        assert_eq!(serialized_property, expected_json);
    }
}
