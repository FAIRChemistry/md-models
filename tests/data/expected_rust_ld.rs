//! This file contains Rust struct definitions with serde serialization.
//!
//! WARNING: This is an auto-generated file.
//! Do not edit directly - any changes will be overwritten.

use derive_builder::Builder;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use derivative::Derivative;
use std::collections::HashMap;
use serde_json::Value;
use uuid;

//
// Type definitions
//
/// Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do
/// eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut
/// enim ad minim veniam, quis nostrud exercitation ullamco laboris
/// nisi ut aliquip ex ea commodo consequat.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Builder, Derivative)]
#[derivative(Default)]
#[serde(default)]
#[allow(non_snake_case)]
pub struct Test {
    /// JSON-LD header
    #[serde(flatten)]
    #[builder(default = "default_test_jsonld_header()")]
    #[derivative(Default(value = "default_test_jsonld_header()"))]
    pub jsonld: Option<JsonLdHeader>,

    /// The name of the test. This is a unique identifier that helps track
    /// individual test cases across the system. It should be
    /// descriptive and follow the standard naming conventions.

    #[builder(default = "2.0.to_string().into()", setter(into))]
    #[derivative(Default(value = "\"2.0\".to_string()"))]
    pub name: String,

    /// number
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(into))]
    #[derivative(Default)]
    pub number: Option<TestNumberType>,

    /// test2
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[builder(default, setter(into, each(name = "to_test2")))]
    #[derivative(Default)]
    pub test2: Vec<Test2>,

    /// ontology
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(into))]
    #[derivative(Default)]
    pub ontology: Option<Ontology>,

    /// Additional properties outside of the schema
    #[serde(flatten)]
    #[builder(default)]
    pub additional_properties: Option<HashMap<String, Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Builder, Derivative)]
#[derivative(Default)]
#[serde(default)]
#[allow(non_snake_case)]
pub struct Test2 {
    /// JSON-LD header
    #[serde(flatten)]
    #[builder(default = "default_test2_jsonld_header()")]
    #[derivative(Default(value = "default_test2_jsonld_header()"))]
    pub jsonld: Option<JsonLdHeader>,

    /// names
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[builder(default, setter(into, each(name = "to_names")))]
    #[derivative(Default)]
    pub names: Vec<String>,

    /// number
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(into))]
    #[derivative(Default)]
    pub number: Option<f64>,

    /// Additional properties outside of the schema
    #[serde(flatten)]
    #[builder(default)]
    pub additional_properties: Option<HashMap<String, Value>>,
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
// Default JSON-LD header function for each object

pub fn default_test_jsonld_header() -> Option<JsonLdHeader> {
    let mut context = SimpleContext::default();

    // Add main prefix and repository URL
    context.terms.insert("tst".to_string(), TermDef::Simple("https://www.github.com/my/repo/".to_string()));

    // Add configured prefixes
    context.terms.insert("schema".to_string(), TermDef::Simple("http://schema.org/".to_string()));

    // Add attribute terms
    context.terms.insert("name".to_string(), TermDef::Detailed(TermDetail {
        id: Some("schema:hello".to_string()),
        type_: Some("@id".to_string()),
        container: None,
        context: None,
    }));
    context.terms.insert("number".to_string(), TermDef::Simple("schema:one".to_string()));
    context.terms.insert("test2".to_string(), TermDef::Detailed(TermDetail {
        id: Some("schema:something".to_string()),
        type_: None,
        container: Some("@list".to_string()),
        context: None,
    }));

    Some(JsonLdHeader {
        context: Some(JsonLdContext::Object(context)),
        id: Some(format!("tst:Test/{}", uuid::Uuid::new_v4())),
        type_: Some(TypeOrVec::Multi(vec![
            "tst:Test".to_string(),
        ]))
    })
}


pub fn default_test2_jsonld_header() -> Option<JsonLdHeader> {
    let mut context = SimpleContext::default();

    // Add main prefix and repository URL
    context.terms.insert("tst".to_string(), TermDef::Simple("https://www.github.com/my/repo/".to_string()));

    // Add configured prefixes
    context.terms.insert("schema".to_string(), TermDef::Simple("http://schema.org/".to_string()));

    // Add attribute terms
    context.terms.insert("names".to_string(), TermDef::Detailed(TermDetail {
        id: Some("schema:hello".to_string()),
        type_: None,
        container: Some("@list".to_string()),
        context: None,
    }));
    context.terms.insert("number".to_string(), TermDef::Simple("schema:one".to_string()));

    Some(JsonLdHeader {
        context: Some(JsonLdContext::Object(context)),
        id: Some(format!("tst:Test2/{}", uuid::Uuid::new_v4())),
        type_: Some(TypeOrVec::Multi(vec![
            "tst:Test2".to_string(),
        ]))
    })
}


/// JSON-LD Header
///
/// JSON-LD (JavaScript Object Notation for Linked Data) provides a way to express
/// linked data using JSON syntax, enabling semantic web technologies and structured
/// data interchange with context and meaning preservation.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Eq, PartialEq)]
pub struct JsonLdHeader {
    /// JSON-LD context (IRI, object, or array)
    #[serde(rename = "@context", skip_serializing_if = "Option::is_none")]
    pub context: Option<JsonLdContext>,

    /// Node identifier (IRI or blank node)
    #[serde(rename = "@id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Type IRI(s) for the node, e.g. schema:Person
    #[serde(rename = "@type", skip_serializing_if = "Option::is_none")]
    pub type_: Option<TypeOrVec>,
}

impl Default for JsonLdHeader {
    /// Returns the default JSON-LD header.
    fn default() -> Self {
        Self {
            context: None,
            id: None,
            type_: None,
        }
    }
}

