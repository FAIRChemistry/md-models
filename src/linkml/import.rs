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

//! Provides functionality to import LinkML schemas into internal data model format.
//!
//! This module contains implementations for converting LinkML schema format into internal
//! data model representations. It handles the conversion of classes, slots, and enumerations
//! from their LinkML representations.
//!
//! # Key Components
//!
//! - `deserialize_linkml`: Main entry point for importing LinkML YAML files
//! - `From<LinkML> for DataModel`: Core conversion from LinkML schema to internal model
//! - `From` implementations for converting individual LinkML components:
//!   - `ClassDefinition` -> `Object`
//!   - `AttributeDefinition` -> `Attribute`
//!   - `EnumDefinition` -> `Enumeration`
use std::{collections::BTreeMap, error::Error, path::PathBuf};

use crate::{
    attribute::Attribute,
    markdown::frontmatter::FrontMatter,
    object::{Enumeration, Object},
    option::AttrOption,
    prelude::DataModel,
};

use super::schema::{AttributeDefinition, ClassDefinition, EnumDefinition, LinkML};

/// Deserializes a LinkML YAML file into a DataModel.
///
/// This function reads a LinkML schema from a YAML file and converts it into the internal
/// DataModel representation. The conversion preserves all relevant schema information including
/// classes, attributes, enumerations, and metadata.
///
/// # Arguments
///
/// * `path` - Path to the LinkML YAML file to import
///
/// # Returns
///
/// * `Ok(DataModel)` - Successfully parsed and converted data model
/// * `Err(Box<dyn Error>)` - Error during file reading or YAML parsing
pub fn deserialize_linkml(path: &PathBuf) -> Result<DataModel, Box<dyn Error>> {
    let yaml = std::fs::read_to_string(path)?;
    let linkml: LinkML = serde_yaml::from_str(&yaml)?;
    Ok(DataModel::from(linkml))
}

/// Implements conversion from LinkML schema to DataModel.
impl From<LinkML> for DataModel {
    /// Converts a LinkML schema into the internal DataModel format.
    ///
    /// This conversion handles:
    /// - Schema metadata through FrontMatter configuration
    /// - Classes and their attributes
    /// - Global slots/attributes
    /// - Enumerations
    ///
    /// The conversion preserves:
    /// - Prefixes and namespace information
    /// - Class hierarchies and relationships
    /// - Attribute definitions and constraints
    /// - Enumeration values and meanings
    fn from(linkml: LinkML) -> Self {
        // Create config from LinkML metadata
        let config = FrontMatter {
            prefix: linkml.id,
            prefixes: Some(linkml.prefixes.into_iter().collect()),
            ..Default::default()
        };

        // Convert classes to objects, merging in global slots
        let mut objects = Vec::new();
        for (name, class) in linkml.classes {
            let mut obj = Object::from(class.clone());
            obj.name = name;

            // Add global slots to object attributes
            for slot_name in class.slots {
                if let Some(slot_def) = linkml.slots.get(&slot_name) {
                    let mut attr = Attribute::from(slot_def.clone());
                    attr.name = slot_name;
                    obj.attributes.push(attr);
                }
            }
            objects.push(obj);
        }

        // Convert enums
        let enums = linkml
            .enums
            .into_iter()
            .map(|(name, def)| {
                let mut enum_ = Enumeration::from(def);
                enum_.name = name;
                enum_
            })
            .collect();

        DataModel {
            name: Some(linkml.name),
            config: Some(config),
            objects,
            enums,
        }
    }
}

/// Implements conversion from LinkML ClassDefinition to Object.
impl From<ClassDefinition> for Object {
    /// Converts a LinkML ClassDefinition into an internal Object representation.
    ///
    /// This conversion handles:
    /// - Class metadata (name, description, URI)
    /// - Local attribute definitions
    /// - Slot usage patterns and constraints
    ///
    /// # Arguments
    ///
    /// * `class` - The LinkML ClassDefinition to convert
    ///
    /// # Returns
    ///
    /// An Object representing the class in the internal model format
    fn from(class: ClassDefinition) -> Self {
        let mut attributes = Vec::new();

        // Convert local attributes
        if let Some(attrs) = class.attributes {
            for (name, def) in attrs {
                let mut attr = Attribute::from(def);
                attr.name = name;
                attributes.push(attr);
            }
        }

        // Add pattern constraints from slot usage
        if let Some(slot_usage) = class.slot_usage {
            for (name, usage) in slot_usage {
                if let Some(pattern) = usage.pattern {
                    if let Some(attr) = attributes.iter_mut().find(|a| a.name == name) {
                        attr.options.push(AttrOption::Pattern(pattern));
                    }
                }
            }
        }

        Object {
            name: class.is_a.unwrap_or_default(),
            docstring: class.description.unwrap_or_default(),
            term: class.class_uri,
            attributes,
            parent: None,
            position: None,
        }
    }
}

