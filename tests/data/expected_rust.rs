//! This file contains Rust struct definitions with serde serialization.
//!
//! WARNING: This is an auto-generated file.
//! Do not edit directly - any changes will be overwritten.

use derive_builder::Builder;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

//
// Type definitions
//
/// Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do
/// eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut
/// enim ad minim veniam, quis nostrud exercitation ullamco laboris
/// nisi ut aliquip ex ea commodo consequat.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Builder, Default)]
#[allow(non_snake_case)]
pub struct Test {
    /// The name of the test. This is a unique identifier that helps track
    /// individual test cases across the system. It should be
    /// descriptive and follow the standard naming conventions.
    #[serde(default)]
    #[builder(default = "2.0.to_string().into()", setter(into))]
    pub name: String,

    /// number
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(into))]
    pub number: Option<TestNumberType>,

    /// test2
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[builder(default, setter(into, each(name = "to_test2")))]
    pub test2: Vec<Test2>,

    /// ontology
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(into))]
    pub ontology: Option<Ontology>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Builder, Default)]
#[allow(non_snake_case)]
pub struct Test2 {
    /// names
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[builder(default, setter(into, each(name = "to_names")))]
    pub names: Vec<String>,

    /// number
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(into))]
    pub number: Option<f64>,
}

//
// Enum definitions
//
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default, PartialEq, Eq)]
pub enum Ontology {
    #[default]
    #[serde(rename = "https://www.evidenceontology.org/term/")]
    Eco,

    #[serde(rename = "https://amigo.geneontology.org/amigo/term/")]
    Go,

    #[serde(rename = "http://semanticscience.org/resource/")]
    Sio,
}

/// Union type for Test.number
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum TestNumberType {
    F64(f64),
    String(String),
}