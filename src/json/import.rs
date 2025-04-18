/*
 * Copyright (c) 2025 Jan Range, Felix Neubauer
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

//! JSON Schema import functionality for converting JSON Schema objects to DataModel structures.
//!
//! This module provides the implementation for converting JSON Schema objects to the internal
//! data model representation. It handles the conversion of schema objects, properties, and enumerations
//! to their corresponding data model types.

use std::collections::{BTreeMap, HashMap, HashSet};

use crate::{
    attribute::Attribute,
    object::{Enumeration, Object},
    option::AttrOption,
    prelude::DataModel,
};

use super::schema::{EnumObject, PrimitiveType, Property, SchemaObject, SchemaType};

/// Types that should be ignored when processing data types
/// "object" and "array" are container types and not actual data types
const IGNORE_TYPES: [&str; 2] = ["object", "array"];

/// Converts a JSON Schema object to a DataModel
///
/// This implementation handles the conversion of the root schema object and all its definitions
/// to the corresponding DataModel structure, including objects and enumerations.
impl TryFrom<SchemaObject> for DataModel {
    type Error = Box<dyn std::error::Error>;

    fn try_from(schema_obj: SchemaObject) -> Result<Self, Self::Error> {
        let mut objects = vec![schema_obj.clone().try_into()?];
        let mut enums = vec![];

        // Process all definitions in the schema
        for (name, definition) in schema_obj.definitions {
            match definition {
                SchemaType::Object(object) => {
                    let mut object: Object = object.try_into()?;
                    object.name = name;
                    objects.push(object);
                }
                SchemaType::Enum(enum_obj) => {
                    let mut enum_obj: Enumeration = enum_obj.try_into()?;
                    enum_obj.name = name;
                    enums.push(enum_obj);
                }
            }
        }

        Ok(DataModel {
            objects,
            enums,
            name: Some(schema_obj.title),
            ..Default::default()
        })
    }
}

/// Converts a JSON Schema object to an Object
///
/// This implementation handles the conversion of a schema object's properties
/// to attributes, and processes required fields.
impl TryFrom<SchemaObject> for Object {
    type Error = Box<dyn std::error::Error>;

    fn try_from(schema_obj: SchemaObject) -> Result<Self, Self::Error> {
        // Convert all properties to attributes
        let mut attributes = schema_obj
            .properties
            .into_iter()
            .map(|(name, property)| {
                let mut attribute: Attribute = property.try_into()?;
                attribute.name = name.clone();
                Ok(attribute)
            })
            .collect::<Result<Vec<Attribute>, Self::Error>>()?;

        // Mark required attributes
        for required_attribute in schema_obj.required {
            let attribute = attributes
                .iter_mut()
                .find(|attr| attr.name == required_attribute);
            if let Some(attr) = attribute {
                attr.required = true;
            }
        }

        Ok(Object {
            name: schema_obj.title,
            attributes,
            docstring: schema_obj.description.unwrap_or_default(),
            term: None,
            parent: None,
            position: None,
        })
    }
}

/// Converts a JSON Schema property to an Attribute
///
/// This implementation handles various property types including:
/// - Array properties with items
/// - Properties with direct data types
/// - Properties with references
/// - Properties with oneOf (multiple possible types)
impl TryFrom<Property> for Attribute {
    type Error = Box<dyn std::error::Error>;

    fn try_from(property: Property) -> Result<Self, Self::Error> {
        let is_array = property
            .dtype
            .as_ref()
            .is_some_and(|dtype| dtype.is_array());

        let mut dtypes = HashSet::new();

        // Handle array items or direct data type
        if is_array {
            // If the property is an array, we need to handle the items
            // which can be a reference, a oneOf, or a data type. We will
            // ignore the dtype in this case.
            if let Some(items) = &property.items {
                dtypes.extend(
                    items
                        .get_types()
                        .into_iter()
                        .map(extract_reference)
                        .collect::<Result<Vec<String>, String>>()?,
                );
            }
        } else if let Some(dtype) = &property.dtype {
            // If the property is not an array, we can just add the dtype
            dtypes.insert(extract_reference(dtype.to_string())?);
        }

        // Add reference if present
        if let Some(reference) = &property.reference {
            dtypes.insert(extract_reference(reference.clone())?);
        }

        // Process oneOf items
        if let Some(one_of) = property.one_of {
            for item in one_of.iter() {
                dtypes.extend(
                    item.get_types()
                        .into_iter()
                        .map(extract_reference)
                        .collect::<Result<Vec<String>, String>>()?,
                );
            }
        }

        Ok(Attribute {
            name: property.title.unwrap_or("MISSING_TITLE".to_string()),
            is_array,
            dtypes: dtypes
                .into_iter()
                .filter(|dtype| !IGNORE_TYPES.contains(&dtype.as_str()))
                .collect::<Vec<String>>(),
            is_id: false,
            docstring: property.description.unwrap_or_default(),
            options: parse_options(&property.options)?,
            term: property.term,
            required: false,
            // TODO: Implement default
            default: None,
            xml: None,
            is_enum: false,
            position: None,
            import_prefix: None,
        })
    }
}

/// Converts a JSON Schema enum object to an Enumeration
///
/// This implementation handles the conversion of enum values to mappings,
/// escaping invalid keys as needed.
impl TryFrom<EnumObject> for Enumeration {
    type Error = Box<dyn std::error::Error>;

    fn try_from(enum_obj: EnumObject) -> Result<Self, Self::Error> {
        let mappings = enum_obj
            .enum_values
            .iter()
            .enumerate()
            .map(|(i, value)| {
                if is_valid_key(value) {
                    // If there are no special characters, we can use the value as is
                    (value.clone(), value.clone())
                } else {
                    // If there are special characters, we need to escape them
                    (format!("VALUE_{i}"), value.clone())
                }
            })
            .collect::<BTreeMap<String, String>>();

        Ok(Enumeration {
            name: enum_obj.title,
            docstring: enum_obj.description.unwrap_or_default(),
            position: None,
            mappings,
        })
    }
}

/// Parses JSON Schema options into AttrOption objects
///
/// This function converts the key-value pairs from the JSON Schema options
/// into AttrOption objects that can be used in the data model.
fn parse_options(
    options: &HashMap<String, PrimitiveType>,
) -> Result<Vec<AttrOption>, Box<dyn std::error::Error>> {
    let mut parsed_options = Vec::new();

    for (key, value) in options {
        let option = AttrOption::from_pair(key, value.to_string().as_str())?;
        parsed_options.push(option);
    }

    Ok(parsed_options)
}

/// Extracts the reference name from a JSON Schema reference string
///
/// This function takes a reference string (e.g., "#/$defs/Test") and extracts
/// the actual type name (e.g., "Test").
fn extract_reference(reference: String) -> Result<String, String> {
    reference
        .split('/')
        .last()
        .filter(|s| !s.is_empty())
        .map(ToString::to_string)
        .ok_or_else(|| "Invalid reference format".to_string())
}

/// Checks if a string is a valid identifier key
///
/// A valid key must:
/// - Not be empty
/// - Start with a letter or underscore
/// - Contain only alphanumeric characters or underscores
fn is_valid_key(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    // Check if the first character is a letter or underscore
    let first_char = s.chars().next().unwrap();
    if !first_char.is_alphabetic() && first_char != '_' {
        return false;
    }

    // Check if all other characters are alphanumeric or underscore
    s.chars().all(|c| c.is_alphanumeric() || c == '_')
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    /// Tests the parsing of a complete JSON Schema into a DataModel
    ///
    /// This test verifies that:
    /// - The schema is correctly parsed into a DataModel
    /// - Objects and enums are correctly extracted
    /// - Attributes are correctly parsed with their properties
    /// - Required fields are marked as such
    /// - Array types are correctly handled
    /// - References are correctly resolved
    /// - OneOf types are correctly handled
    #[test]
    fn test_parse_schema() {
        let schema = json!({
          "$schema": "https://json-schema.org/draft/2020-12/schema",
          "$id": "https://www.github.com/my/repo/",
          "title": "Test",
          "type": "object",
          "properties": {
            "array_valued": {
              "title": "array_valued",
              "type": "array",
              "$term": "http://schema.org/something",
              "items": {
                "$ref": "#/$defs/Test2"
              }
            },
            "multiple_types": {
              "title": "multiple_types",
              "oneOf": [
                {
                  "type": "number"
                },
                {
                  "$ref": "#/$defs/Test2"
                }
              ]
            },
            "multiple_types_array": {
              "title": "multiple_types_array",
              "type": "array",
              "items": {
                "oneOf": [
                  {
                    "type": "number"
                  },
                  {
                    "$ref": "#/$defs/Test2"
                  }
                ]
              }
            },
            "name": {
              "title": "name",
              "type": "string",
              "default": "test",
              "description": "A test description",
              "$term": "http://schema.org/hello"
            },
            "number": {
              "title": "number",
              "type": "number",
              "$term": "http://schema.org/one",
              "minimum": 0.0
            },
            "ontology": {
              "title": "ontology",
              "$ref": "#/$defs/Ontology"
            },
            "single_valued": {
              "title": "single_valued",
              "type": "object",
              "$ref": "#/$defs/Test2"
            }
          },
          "$defs": {
            "Ontology": {
              "title": "Ontology",
              "type": "string",
              "enum": [
                "https://www.evidenceontology.org/term/",
                "https://amigo.geneontology.org/amigo/term/",
                "http://semanticscience.org/resource/"
              ]
            },
            "Test2": {
              "title": "Test2",
              "type": "object",
              "properties": {
                "names": {
                  "title": "names",
                  "type": "array",
                  "$term": "http://schema.org/hello",
                  "items": {
                    "type": "string"
                  }
                },
                "number": {
                  "title": "number",
                  "type": "number",
                  "$term": "http://schema.org/one",
                  "minimum": 0.0
                }
              },
              "required": [],
              "additionalProperties": false
            },
            "no_title_and_no_required": {
                "type": "object",
                "properties": {
                    "val": {
                        "type": "string"
                    }
                }
            }
          },
          "required": [
            "name"
          ],
          "additionalProperties": false
        });

        let schema: SchemaObject = serde_json::from_value(schema).expect("Failed to parse schema");
        let data_model =
            DataModel::try_from(schema).expect("Failed to convert schema to data model");

        assert_eq!(data_model.name, Some("Test".to_string()));
        assert_eq!(data_model.objects.len(), 3);
        assert_eq!(data_model.enums.len(), 1);

        // Test root object (Test)
        let root = data_model
            .objects
            .iter()
            .find(|object| object.name == "Test")
            .expect("Root object not found");

        assert_eq!(root.attributes.len(), 7);
        assert_eq!(root.attributes[0].name, "array_valued");
        assert_eq!(root.attributes[1].name, "multiple_types");
        assert_eq!(root.attributes[2].name, "multiple_types_array");
        assert_eq!(root.attributes[3].name, "name");
        assert_eq!(root.attributes[4].name, "number");
        assert_eq!(root.attributes[5].name, "ontology");
        assert_eq!(root.attributes[6].name, "single_valued");

        // Test Test2 object
        let test2 = data_model
            .objects
            .iter()
            .find(|object| object.name == "Test2")
            .expect("Test2 object not found");

        assert_eq!(test2.attributes.len(), 2);
        assert_eq!(test2.attributes[0].name, "names");
        assert_eq!(test2.attributes[1].name, "number");

        // Verify Test2 attributes in detail
        let names_attr = &test2.attributes[0];
        assert!(names_attr.is_array);
        assert_eq!(names_attr.dtypes, vec!["string"]);
        assert_eq!(names_attr.term, Some("http://schema.org/hello".to_string()));

        let number_attr = &test2.attributes[1];
        assert!(!number_attr.is_array);
        assert_eq!(number_attr.dtypes, vec!["number"]);
        assert_eq!(number_attr.term, Some("http://schema.org/one".to_string()));

        // Verify no_title_and_no_required object
        let no_title_and_no_required = data_model
            .objects
            .iter()
            .find(|object| object.name == "no_title_and_no_required")
            .expect("no_title_and_no_required object not found");

        assert!(!no_title_and_no_required.name.is_empty());
        assert_eq!(no_title_and_no_required.attributes.len(), 1);
        assert_eq!(no_title_and_no_required.attributes[0].name, "val");

        // Test Ontology enum
        let ontology = data_model
            .enums
            .iter()
            .find(|e| e.name == "Ontology")
            .expect("Ontology enum not found");

        assert_eq!(ontology.mappings.len(), 3);
        assert_eq!(
            ontology.mappings["VALUE_0"],
            "https://www.evidenceontology.org/term/"
        );
        assert_eq!(
            ontology.mappings["VALUE_1"],
            "https://amigo.geneontology.org/amigo/term/"
        );
        assert_eq!(
            ontology.mappings["VALUE_2"],
            "http://semanticscience.org/resource/"
        );

        // Verify root object attribute details
        let array_valued = &root.attributes[0];
        assert!(array_valued.is_array);
        assert_eq!(array_valued.dtypes, vec!["Test2"]);
        assert_eq!(
            array_valued.term,
            Some("http://schema.org/something".to_string())
        );

        let multiple_types = &root.attributes[1];
        assert!(!multiple_types.is_array);
        let multiple_types_dtypes: HashSet<_> = multiple_types.dtypes.iter().collect();
        assert_eq!(
            multiple_types_dtypes,
            HashSet::from([&"number".to_string(), &"Test2".to_string()])
        );

        let name_attr = &root.attributes[3];
        assert!(name_attr.required);
        assert_eq!(name_attr.term, Some("http://schema.org/hello".to_string()));
    }

    /// Tests the parsing of a simple property into an Attribute
    ///
    /// This test verifies that a property with a simple type (number)
    /// is correctly converted to an Attribute with the right properties.
    #[test]
    fn test_parse_property() {
        let property = json!({
          "title": "number",
          "type": "number",
          "$term": "http://schema.org/one",
          "minimum": 0.0,
          "description": "test"
        });

        let property: Property = serde_json::from_value(property).unwrap();
        let attribute = Attribute::try_from(property).unwrap();
        assert_eq!(attribute.name, "number");
        assert_eq!(attribute.dtypes, vec!["number"]);
        assert_eq!(attribute.docstring, "test");
        assert_eq!(attribute.term, Some("http://schema.org/one".to_string()));
        assert!(!attribute.required);
        assert_eq!(attribute.default, None);
        assert!(!attribute.is_array);
        assert_eq!(attribute.xml, None);
        assert!(!attribute.is_enum);
        assert_eq!(attribute.position, None);
        assert_eq!(attribute.import_prefix, None);
    }

    /// Tests the parsing of a property with oneOf (multiple types)
    ///
    /// This test verifies that a property with multiple possible types
    /// is correctly converted to an Attribute with all types included.
    #[test]
    fn test_parse_property_with_one_of() {
        let property = json!({
            "title": "number",
            "oneOf": [
                {
                    "type": "number"
                },
                {
                    "type": "string"
                }
            ]
        });

        let property: Property = serde_json::from_value(property).unwrap();
        let attribute = Attribute::try_from(property).unwrap();

        assert_eq!(attribute.name, "number");
        assert_eq!(
            attribute.dtypes.into_iter().collect::<HashSet<_>>(),
            vec!["number".to_string(), "string".to_string()]
                .into_iter()
                .collect::<HashSet<_>>()
        );
        assert_eq!(attribute.docstring, "");
        assert_eq!(attribute.term, None);
        assert!(!attribute.required);
        assert_eq!(attribute.default, None);
        assert_eq!(attribute.xml, None);
        assert!(!attribute.is_array);
        assert!(!attribute.is_enum);
        assert_eq!(attribute.position, None);
        assert_eq!(attribute.import_prefix, None);
    }

    /// Tests the parsing of a property with a reference
    ///
    /// This test verifies that a property with a reference to another type
    /// is correctly converted to an Attribute with the referenced type.
    #[test]
    fn test_parse_property_with_reference() {
        let property = json!({
            "title": "number",
            "$ref": "#/$defs/Test"
        });

        let property: Property = serde_json::from_value(property).unwrap();
        let attribute = Attribute::try_from(property).unwrap();

        assert_eq!(attribute.name, "number");
        assert_eq!(attribute.dtypes, vec!["Test".to_string()]);
        assert_eq!(attribute.docstring, "");
        assert_eq!(attribute.term, None);
        assert!(!attribute.required);
        assert_eq!(attribute.default, None);
        assert!(!attribute.is_array);
        assert_eq!(attribute.xml, None);
        assert!(!attribute.is_enum);
        assert_eq!(attribute.position, None);
        assert_eq!(attribute.import_prefix, None);
    }

    /// Tests the parsing of an array property without a reference
    ///
    /// This test verifies that an array property with a simple type
    /// is correctly converted to an Attribute with is_array=true.
    #[test]
    fn test_parse_property_array_without_reference() {
        let property = json!({
            "title": "number",
            "type": "array",
            "items": {
                "type": "string"
            }
        });

        let property: Property = serde_json::from_value(property).unwrap();
        let attribute = Attribute::try_from(property).unwrap();

        assert_eq!(attribute.name, "number");
        assert_eq!(attribute.dtypes, vec!["string".to_string()]);
        assert_eq!(attribute.docstring, "");
        assert_eq!(attribute.term, None);
        assert!(!attribute.required);
        assert_eq!(attribute.default, None);
        assert!(attribute.is_array);
        assert_eq!(attribute.xml, None);
        assert!(!attribute.is_enum);
        assert_eq!(attribute.position, None);
        assert_eq!(attribute.import_prefix, None);
    }

    /// Tests extracting a reference from an array property
    ///
    /// This test verifies that the type information is correctly extracted
    /// from an array property with items of a specific type.
    #[test]
    fn test_extract_reference_from_array() {
        let property = json!({
            "title": "number",
            "type": "array",
            "items": {
                "type": "string"
            }
        });

        let property: Property = serde_json::from_value(property).unwrap();
        let attribute = Attribute::try_from(property).unwrap();

        assert_eq!(attribute.name, "number");
        assert_eq!(attribute.dtypes, vec!["string".to_string()]);
        assert_eq!(attribute.docstring, "");
        assert_eq!(attribute.term, None);
        assert!(!attribute.required);
        assert_eq!(attribute.default, None);
        assert!(attribute.is_array);
        assert_eq!(attribute.xml, None);
        assert!(!attribute.is_enum);
        assert_eq!(attribute.position, None);
        assert_eq!(attribute.import_prefix, None);
    }

    /// Tests extracting references from a oneOf property
    ///
    /// This test verifies that all type information is correctly extracted
    /// from a property with oneOf containing both a reference and a simple type.
    #[test]
    fn test_extract_reference_from_one_of() {
        let property = json!({
            "title": "number",
            "oneOf": [
                {
                    "$ref": "#/$defs/Test"
                },
                {
                    "type": "string"
                }
            ]
        });

        let property: Property = serde_json::from_value(property).unwrap();
        let attribute = Attribute::try_from(property).unwrap();

        assert_eq!(attribute.name, "number");
        assert_eq!(
            attribute.dtypes.into_iter().collect::<HashSet<_>>(),
            vec!["Test".to_string(), "string".to_string()]
                .into_iter()
                .collect::<HashSet<_>>()
        );
    }

    /// Tests parsing a schema object into an Object
    ///
    /// This test verifies that a schema object with properties is correctly
    /// converted to an Object with attributes, and required fields are marked.
    #[test]
    fn test_parse_object() {
        let object = json!({
            "title": "Test",
            "type": "object",
            "properties": {
                "number": {
                    "title": "number",
                    "type": "number"
                },
                "string": {
                    "name": "string",
                    "type": "string"
                }
            },
            "required": ["number"]
        });

        let object: SchemaObject = serde_json::from_value(object).unwrap();
        let data_model = Object::try_from(object).unwrap();

        assert_eq!(data_model.name, "Test");
        assert_eq!(data_model.attributes.len(), 2);
        assert_eq!(data_model.attributes[0].name, "number");
        assert_eq!(data_model.attributes[1].name, "string");

        let attribute1 = data_model.attributes[0].clone();

        assert_eq!(attribute1.name, "number");
        assert_eq!(attribute1.dtypes, vec!["number"]);
        assert_eq!(attribute1.docstring, "");
        assert_eq!(attribute1.term, None);
        assert!(attribute1.required);
        assert_eq!(attribute1.default, None);
        assert!(!attribute1.is_array);

        let attribute2 = data_model.attributes[1].clone();

        assert_eq!(attribute2.name, "string");
        assert_eq!(attribute2.dtypes, vec!["string"]);
        assert_eq!(attribute2.docstring, "");
        assert_eq!(attribute2.term, None);
        assert!(!attribute2.required);
        assert_eq!(attribute2.default, None);
        assert!(!attribute2.is_array);
    }

    /// Tests parsing an enum object into an Enumeration
    ///
    /// This test verifies that an enum object with simple values
    /// is correctly converted to an Enumeration with mappings.
    #[test]
    fn test_parse_enum() {
        let enum_obj = json!({
            "title": "Test",
            "type": "string",
            "enum": ["value1", "value2", "value3"]
        });

        let enum_obj: EnumObject = serde_json::from_value(enum_obj).unwrap();
        let enumeration = Enumeration::try_from(enum_obj).unwrap();

        assert_eq!(enumeration.name, "Test");
        assert_eq!(enumeration.mappings.len(), 3);
        assert_eq!(enumeration.mappings["value1"], "value1");
        assert_eq!(enumeration.mappings["value2"], "value2");
        assert_eq!(enumeration.mappings["value3"], "value3");
    }

    /// Tests parsing an enum object with special characters
    ///
    /// This test verifies that an enum object with values containing special characters
    /// is correctly converted to an Enumeration with escaped mappings.
    #[test]
    fn test_parse_enum_with_special_characters() {
        let enum_obj = json!({
            "title": "Test",
            "type": "string",
            "enum": ["https://www.evidenceontology.org/term/", "https://amigo.geneontology.org/amigo/term/", "http://semanticscience.org/resource/"]
        });

        let enum_obj: EnumObject = serde_json::from_value(enum_obj).unwrap();
        let enumeration = Enumeration::try_from(enum_obj).unwrap();

        assert_eq!(enumeration.name, "Test");
        assert_eq!(enumeration.mappings.len(), 3);
        assert_eq!(
            enumeration.mappings["VALUE_0"],
            "https://www.evidenceontology.org/term/"
        );
        assert_eq!(
            enumeration.mappings["VALUE_1"],
            "https://amigo.geneontology.org/amigo/term/"
        );
        assert_eq!(
            enumeration.mappings["VALUE_2"],
            "http://semanticscience.org/resource/"
        );
    }

    /// Tests the extract_reference function
    ///
    /// This test verifies that the extract_reference function correctly
    /// extracts type names from reference strings and handles edge cases.
    #[test]
    fn test_extract_reference() {
        assert_eq!(
            extract_reference("#/$defs/Test".to_string()),
            Ok("Test".to_string())
        );
        assert_eq!(
            extract_reference("Test".to_string()),
            Ok("Test".to_string())
        );
        assert_eq!(
            extract_reference("".to_string()),
            Err("Invalid reference format".to_string())
        );
    }
}
