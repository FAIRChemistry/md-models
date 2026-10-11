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

//! Provides functionality to export data models to LinkML format.
//!
//! This module contains implementations for converting internal data model representations
//! to LinkML schema format. It handles the conversion of objects, attributes, and enumerations
//! to their corresponding LinkML representations.
//!
//! The module provides several key components:
//! - Serialization of DataModel instances to LinkML YAML format
//! - Conversion implementations between internal model types and LinkML schema types
//! - Utilities for handling global slots and attribute sharing between classes
//! - Pattern constraint management through slot usage
//!
//! The conversion process preserves:
//! - Documentation and descriptions
//! - Data types and ranges
//! - Cardinality constraints
//! - Identifier flags
//! - Required/optional status
//! - URI/term mappings
//! - Enumeration values and meanings
//! - Minimum/maximum value constraints
//! - Pattern validation rules

use std::{error::Error, path::PathBuf};

use indexmap::IndexMap;

use crate::{
    attribute::Attribute,
    object::{Enumeration, Object},
    prelude::DataModel,
    tree::{self},
};

use super::schema::{
    AnyOfRange, AttributeDefinition, ClassDefinition, EnumDefinition, Example, LinkML,
    PermissibleValue, SlotUsage,
};

/// Serializes a DataModel to LinkML YAML format and writes it to a file.
///
/// This function takes a DataModel instance and converts it to LinkML schema format,
/// then serializes it to YAML. If an output path is provided, the YAML will be written
/// to that file. The function returns the serialized YAML string regardless of whether
/// it was written to a file.
///
/// # Arguments
///
/// * `model` - The DataModel to serialize
/// * `out` - Optional output path to write the YAML to
///
/// # Returns
///
/// * `Ok(String)` - The serialized YAML string
/// * `Err(Box<dyn Error>)` - If serialization or file writing fails
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
    /// This conversion process handles:
    /// - Basic schema configuration including ID, prefixes, and name
    /// - Class definitions and their attributes
    /// - Global slots (shared attributes across classes)
    /// - Enumeration definitions
    /// - Import declarations
    /// - Default type configurations
    ///
    /// The conversion maintains the hierarchical structure of the data model while
    /// adapting it to LinkML's schema format requirements.
    fn from(model: DataModel) -> Self {
        // Basic configuration
        let config = model.clone().config.unwrap_or_default();
        let id = &config.prefix;
        let prefixes: IndexMap<String, String> =
            config.prefixes.unwrap_or_default().into_iter().collect();
        let title = model
            .name
            .clone()
            .unwrap_or("Unnamed Data Model".to_string());
        let name = to_ncname(&title);

        // Classes - ensure sorting by collecting into a BTreeMap
        let mut classes: IndexMap<String, ClassDefinition> = IndexMap::from_iter(
            model
                .objects
                .iter()
                .map(|c| (c.name.clone(), c.clone().into())),
        );

        // The `Any` escape hatch is only declared when something uses it, so a
        // model without unions is unchanged.
        if has_union_attribute(&model) && !classes.contains_key(ANY_CLASS) {
            classes.insert(
                ANY_CLASS.to_string(),
                ClassDefinition {
                    description: Some(
                        "Placeholder range for slots that accept more than one type.".to_string(),
                    ),
                    class_uri: Some("linkml:Any".to_string()),
                    ..Default::default()
                },
            );
        }

        // Extract slots and update classes
        let slots = extract_slots(&model);

        classes.values_mut().for_each(|c| {
            remove_global_slots(c, &slots);
        });

        // Determine the order of classes based on dependencies
        let graph = tree::dependency_graph(&model);
        let class_order = tree::get_topological_order(&graph);

        // Set the root class
        if let Some(root) = class_order.first() {
            if let Some(class) = classes.get_mut(root) {
                class.tree_root = Some(true);
            }
        }

        // Enums
        let enums: IndexMap<String, EnumDefinition> = model
            .enums
            .iter()
            .map(|e| (e.name.clone(), e.clone().into()))
            .collect::<IndexMap<String, EnumDefinition>>();

        Self {
            id: id.clone(),
            name,
            title,
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
/// Global slots are attributes that appear in multiple classes with identical definitions.
/// This function identifies such attributes and extracts them to be defined at the schema level
/// rather than within individual classes.
///
/// The extraction process:
/// 1. Collects all attributes from all classes
/// 2. Identifies attributes that appear multiple times with identical definitions
/// 3. Returns these as global slots
///
/// # Arguments
///
/// * `model` - The data model to extract slots from
///
/// # Returns
///
/// A HashMap mapping slot names to their definitions
fn extract_slots(model: &DataModel) -> IndexMap<String, AttributeDefinition> {
    // Extract and convert attributes to a map
    let attributes: IndexMap<String, AttributeDefinition> = model
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
/// This function modifies a class definition to reference global slots instead of
/// duplicating attribute definitions. It performs the following steps:
/// 1. Identifies which of the class's attributes match global slot definitions
/// 2. Adds references to those slots in the class's slots list
/// 3. Removes the matching attributes from the class's local attributes
///
/// This process helps reduce redundancy and maintain consistency across the schema.
///
/// # Arguments
///
/// * `class` - The class definition to update
/// * `slots` - The map of global slots to reference
fn remove_global_slots(class: &mut ClassDefinition, slots: &IndexMap<String, AttributeDefinition>) {
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
    /// This conversion process handles:
    /// - Converting attributes to LinkML format
    /// - Setting up slot usage for pattern constraints
    /// - Preserving documentation and URI terms
    /// - Maintaining inheritance relationships
    /// - Managing attribute constraints and validations
    fn from(obj: Object) -> Self {
        // Create a map of attributes
        let attrib = obj
            .attributes
            .iter()
            .map(|a| (a.name.clone(), a.clone().into()))
            .collect::<IndexMap<String, AttributeDefinition>>();

        // Derive slot usage from attributes
        let mut slot_usage = IndexMap::new();
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
            class_uri: obj.term.clone(),
            slots: Vec::new(),
            is_a: None,
            mixins: obj.mixins,
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
    /// This conversion preserves:
    /// - Array/multivalued status
    /// - Data type (range)
    /// - Documentation
    /// - ID status
    /// - Required status
    /// - Minimum and maximum values
    /// - Examples
    /// - Term mappings
    fn from(attribute: Attribute) -> Self {
        let minimum_value = attribute.options.iter().find(|o| o.key() == "minimum");
        let maximum_value = attribute.options.iter().find(|o| o.key() == "maximum");
        let example = attribute
            .options
            .iter()
            .filter(|o| o.key() == "example")
            .map(|o| Example {
                value: Some(o.value()),
                description: None,
            })
            .collect::<Vec<_>>();

        // A multi-typed attribute has no single range. LinkML conjoins
        // `any_of` with the schema's `default_range`, so the union also needs
        // `range: Any` to widen the base and let the alternatives decide.
        let is_union = attribute.dtypes.len() > 1;
        let any_of = if is_union {
            attribute
                .dtypes
                .iter()
                .map(|dtype| AnyOfRange {
                    range: dtype.clone(),
                })
                .collect()
        } else {
            Vec::new()
        };

        AttributeDefinition {
            slot_uri: attribute.term,
            multivalued: Some(attribute.is_array),
            range: if is_union {
                Some(ANY_CLASS.to_string())
            } else if attribute.dtypes[0] == "string" {
                None
            } else {
                Some(attribute.dtypes[0].clone())
            },
            any_of,
            description: Some(attribute.docstring),
            identifier: Some(attribute.is_id),
            required: Some(attribute.required),
            readonly: None,
            minimum_value: minimum_value.and_then(|v| to_json_number(&v.value())),
            maximum_value: maximum_value.and_then(|v| to_json_number(&v.value())),
            recommended: None,
            examples: example,
            annotations: None,
        }
    }
}

/// Whether a value can be used where LinkML expects a URI or a CURIE.
fn is_uri_or_curie(value: &str) -> bool {
    let value = value.trim();
    if value.is_empty() || value.contains(char::is_whitespace) {
        return false;
    }

    match value.split_once(':') {
        Some((prefix, rest)) => {
            !rest.is_empty()
                && !prefix.is_empty()
                && prefix
                    .chars()
                    .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.')
        }
        None => false,
    }
}

/// Name of the LinkML class used as the range of a union-typed slot.
///
/// LinkML has no union range. The idiom is a class whose `class_uri` is
/// `linkml:Any`, which imposes no constraint of its own and lets the `any_of`
/// alternatives decide.
pub(crate) const ANY_CLASS: &str = "Any";

/// Whether any attribute in the model is multi-typed.
fn has_union_attribute(model: &DataModel) -> bool {
    model
        .objects
        .iter()
        .any(|object| object.attributes.iter().any(|a| a.dtypes.len() > 1))
}

/// Converts a model title into a valid XML NCName. LinkML constrains a
/// schema's `name` to an NCName, while `title` is free text.
fn to_ncname(name: &str) -> String {
    let mut ncname = String::new();
    let mut previous_was_separator = false;

    for character in name.chars() {
        if character.is_alphanumeric() || character == '_' || character == '-' || character == '.' {
            ncname.push(character);
            previous_was_separator = false;
        } else if !ncname.is_empty() && !previous_was_separator {
            ncname.push('_');
            previous_was_separator = true;
        }
    }

    while ncname.ends_with('_') {
        ncname.pop();
    }

    // An NCName may not start with a digit, a dot or a hyphen.
    match ncname.chars().next() {
        Some(first) if first.is_alphabetic() || first == '_' => ncname,
        Some(_) => format!("_{ncname}"),
        None => "unnamed_data_model".to_string(),
    }
}

/// Parses an option value into a JSON number, preserving integers as integers.
fn to_json_number(value: &str) -> Option<serde_json::Number> {
    if let Ok(integer) = value.parse::<i64>() {
        return Some(serde_json::Number::from(integer));
    }

    value
        .parse::<f64>()
        .ok()
        .and_then(serde_json::Number::from_f64)
}

/// Implements conversion from Enumeration to LinkML EnumDefinition.
impl From<Enumeration> for EnumDefinition {
    /// Converts an Enumeration into a LinkML EnumDefinition.
    ///
    /// This conversion process handles:
    /// - Documentation preservation
    /// - Enumeration values and their meanings
    /// - Value descriptions
    /// - Semantic mappings
    fn from(enum_: Enumeration) -> Self {
        let mut values = IndexMap::new();
        for (key, value) in enum_.mappings.iter() {
            values.insert(
                key.clone(),
                PermissibleValue {
                    text: None,
                    description: Some(value.clone()),
                    // `meaning` must be a URI or a CURIE. An enumeration whose
                    // values are plain labels — the common case — otherwise
                    // yields a schema reference tooling rejects with
                    // "… is not a valid URI or CURIE".
                    meaning: is_uri_or_curie(value).then(|| value.clone()),
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
    use pretty_assertions::assert_eq;
    use std::{collections::BTreeMap, path::PathBuf};

    use crate::option::AttrOption;

    use super::*;

    #[test]
    fn serialize_linkml_test() {
        let model = DataModel::from_markdown(&PathBuf::from("tests/data/model.md")).unwrap();
        let yaml = serde_yaml::from_str::<LinkML>(&serialize_linkml(model, None).unwrap()).unwrap();

        let expected_yaml = serde_yaml::from_str::<LinkML>(
            &std::fs::read_to_string("tests/data/expected_linkml.yml").unwrap(),
        )
        .unwrap();

        assert_eq!(yaml, expected_yaml);
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
        assert!(class_def.is_a.is_none());
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
            enum_def.permissible_values["KEY1"].description,
            Some("value1".to_string())
        );
        // `meaning` must be a URI or CURIE; a plain label is not one, and
        // emitting it produces a schema LinkML refuses to load.
        assert_eq!(enum_def.permissible_values["KEY1"].meaning, None);
    }

    /// A URI or CURIE value *is* a meaning and must be carried across.
    #[test]
    fn test_enum_uri_values_become_meanings() {
        let enum_ = Enumeration {
            mappings: BTreeMap::from([
                (
                    "GO".to_string(),
                    "https://amigo.geneontology.org/amigo/term/".to_string(),
                ),
                ("SCHEMA".to_string(), "schema:Thing".to_string()),
                ("PLAIN".to_string(), "Something other".to_string()),
            ]),
            ..Default::default()
        };

        let enum_def: EnumDefinition = enum_.into();

        assert_eq!(
            enum_def.permissible_values["GO"].meaning,
            Some("https://amigo.geneontology.org/amigo/term/".to_string())
        );
        assert_eq!(
            enum_def.permissible_values["SCHEMA"].meaning,
            Some("schema:Thing".to_string())
        );
        assert_eq!(
            enum_def.permissible_values["PLAIN"].meaning, None,
            "a label containing whitespace is not a CURIE"
        );
    }

    #[test]
    fn test_is_uri_or_curie() {
        assert!(is_uri_or_curie("https://example.org/x"));
        assert!(is_uri_or_curie("schema:Thing"));
        assert!(is_uri_or_curie("urn:uuid:1234"));
        assert!(!is_uri_or_curie("Something other"));
        assert!(!is_uri_or_curie("plain"));
        assert!(!is_uri_or_curie(""));
        assert!(!is_uri_or_curie("no colon here"));
        assert!(!is_uri_or_curie(":missingprefix"));
        assert!(!is_uri_or_curie("trailing:"));
    }

    //
    // A LinkML schema `name` must be an NCName; `title` is free text. Emitting
    // the title as the name produced schemas that reference tooling rejects.
    //

    #[test]
    fn test_to_ncname_sanitises_titles() {
        let cases = [
            ("My Test Model", "My_Test_Model"),
            ("Unnamed Data Model", "Unnamed_Data_Model"),
            ("AlreadyValid_Name-1.2", "AlreadyValid_Name-1.2"),
            ("Model (v1.0): draft!", "Model_v1.0_draft"),
            ("2024 Model", "_2024_Model"),
            ("-leading-hyphen", "_-leading-hyphen"),
            ("   ", "unnamed_data_model"),
            ("", "unnamed_data_model"),
            ("!!!", "unnamed_data_model"),
            ("Modele d'essai", "Modele_d_essai"),
        ];

        for (input, expected) in cases {
            assert_eq!(to_ncname(input), expected, "sanitising {input:?}");
        }
    }

    /// Every sanitised name must satisfy the NCName production.
    #[test]
    fn test_ncname_output_is_always_valid() {
        let inputs = [
            "My Test Model",
            "2024 Model",
            "Model (v1.0): draft!",
            "",
            "   ",
            "!!!",
            "a b  c",
        ];

        for input in inputs {
            let ncname = to_ncname(input);
            let mut chars = ncname.chars();
            let first = chars.next().expect("NCName must not be empty");

            assert!(
                first.is_alphabetic() || first == '_',
                "NCName {ncname:?} must start with a letter or underscore"
            );
            assert!(
                chars.all(|c| c.is_alphanumeric() || c == '_' || c == '-' || c == '.'),
                "NCName {ncname:?} contains an illegal character"
            );
            assert!(
                !ncname.contains(char::is_whitespace),
                "NCName {ncname:?} must not contain whitespace"
            );
        }
    }

    #[test]
    fn test_schema_name_is_ncname_and_title_is_preserved() {
        let mut model = DataModel::new(None, None);
        model.name = Some("My Test Model".to_string());

        let linkml: LinkML = model.into();

        assert_eq!(linkml.name, "My_Test_Model");
        assert_eq!(linkml.title, "My Test Model");
    }

    /// A model with no title heading falls back to a sanitised default.
    #[test]
    fn test_default_schema_name_is_ncname() {
        let model = DataModel::new(None, None);
        let linkml: LinkML = model.into();

        assert_eq!(linkml.name, "Unnamed_Data_Model");
        assert_eq!(linkml.title, "Unnamed Data Model");
    }

    //
    // Multi-typed attributes must keep their alternatives. Emitting no range at
    //

    /// Builds an attribute with the given data types.
    fn attribute_with_dtypes(dtypes: &[&str], is_array: bool) -> Attribute {
        let mut attribute = Attribute::new("value".to_string(), false);
        attribute.dtypes = dtypes.iter().map(|d| d.to_string()).collect();
        attribute.is_array = is_array;
        attribute
    }

    #[test]
    fn test_multityped_attribute_emits_any_of() {
        let definition: AttributeDefinition =
            attribute_with_dtypes(&["string", "float", "boolean"], false).into();

        assert_eq!(
            definition.range,
            Some(ANY_CLASS.to_string()),
            "a union widens its range so any_of decides"
        );
        assert_eq!(
            definition.any_of,
            vec![
                AnyOfRange {
                    range: "string".to_string()
                },
                AnyOfRange {
                    range: "float".to_string()
                },
                AnyOfRange {
                    range: "boolean".to_string()
                },
            ]
        );
    }

    #[test]
    fn test_multityped_array_keeps_multivalued() {
        let definition: AttributeDefinition =
            attribute_with_dtypes(&["string", "float"], true).into();

        assert_eq!(definition.multivalued, Some(true));
        assert_eq!(definition.any_of.len(), 2);
        assert_eq!(definition.range, Some(ANY_CLASS.to_string()));
    }

    #[test]
    fn test_multityped_with_object_reference() {
        let definition: AttributeDefinition =
            attribute_with_dtypes(&["string", "Other"], false).into();

        assert_eq!(
            definition.any_of,
            vec![
                AnyOfRange {
                    range: "string".to_string()
                },
                AnyOfRange {
                    range: "Other".to_string()
                },
            ]
        );
    }

    /// Single-typed attributes must not gain an `any_of`.
    #[test]
    fn test_single_typed_attribute_has_no_any_of() {
        let string_attr: AttributeDefinition = attribute_with_dtypes(&["string"], false).into();
        assert!(string_attr.any_of.is_empty());
        assert_eq!(string_attr.range, None, "string relies on default_range");

        let float_attr: AttributeDefinition = attribute_with_dtypes(&["float"], false).into();
        assert!(float_attr.any_of.is_empty());
        assert_eq!(float_attr.range, Some("float".to_string()));
    }

    //
    // Numeric bounds
    //

    /// A fractional bound on a float attribute must be representable.
    #[test]
    fn test_fractional_bounds_do_not_panic() {
        let mut attribute = attribute_with_dtypes(&["float"], false);
        attribute.options.push(AttrOption::MinimumValue(0.5));
        attribute.options.push(AttrOption::MaximumValue(99.9));

        let definition: AttributeDefinition = attribute.into();

        assert_eq!(
            definition.minimum_value.map(|n| n.to_string()),
            Some("0.5".to_string())
        );
        assert_eq!(
            definition.maximum_value.map(|n| n.to_string()),
            Some("99.9".to_string())
        );
    }

    /// Whole numbers must stay integers rather than becoming `0.0`.
    #[test]
    fn test_integral_bounds_stay_integral() {
        let mut attribute = attribute_with_dtypes(&["integer"], false);
        attribute.options.push(AttrOption::MinimumValue(0.0));
        attribute.options.push(AttrOption::MaximumValue(100.0));

        let definition: AttributeDefinition = attribute.into();

        assert_eq!(
            definition.minimum_value.map(|n| n.to_string()),
            Some("0".to_string())
        );
        assert_eq!(
            definition.maximum_value.map(|n| n.to_string()),
            Some("100".to_string())
        );
    }

    #[test]
    fn test_negative_bounds_are_preserved() {
        let mut attribute = attribute_with_dtypes(&["float"], false);
        attribute.options.push(AttrOption::MinimumValue(-273.15));

        let definition: AttributeDefinition = attribute.into();

        assert_eq!(
            definition.minimum_value.map(|n| n.to_string()),
            Some("-273.15".to_string())
        );
    }
}