impl JsonLdHeader {
    /// Adds a new term definition to the JSON-LD context, creating a context object if none exists.
    ///
    /// This method provides a convenient way to extend the JSON-LD context with additional term
    /// mappings, allowing for semantic annotation of properties and values within the document.
    /// If the header does not already contain a context, a new SimpleContext object will be
    /// created automatically to hold the term definition.
    ///
    /// # Arguments
    ///
    /// * `name` - The term name to be defined in the context
    /// * `term` - The term definition, either a simple IRI mapping or a detailed definition
    ///
    /// # Example
    ///
    /// ```no_compile
    /// let mut header = JsonLdHeader::default();
    /// header.add_term("name", TermDef::Simple("https://schema.org/name".to_string()));
    /// ```
    pub fn add_term(&mut self, name: &str, term: TermDef) {
        let context = self
            .context
            .get_or_insert_with(|| JsonLdContext::Object(SimpleContext::default()));

        if let JsonLdContext::Object(object) = context {
            object.terms.insert(name.to_string(), term);
        }
    }

    /// Updates an existing term definition in the JSON-LD context or adds it if it doesn't exist.
    ///
    /// This method functions similarly to add_term but provides clearer semantics when the
    /// intention is to modify an existing term definition. The behavior is identical to add_term
    /// as HashMap::insert will overwrite existing entries with the same key, but this method
    /// name makes the intent more explicit in code that is updating rather than initially
    /// defining terms.
    ///
    /// # Arguments
    ///
    /// * `name` - The term name to be updated in the context
    /// * `term` - The new term definition to replace any existing definition
    ///
    /// # Example
    ///
    /// ```no_compile
    /// let mut header = JsonLdHeader::default();
    /// header.add_term("name", TermDef::Simple("https://schema.org/name".to_string()));
    /// header.update_term("name", TermDef::Simple("https://example.org/fullName".to_string()));
    /// ```
    pub fn update_term(&mut self, name: &str, term: TermDef) {
        let context = self
            .context
            .get_or_insert_with(|| JsonLdContext::Object(SimpleContext::default()));

        if let JsonLdContext::Object(object) = context {
            object.terms.insert(name.to_string(), term);
        }
    }

    /// Removes a term definition from the JSON-LD context if it exists.
    ///
    /// This method allows for the removal of previously defined terms from the JSON-LD context,
    /// which can be useful when dynamically managing context definitions or when certain terms
    /// are no longer needed in the semantic annotation of the document. The method will only
    /// attempt removal if the context exists and is an object type; it will silently do nothing
    /// if the context is missing or is not an object.
    ///
    /// # Arguments
    ///
    /// * `name` - The term name to be removed from the context
    ///
    /// # Returns
    ///
    /// Returns `true` if the term was found and removed, `false` if the term was not present
    /// or if the context is not an object type.
    ///
    /// # Example
    ///
    /// ```no_compile
    /// let mut header = JsonLdHeader::default();
    /// header.add_term("name", TermDef::Simple("https://schema.org/name".to_string()));
    /// let was_removed = header.remove_term("name");
    /// assert!(was_removed);
    /// ```
    pub fn remove_term(&mut self, name: &str) -> bool {
        if let Some(JsonLdContext::Object(object)) = &mut self.context {
            object.terms.remove(name).is_some()
        } else {
            false
        }
    }
}

/// Accept either a single type IRI or an array of them.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Eq, PartialEq)]
#[serde(untagged)]
pub enum TypeOrVec {
    Single(String),
    Multi(Vec<String>),
}

/// JSON-LD Context:
/// - a single IRI (remote context)
/// - an inline context object
/// - or an array of these (merged sequentially)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Eq, PartialEq)]
#[serde(untagged)]
pub enum JsonLdContext {
    Iri(String),
    Object(SimpleContext),
    Array(Vec<JsonLdContext>),
}

/// A simple inline @context object with essential global keys and term definitions.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default, Eq, PartialEq)]
pub struct SimpleContext {
    /// Base IRI used for relative resolution.
    #[serde(rename = "@base", skip_serializing_if = "Option::is_none")]
    pub base: Option<String>,

    /// Default vocabulary IRI for terms without explicit IRIs.
    #[serde(rename = "@vocab", skip_serializing_if = "Option::is_none")]
    pub vocab: Option<String>,

    /// Default language for string literals.
    #[serde(rename = "@language", skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,

    /// Mapping of term → IRI or detailed definition.
    #[serde(flatten, skip_serializing_if = "HashMap::is_empty", default)]
    pub terms: HashMap<String, TermDef>,
}

/// Term definition can be a simple mapping (string → IRI) or a detailed object.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Eq, PartialEq)]
#[serde(untagged)]
pub enum TermDef {
    /// Simple alias: `"name": "https://schema.org/name"`
    Simple(String),
    /// Expanded form with type coercion, container behavior, or nested context.
    Detailed(TermDetail),
}

/// Detailed term definition (subset of JSON-LD 1.1 features).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default, Eq, PartialEq)]
pub struct TermDetail {
    /// Absolute or relative IRI that the term expands to, or a keyword like "@id".
    #[serde(rename = "@id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Type coercion or value type ("@id", "@vocab", or datatype IRI).
    #[serde(rename = "@type", skip_serializing_if = "Option::is_none")]
    pub type_: Option<String>,

    /// Container behavior ("@list", "@set", "@index", etc.).
    #[serde(rename = "@container", skip_serializing_if = "Option::is_none")]
    pub container: Option<String>,

    /// Optional nested (scoped) context.
    #[serde(rename = "@context", skip_serializing_if = "Option::is_none")]
    pub context: Option<Box<JsonLdContext>>,
}