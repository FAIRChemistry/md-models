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

//! LinkML schema definitions for parsing and representing LinkML schemas in Rust.
//!
//! This module provides Rust structs that map directly to the YAML structure of LinkML schemas.
//! Each struct is annotated with serde derive macros to enable serialization/deserialization.

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

/// The root schema object that contains all LinkML schema definitions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LinkML {
    /// Unique identifier for the schema
    pub id: String,
    /// Name of the schema
    pub name: String,
    /// Title of the schema
    pub title: String,
    /// Optional description of the schema's purpose
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "remove_newlines"
    )]
    pub description: Option<String>,
    /// Optional license identifier
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
    /// List of related resources and references
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub see_also: Vec<String>,
    /// Mapping of prefix strings to their expanded URI forms
    #[serde(skip_serializing_if = "IndexMap::is_empty")]
    pub prefixes: IndexMap<String, String>,
    /// Default prefix to use when none is specified
    pub default_prefix: String,
    /// Default range type for slots
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_range: Option<String>,
    /// List of imported schemas
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub imports: Vec<String>,
    /// Map of class definitions
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub classes: IndexMap<String, ClassDefinition>,
    /// Map of slot definitions
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub slots: IndexMap<String, AttributeDefinition>,
    /// Map of enum definitions
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub enums: IndexMap<String, EnumDefinition>,
}

/// Represents a contributor to the schema
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Contributor {
    /// Name of the contributor
    pub name: String,
}

/// Defines a subset of schema elements
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Subset {
    /// Optional description of the subset's purpose
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "remove_newlines"
    )]
    pub description: Option<String>,
}

/// Defines a custom data type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TypeDefinition {
    /// Base type that this type extends
    pub base: String,
    /// Optional regex pattern for validation
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,
    /// Optional description of the type
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "remove_newlines"
    )]
    pub description: Option<String>,
    /// Optional minimum value for numeric types
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub minimum_value: Option<i64>,
}

/// Defines an enumeration type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EnumDefinition {
    /// Optional description of the enum
    #[serde(
        default,
        skip_serializing_if = "is_empty_string_option",
        deserialize_with = "remove_newlines"
    )]
    pub description: Option<String>,
    /// Map of allowed values and their definitions
    #[serde(skip_serializing_if = "IndexMap::is_empty")]
    pub permissible_values: IndexMap<String, PermissibleValue>,
}

/// Represents a single permissible value in an enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PermissibleValue {
    /// Human-readable text for the value
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Optional URI defining the value's meaning
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub meaning: Option<String>,
    /// Optional description of the value
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "remove_newlines"
    )]
    pub description: Option<String>,
}

/// Defines a class in the schema
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClassDefinition {
    /// Optional description of the class
    #[serde(
        default,
        skip_serializing_if = "is_empty_string_option",
        deserialize_with = "remove_newlines"
    )]
    pub description: Option<String>,
    /// Optional URI identifying the class
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub class_uri: Option<String>,
    /// List of slots that this class can have
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub slots: Vec<String>,
    /// Optional parent class name
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_a: Option<String>,
    /// Whether this class is a tree root
    #[serde(default, skip_serializing_if = "is_false_option")]
    pub tree_root: Option<bool>,
    /// Map of slot usage definitions
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slot_usage: Option<IndexMap<String, SlotUsage>>,
    /// Map of attributes
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attributes: Option<IndexMap<String, AttributeDefinition>>,
    /// Mixed in class
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mixins: Vec<String>,
}

/// Represents an annotation on a schema element
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Annotation {
    /// The annotation value
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

/// Defines how a slot is used in a specific context
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SlotUsage {
    /// Optional pattern for validation
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,
}

/// Defines a slot (property/field) in the schema
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AttributeDefinition {
    /// Optional description of the slot
    #[serde(
        default,
        skip_serializing_if = "is_empty_string_option",
        deserialize_with = "remove_newlines"
    )]
    pub description: Option<String>,
    /// Semantic type of the slot
    #[serde(default, skip_serializing_if = "is_empty_string_option")]
    pub slot_uri: Option<String>,
    /// Whether this slot serves as an identifier
    #[serde(default, skip_serializing_if = "is_false_option")]
    pub identifier: Option<bool>,
    /// Whether this slot is required
    #[serde(default, skip_serializing_if = "is_false_option")]
    pub required: Option<bool>,
    /// Optional type range for the slot
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub range: Option<String>,
    /// Whether this slot is read-only
    #[serde(default, skip_serializing_if = "is_false_option")]
    pub readonly: Option<bool>,
    /// Whether this slot can have multiple values
    #[serde(default, skip_serializing_if = "is_false_option")]
    pub multivalued: Option<bool>,
    /// Optional minimum value for numeric slots
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub minimum_value: Option<i64>,
    /// Optional maximum value for numeric slots
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub maximum_value: Option<i64>,
    /// Whether this slot is recommended
    #[serde(default, skip_serializing_if = "is_false_option")]
    pub recommended: Option<bool>,
    /// Optional map of example values
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub examples: Vec<Example>,
    /// Optional map of annotations
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub annotations: Option<IndexMap<String, Annotation>>,
}

/// Represents an example value for a slot
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Example {
    /// The example value
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    /// Optional description of the example
    #[serde(
        default,
        skip_serializing_if = "is_empty_string_option",
        deserialize_with = "remove_newlines"
    )]
    pub description: Option<String>,
}

// Helper functions
fn remove_newlines<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let opt = Option::<String>::deserialize(deserializer)?;
    Ok(opt.map(|s| {
        s.lines()
            .map(|line| line.trim())
            .collect::<Vec<&str>>()
            .join(" ")
    }))
}

fn is_empty_string_option(s: &Option<String>) -> bool {
    s.is_none() || s.as_ref().unwrap().is_empty()
}

fn is_false_option(s: &Option<bool>) -> bool {
    s.is_none() || !s.as_ref().unwrap()
}
