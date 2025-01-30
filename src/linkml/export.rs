//! Provides functionality to export data models to LinkML format.
//!
//! This module contains implementations for converting internal data model representations
//! to LinkML schema format. It handles the conversion of objects, attributes, and enumerations
//! to their corresponding LinkML representations.

use std::{collections::HashMap, error::Error, path::PathBuf};

use crate::{
    attribute::Attribute,
    object::{Enumeration, Object},
    prelude::DataModel,
};

use super::schema::{
    AttributeDefinition, ClassDefinition, EnumDefinition, LinkML, PermissibleValue, SlotUsage,
};

/// Serializes a DataModel to LinkML YAML format and writes it to a file.
///
/// This function takes a DataModel instance and converts it to LinkML schema format,
/// then serializes it to YAML and writes the output to the specified file path.
///
/// # Arguments
///
/// * `model` - The DataModel to serialize
/// * `out` - The output path to write the YAML to, if provided.
///
/// # Returns
///
/// * `Ok(yaml)` if serialization and file writing succeed
/// * `Err(Box<dyn Error>)` if YAML serialization fails or if file writing fails
pub fn serialize_linkml(model: DataModel, out: Option<&PathBuf>) -> Result<String, Box<dyn Error>> {
    let linkml = LinkML::from(model);
    let yaml = serde_yaml::to_string(&linkml)?;
    if let Some(out) = out {
        std::fs::write(out, &yaml)?;
    }

    Ok(yaml)
}

/// Implements conversion from DataModel to LinkML schema format.
impl From<DataModel> for LinkML {
    /// Converts a DataModel instance into a LinkML schema.
    ///
    /// This conversion handles:
    /// - Basic configuration (id, prefixes, name)
    /// - Classes and their attributes
    /// - Global slots (shared attributes)
    /// - Enumerations
    ///
    /// # Arguments
    ///
    /// * `model` - The DataModel to convert
    ///
    /// # Returns
    ///
    /// A LinkML schema representing the data model
    fn from(model: DataModel) -> Self {
        // Basic configuration
        let config = model.clone().config.unwrap_or_default();
        let id = &config.prefix;
        let prefixes = &config.prefixes.unwrap_or_default();
        let name = model
            .name
            .clone()
            .unwrap_or("Unnamed Data Model".to_string());

        // Classes
        let mut classes: HashMap<String, ClassDefinition> = model
            .objects
            .iter()
            .map(|c| (c.name.clone(), c.clone().into()))
            .collect::<HashMap<String, ClassDefinition>>();

        // Extract slots and update classes
        let slots = extract_slots(&model);

        classes.values_mut().for_each(|c| {
            remove_global_slots(c, &slots);
        });

        // Enums
        let enums: HashMap<String, EnumDefinition> = model
            .enums
            .iter()
            .map(|e| (e.name.clone(), e.clone().into()))
            .collect::<HashMap<String, EnumDefinition>>();

        Self {
            id: id.clone(),
            name: name.clone(),
            title: name,
            description: None,
            license: None,
            see_also: Vec::new(),
            prefixes: prefixes.clone(),
            default_prefix: id.clone(),
            default_range: Some("string".to_string()),
            imports: vec!["linkml:types".to_string()],
            classes,
            slots,
            enums,
        }
    }
}

/// Extracts global slots (shared attributes) from a data model.
///
/// Global slots are attributes that appear in multiple classes with the same definition.
///
/// # Arguments
///
/// * `model` - The data model to extract slots from
///
/// # Returns
///
/// A HashMap mapping slot names to their definitions
fn extract_slots(model: &DataModel) -> HashMap<String, AttributeDefinition> {
    // Extract and convert attributes to a map
    let attributes: HashMap<String, AttributeDefinition> = model
        .objects
        .iter()
        .flat_map(|o| o.attributes.iter())
        .map(|a| (a.name.clone(), a.clone().into()))
        .collect();

    // Filter out non-duplicate attributes (global slots)
    attributes
        .clone()
        .into_iter()
        .filter(
            // Check if the attribute is defined in more than one class
            |(name_a, def_a)| {
                attributes
                    .iter()
                    .filter(|(name_b, def_b)| name_a == *name_b && def_a == *def_b)
                    .count()
                    > 1
            },
        )
        .collect()
}

