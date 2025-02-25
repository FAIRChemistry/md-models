//! This file contains Rust struct definitions with serde serialization.
//!
//! WARNING: This is an auto-generated file.
//! Do not edit directly - any changes will be overwritten.

use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use derive_builder::Builder;

//
// Type definitions
//
/// Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do
/// eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut
/// enim ad minim veniam, quis nostrud exercitation ullamco laboris
/// nisi ut aliquip ex ea commodo consequat.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Builder)]
#[allow(non_snake_case)]
pub struct Test {
    /// The name of the test. This is a unique identifier that helps track
    /// individual test cases across the system. It should be
    /// descriptive and follow the standard naming conventions.
    pub name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub number: Option<TestNumberType>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub test2: Option<Vec<Test2>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub ontology: Option<Ontology>,

}


#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Builder)]
#[allow(non_snake_case)]
pub struct Test2 {

    #[serde(skip_serializing_if = "Option::is_none")]
    pub names: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub number: Option<f64>,

}


//
// Enum definitions
//

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum Ontology {
    #[serde(rename = "https://www.evidenceontology.org/term/")]
    ECO,
    #[serde(rename = "https://amigo.geneontology.org/amigo/term/")]
    GO,
    #[serde(rename = "http://semanticscience.org/resource/")]
    SIO,
}

/// Union type for Test.number
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum TestNumberType {
    F64(f64),
    String(String),
}