/// Implements conversion from LinkML AttributeDefinition to Attribute.
impl From<AttributeDefinition> for Attribute {
    /// Converts a LinkML AttributeDefinition into an internal Attribute representation.
    ///
    /// This conversion preserves:
    /// - Documentation
    /// - Data type/range
    /// - Cardinality (multivalued status)
    /// - Identifier status
    /// - Required status
    /// - URI/term mapping
    ///
    /// # Arguments
    ///
    /// * `attr` - The LinkML AttributeDefinition to convert
    ///
    /// # Returns
    ///
    /// An Attribute representing the slot in the internal model format
    fn from(attr: AttributeDefinition) -> Self {
        Attribute {
            name: String::new(), // Set later when context is available
            docstring: attr.description.unwrap_or_default(),
            dtypes: vec![attr.range.unwrap_or_else(|| "string".to_string())],
            term: attr.slot_uri,
            is_array: attr.multivalued.unwrap_or(false),
            is_id: attr.identifier.unwrap_or(false),
            required: attr.required.unwrap_or(false),
            options: Vec::new(), // Patterns added later from slot_usage
            default: None,
            is_enum: false,
            position: None,
            xml: None,
            import_prefix: None,
        }
    }
}

/// Implements conversion from LinkML EnumDefinition to Enumeration.
impl From<EnumDefinition> for Enumeration {
    /// Converts a LinkML EnumDefinition into an internal Enumeration representation.
    ///
    /// This conversion preserves:
    /// - Documentation
    /// - Enumeration values and their meanings
    /// - Value mappings
    ///
    /// # Arguments
    ///
    /// * `enum_def` - The LinkML EnumDefinition to convert
    ///
    /// # Returns
    ///
    /// An Enumeration representing the enum in the internal model format
    fn from(enum_def: EnumDefinition) -> Self {
        let mappings = enum_def
            .permissible_values
            .into_iter()
            .map(|(key, value)| (key, value.meaning.unwrap_or_default()))
            .collect::<BTreeMap<String, String>>();

        Enumeration {
            name: String::new(), // Set later when context is available
            docstring: enum_def.description.unwrap_or_default(),
            mappings,
            position: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn deserialize_linkml_test() {
        let model = deserialize_linkml(&PathBuf::from("tests/data/expected_linkml.yml")).unwrap();
        let expected_model =
            DataModel::from_markdown(&PathBuf::from("tests/data/model.md")).unwrap();

        assert_eq!(
            model.objects.len(),
            expected_model.objects.len(),
            "Objects length mismatch"
        );
        assert_eq!(
            model.enums.len(),
            expected_model.enums.len(),
            "Enums length mismatch"
        );

        for obj in model.objects.iter() {
            let other_obj = expected_model
                .objects
                .iter()
                .find(|o| o.name == obj.name)
                .unwrap_or_else(|| panic!("Object {} not found", obj.name));
            assert_eq!(obj.name, other_obj.name, "Object name mismatch");
            assert_eq!(
                obj.docstring, other_obj.docstring,
                "Object docstring mismatch"
            );
            assert_eq!(obj.term, other_obj.term, "Object term mismatch");
            assert_eq!(
                obj.attributes.len(),
                other_obj.attributes.len(),
                "Attributes length mismatch"
            );

            for attr in obj.attributes.iter() {
                let other_attr = other_obj
                    .attributes
                    .iter()
                    .find(|a| a.name == attr.name)
                    .unwrap_or_else(|| panic!("Attribute {} not found", attr.name));
                assert_eq!(attr.name, other_attr.name, "Attribute name mismatch");
            }
        }

        for enum_ in model.enums.iter() {
            let other_enum = expected_model
                .enums
                .iter()
                .find(|e| e.name == enum_.name)
                .unwrap_or_else(|| panic!("Enum {} not found", enum_.name));
            assert_eq!(enum_.name, other_enum.name, "Enum name mismatch");
            assert_eq!(
                enum_.docstring, other_enum.docstring,
                "Enum docstring mismatch"
            );
            assert_eq!(
                enum_.mappings, other_enum.mappings,
                "Enum mappings mismatch"
            );
        }
    }

    // Add more specific tests for each conversion implementation
}