/// Updates a class definition to use global slots where appropriate.
///
/// This function:
/// 1. Identifies which of the class's attributes are global slots
/// 2. Adds references to those slots in the class's slots list
/// 3. Removes those attributes from the class's local attributes
///
/// # Arguments
///
/// * `class` - The class definition to update
/// * `slots` - The map of global slots
fn remove_global_slots(class: &mut ClassDefinition, slots: &HashMap<String, AttributeDefinition>) {
    // Get the class's attributes
    let class_attrs = class.attributes.clone().unwrap_or_default();

    // Fill slots with globally defined duplicate attributes that exist in this class
    class.slots = class_attrs
        .keys()
        .filter(|name| slots.contains_key(*name))
        .cloned()
        .collect();

    // Keep only non-duplicate attributes in the class
    class.attributes = Some(
        class_attrs
            .iter()
            .filter(|(name, _)| !slots.contains_key(*name))
            .map(|(name, def)| (name.clone(), def.clone()))
            .collect(),
    );
}

/// Implements conversion from Object to LinkML ClassDefinition.
impl From<Object> for ClassDefinition {
    /// Converts an Object into a LinkML ClassDefinition.
    ///
    /// This handles:
    /// - Converting attributes to LinkML format
    /// - Setting up slot usage for pattern constraints
    /// - Preserving documentation and URI terms
    ///
    /// # Arguments
    ///
    /// * `obj` - The Object to convert
    ///
    /// # Returns
    ///
    /// A LinkML ClassDefinition
    fn from(obj: Object) -> Self {
        // Create a map of attributes
        let attrib = obj
            .attributes
            .iter()
            .map(|a| (a.name.clone(), a.clone().into()))
            .collect::<HashMap<String, AttributeDefinition>>();

        // Derive slot usage from attributes
        let mut slot_usage = HashMap::new();
        for attr in obj.attributes.iter() {
            let pattern_option = attr.options.iter().find(|o| o.key() == "pattern");
            if let Some(pattern) = pattern_option {
                slot_usage.insert(
                    attr.name.clone(),
                    SlotUsage {
                        pattern: Some(pattern.value().to_string()),
                    },
                );
            }
        }

        ClassDefinition {
            description: Some(obj.docstring),
            class_uri: obj.term,
            slots: Vec::new(),
            is_a: Some(obj.name),
            tree_root: None,
            attributes: Some(attrib),
            slot_usage: if slot_usage.is_empty() {
                None
            } else {
                Some(slot_usage)
            },
        }
    }
}

/// Implements conversion from Attribute to LinkML AttributeDefinition.
impl From<Attribute> for AttributeDefinition {
    /// Converts an Attribute into a LinkML AttributeDefinition.
    ///
    /// This preserves:
    /// - Array/multivalued status
    /// - Data type (range)
    /// - Documentation
    /// - ID status
    /// - Required status
    ///
    /// # Arguments
    ///
    /// * `attribute` - The Attribute to convert
    ///
    /// # Returns
    ///
    /// A LinkML AttributeDefinition
    fn from(attribute: Attribute) -> Self {
        AttributeDefinition {
            multivalued: Some(attribute.is_array),
            range: if attribute.dtypes[0] == "string" {
                None
            } else {
                Some(attribute.dtypes[0].clone())
            },
            description: Some(attribute.docstring),
            identifier: Some(attribute.is_id),
            required: Some(attribute.required),
            readonly: None,
            minimum_value: None,
            maximum_value: None,
            recommended: None,
            examples: None,
            annotations: None,
        }
    }
}

