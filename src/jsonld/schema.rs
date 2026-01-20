use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

/// Represents the JSON-LD document header, containing context, id, and type definitions.
/// This structure is used as the envelope for JSON-LD documents and nodes.
///
/// The fields correspond to the standard JSON-LD keywords:
/// - `@context`: Describes the term definitions and mapping information; can be an IRI, an object, or an array of contexts.
/// - `@id`: The node identifier, usually as an IRI string.
/// - `@type`: The semantic type(s) of the node as a string or array of strings.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Default)]
pub struct JsonLdHeader {
    #[serde(rename = "@context", skip_serializing_if = "Option::is_none")]
    pub context: Option<JsonLdContext>,

    #[serde(rename = "@id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    #[serde(rename = "@type", skip_serializing_if = "Option::is_none")]
    pub type_: Option<TypeOrVec>,
}

impl JsonLdHeader {
    /// Add a new term definition to the context, creating the context object if not present.
    ///
    /// # Arguments
    ///
    /// * `name` - The key/term to be added to the `@context`
    /// * `term` - The associated mapping (simple IRI, detailed mapping, or null)
    pub fn add_term(&mut self, name: &str, term: TermDef) {
        let ctx = self
            .context
            .get_or_insert_with(|| JsonLdContext::Object(SimpleContext::default()));
        if let JsonLdContext::Object(object) = ctx {
            object.terms.insert(name.to_string(), term);
        }
    }

    /// Insert or replace a term definition in the context.
    pub fn update_term(&mut self, name: &str, term: TermDef) {
        self.add_term(name, term);
    }

    /// Remove a term from the context. Returns true if the term existed and was removed.
    pub fn remove_term(&mut self, name: &str) -> bool {
        if let Some(JsonLdContext::Object(object)) = &mut self.context {
            object.terms.shift_remove(name).is_some()
        } else {
            false
        }
    }

    /// Add an `@import` field as required by JSON-LD 1.1.
    /// If `@import` already exists, merges multiple contexts into an array.
    ///
    /// # Arguments
    ///
    /// * `import_url` - The URL (IRI) to include with `@import`.
    pub fn add_import(&mut self, import_url: impl Into<String>) {
        let import_url = import_url.into();
        match self.context.take() {
            None => {
                let mut obj = SimpleContext::default();
                obj.import = Some(import_url);
                self.context = Some(JsonLdContext::Object(obj));
            }
            Some(JsonLdContext::Object(mut obj)) => {
                if obj.import.is_none() {
                    obj.import = Some(import_url);
                    self.context = Some(JsonLdContext::Object(obj));
                } else {
                    let mut arr = Vec::new();
                    arr.push(JsonLdContext::Object(obj));
                    arr.push(JsonLdContext::Object(SimpleContext {
                        import: Some(import_url),
                        ..Default::default()
                    }));
                    self.context = Some(JsonLdContext::Array(arr));
                }
            }
            Some(JsonLdContext::Iri(iri)) => {
                let mut arr = Vec::new();
                arr.push(JsonLdContext::Iri(iri));
                arr.push(JsonLdContext::Object(SimpleContext {
                    import: Some(import_url),
                    ..Default::default()
                }));
                self.context = Some(JsonLdContext::Array(arr));
            }
            Some(JsonLdContext::Array(mut arr)) => {
                arr.push(JsonLdContext::Object(SimpleContext {
                    import: Some(import_url),
                    ..Default::default()
                }));
                self.context = Some(JsonLdContext::Array(arr));
            }
        }
    }
}

/// Represents the possible values for the `@type` field in JSON-LD nodes.
/// Accepts a single type string or an array of type strings.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(untagged)]
pub enum TypeOrVec {
    /// A single type annotation as a string
    Single(String),
    /// Multiple types as a vector of strings
    Multi(Vec<String>),
}

/// Describes how a JSON-LD `@context` may be represented:
/// as a remote IRI, an inline object, or a list of multiple contexts.
///
/// This flexibility enables referencing remote contexts, inlining ad-hoc mappings,
/// or combining several contexts in a single document.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(untagged)]
pub enum JsonLdContext {
    /// The context is a remote document specified by an IRI.
    Iri(String),
    /// The context is a locally inlined object containing mappings and settings.
    Object(SimpleContext),
    /// The context is a list of IRIs and/or objects, processed in order.
    Array(Vec<JsonLdContext>),
}

/// Defines the structure of an inline JSON-LD `@context` object, supporting:
/// - reserved context keywords (e.g., `@base`, `@vocab`, `@protected`)
/// - term mappings for expanding compact keys to IRIs or more detailed definitions
///
/// Most reserved keywords are optional.
/// Doubly-Option fields (e.g., `Option<Option<String>>`) allow distinguishing between
/// omitted, null, or non-null values when serializing.
#[derive(Debug, Clone, Serialize, Deserialize, Default, Eq, PartialEq)]
pub struct SimpleContext {
    /// `@import`: Import external context definition (JSON-LD 1.1 feature).
    #[serde(rename = "@import", skip_serializing_if = "Option::is_none")]
    pub import: Option<String>,

