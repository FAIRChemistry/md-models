//! This file contains Rust struct definitions with serde serialization.
//!
//! WARNING: This is an auto-generated file.
//! Do not edit directly - any changes will be overwritten.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// JSON-LD base types
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct JsonLdContext(pub HashMap<String, serde_json::Value>);

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct JsonLd {
    #[serde(rename = "@context", skip_serializing_if = "Option::is_none")]
    pub context: Option<JsonLdContext>,
    #[serde(rename = "@id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(rename = "@type", skip_serializing_if = "Option::is_none")]
    pub type_: Option<String>,
}

//
// Model Type definitions
//
/// Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do
/// eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut
/// enim ad minim veniam, quis nostrud exercitation ullamco laboris
/// nisi ut aliquip ex ea commodo consequat.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Test {
    #[serde(flatten)]
    pub json_ld: JsonLd,
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

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Test2 {
    #[serde(flatten)]
    pub json_ld: JsonLd,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub names: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number: Option<f64>,
}

//
// Model Enum definitions
//

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Ontology {
    #[serde(rename = "https://www.evidenceontology.org/term/")]
    ECO,
    #[serde(rename = "https://amigo.geneontology.org/amigo/term/")]
    GO,
    #[serde(rename = "http://semanticscience.org/resource/")]
    SIO,
}


//
// Enum definitions for attributes with multiple types
//

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestNumberType {
    Float(f64),
    String(String),
}