/// Implements conversion from Enumeration to LinkML EnumDefinition.
impl From<Enumeration> for EnumDefinition {
    /// Converts an Enumeration into a LinkML EnumDefinition.
    ///
    /// This preserves:
    /// - Documentation
    /// - Enumeration values and their meanings
    ///
    /// # Arguments
    ///
    /// * `enum_` - The Enumeration to convert
    ///
    /// # Returns
    ///
    /// A LinkML EnumDefinition
    fn from(enum_: Enumeration) -> Self {
        let mut values = HashMap::new();
        for (key, value) in enum_.mappings.iter() {
            values.insert(
                key.clone(),
                PermissibleValue {
                    text: None,
                    description: Some(value.clone()),
                    meaning: Some(value.clone()),
                },
            );
        }
        EnumDefinition {
            description: Some(enum_.docstring),
            permissible_values: values,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, path::PathBuf};

    use crate::option::AttrOption;

    use super::*;

    #[test]
    fn serialize_linkml_test() {
        let model = DataModel::from_markdown(&PathBuf::from("tests/data/model.md")).unwrap();
        let yaml = serialize_linkml(model, None).unwrap();

        let expected_yaml = serde_yaml::from_str::<LinkML>(
            &std::fs::read_to_string("tests/data/expected_linkml.yml").unwrap(),
        )
        .unwrap();
        let yaml_yaml = serde_yaml::from_str::<LinkML>(&yaml).unwrap();
        assert_eq!(yaml_yaml, expected_yaml);
    }

    #[test]
    #[allow(clippy::field_reassign_with_default)]
    fn test_class_definition_conversion() {
        let mut obj = Object::default();
        obj.name = "TestClass".to_string();
        obj.docstring = "Test description".to_string();
        obj.term = Some("http://example.org/TestClass".to_string());

        let mut attr = Attribute::default();
        attr.name = "test_attr".to_string();
        attr.options = vec![AttrOption::Pattern("^test.*$".to_string())];
        attr.dtypes = vec!["string".to_string()];
        obj.attributes = vec![attr];

        let class_def: ClassDefinition = obj.into();
        assert_eq!(class_def.description, Some("Test description".to_string()));
        assert_eq!(
            class_def.class_uri,
            Some("http://example.org/TestClass".to_string())
        );
        assert_eq!(class_def.is_a, Some("TestClass".to_string()));
        assert!(class_def.slot_usage.is_some());
    }

    #[test]
    #[allow(clippy::field_reassign_with_default)]
    fn test_attribute_definition_conversion() {
        let mut attr = Attribute::default();
        attr.is_array = true;
        attr.dtypes = vec!["integer".to_string()];
        attr.docstring = "Test attribute".to_string();
        attr.is_id = true;
        attr.required = true;

        let attr_def: AttributeDefinition = attr.into();
        assert_eq!(attr_def.multivalued, Some(true));
        assert_eq!(attr_def.range, Some("integer".to_string()));
        assert_eq!(attr_def.description, Some("Test attribute".to_string()));
        assert_eq!(attr_def.identifier, Some(true));
        assert_eq!(attr_def.required, Some(true));
    }

    #[test]
    #[allow(clippy::field_reassign_with_default)]
    fn test_enum_definition_conversion() {
        let mut enum_ = Enumeration::default();
        enum_.docstring = "Test enum".to_string();
        enum_.mappings = BTreeMap::from([
            ("KEY1".to_string(), "value1".to_string()),
            ("KEY2".to_string(), "value2".to_string()),
        ]);

        let enum_def: EnumDefinition = enum_.into();
        assert_eq!(enum_def.description, Some("Test enum".to_string()));
        assert_eq!(enum_def.permissible_values.len(), 2);
        assert!(enum_def.permissible_values.contains_key("KEY1"));
        assert_eq!(
            enum_def.permissible_values["KEY1"].meaning,
            Some("value1".to_string())
        );
    }
}