    /// `@type`: The type of the context object.
    #[serde(rename = "@type", skip_serializing_if = "Option::is_none")]
    pub type_: Option<TypeOrVec>,

    /// `@version`: Indicates the JSON-LD version this context targets.
    #[serde(rename = "@version", skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    /// `@base`: The default base IRI; Some(None) serializes to null (explicitly clear).
    #[serde(rename = "@base", skip_serializing_if = "Option::is_none")]
    pub base: Option<Option<String>>,

    /// `@vocab`: The default vocabulary IRI; Some(None) outputs null for explicit clearing.
    #[serde(rename = "@vocab", skip_serializing_if = "Option::is_none")]
    pub vocab: Option<Option<String>>,

    /// `@language`: Default language code. Some(None) serializes as null.
    #[serde(rename = "@language", skip_serializing_if = "Option::is_none")]
    pub language: Option<Option<String>>,

    /// `@direction`: Base direction for string values ("ltr" or "rtl").
    #[serde(rename = "@direction", skip_serializing_if = "Option::is_none")]
    pub direction: Option<String>,

    /// `@protected`: When true, marks all term definitions in this context as protected.
    #[serde(rename = "@protected", skip_serializing_if = "Option::is_none")]
    pub protected: Option<bool>,

    /// Key-value term definitions (mappings or detailed definitions for context terms).
    #[serde(flatten, skip_serializing_if = "IndexMap::is_empty", default)]
    pub terms: IndexMap<String, TermDef>,
}

/// Represents a context term mapping:
/// - as a simple IRI string,
/// - as a detailed object (`TermDetail`) with advanced options,
/// - or explicitly `null` to remove or overwrite a term definition.
///
/// The null variant uses `serde_json::Value::Null` under the hood to allow proper null emission in serialization.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(untagged)]
pub enum TermDef {
    /// Direct mapping to an IRI (string).
    Simple(String),
    /// Full-featured definition using advanced JSON-LD keyword fields.
    Detailed(TermDetail),
    /// Explicit null value for term removal or overriding.
    Null(JsonValue), // Must be JsonValue::Null in use.
}

/// Provides a full mapping for a term in a JSON-LD context, supporting advanced features such as:
/// type coercion, container types, nested context definition, language, protection, prefixing, reverse properties, and data nesting.
///
/// Each field is optional and maps to the appropriate JSON-LD 1.1 feature.
#[derive(Debug, Clone, Serialize, Deserialize, Default, Eq, PartialEq)]
pub struct TermDetail {
    /// The IRI or keyword that this term maps to (`@id`).
    #[serde(rename = "@id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Type coercion target for values (`@type`). E.g., "@id", "@vocab", or custom IRI.
    #[serde(rename = "@type", skip_serializing_if = "Option::is_none")]
    pub type_: Option<String>,

    /// Specifies container type(s) (e.g., ["@set"], "@list"). Single value or array.
    #[serde(rename = "@container", skip_serializing_if = "Option::is_none")]
    pub container: Option<OneOrMany<String>>,

    /// Nested context (`@context`) to use for this term, if any.
    #[serde(rename = "@context", skip_serializing_if = "Option::is_none")]
    pub context: Option<Box<JsonLdContext>>,

    /// Marks this context term as protected (`@protected`).
    #[serde(rename = "@protected", skip_serializing_if = "Option::is_none")]
    pub protected: Option<bool>,

    /// Whether this term provides prefix expansion (`@prefix`).
    #[serde(rename = "@prefix", skip_serializing_if = "Option::is_none")]
    pub prefix: Option<bool>,

    /// Defines a reverse property mapping (`@reverse`).
    #[serde(rename = "@reverse", skip_serializing_if = "Option::is_none")]
    pub reverse: Option<String>,

    /// Directs compacted output to nest data at this property (`@nest`).
    #[serde(rename = "@nest", skip_serializing_if = "Option::is_none")]
    pub nest: Option<String>,

    /// Assigns the property to a named index (`@index`).
    #[serde(rename = "@index", skip_serializing_if = "Option::is_none")]
    pub index: Option<String>,

    /// Sets the default language for string values in this term. If Some(None), serializes as explicit null.
    #[serde(rename = "@language", skip_serializing_if = "Option::is_none")]
    pub language: Option<Option<String>>,
}

/// Utility type that accepts either a single element or a list, used for keywords like
/// `@container` which can take a string or array of strings in JSON-LD.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(untagged)]
pub enum OneOrMany<T> {
    /// Single value.
    One(T),
    /// Multiple values.
    Many(Vec<T>),
}

impl TermDef {
    /// Convenience constructor for a null-valued term, meaning term removal or explicit undefinition.
    pub fn null() -> Self {
        TermDef::Null(JsonValue::Null)
    }
}

impl SimpleContext {
    /// Ensures the inline context explicitly contains `@version: 1.1`.
    /// Returns self for fluent usage.
    pub fn ensure_v11(mut self) -> Self {
        self.version = Some("1.1".to_string());
        self
    }
}
