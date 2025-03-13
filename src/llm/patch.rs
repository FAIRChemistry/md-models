//! # JSON Patch Implementation
//!
//! This module provides functionality for applying JSON Patch operations to JSON data
//! according to [RFC 6902](https://tools.ietf.org/html/rfc6902).
//!
//! The implementation supports the standard operations:
//! - add: adds a value at a specified location
//! - remove: removes a value at a specified location
//! - replace: replaces a value at a specified location
//! - move: moves a value from one location to another
//! - copy: copies a value from one location to another
//!
//! After applying patches, the module validates the resulting JSON against a data model
//! to ensure the modifications maintain data integrity.

use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};
use serde_json::{from_value, Value};
use thiserror::Error;

use crate::{
    json::validation::{validate_json, ValidationError},
    prelude::DataModel,
};

/// Represents a collection of JSON Patch operations to be applied to a JSON document.
///
/// This struct follows the JSON Patch specification (RFC 6902) and contains
/// a list of operations to be applied in sequence.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub(crate) struct JSONPatch {
    #[schemars(description = "A collection of JSON patch operations to be applied")]
    patches: Vec<JSONPatchOp>,
}

impl JSONPatch {
    /// Generates a JSON Schema for the JSONPatch structure.
    ///
    /// This is useful for validating incoming JSON Patch documents
    /// or for generating documentation.
    ///
    /// # Returns
    /// A JSON Value representing the schema for JSONPatch
    #[allow(dead_code)]
    pub(crate) fn schema() -> Value {
        let schema = schema_for!(JSONPatch);
        serde_json::to_value(schema).unwrap()
    }

    /// Applies the JSON Patch operations to the provided JSON dataset and validates
    /// the result against the data model.
    ///
    /// # Arguments
    /// * `dataset` - The JSON data to be modified
    /// * `model` - The data model to validate against after applying patches
    /// * `root` - Optional root element name for validation context
    ///
    /// # Returns
    /// * `Ok(Vec<ValidationError>)` - Empty vector if validation passes, or validation errors if present
    /// * `Err(PatchError)` - If patch application fails
    #[allow(dead_code)]
    pub(crate) fn apply(
        &self,
        dataset: &mut Value,
        model: &DataModel,
        root: Option<String>,
    ) -> Result<Vec<ValidationError>, PatchError> {
        // Convert our JSONPatch to the format expected by json_patch crate
        let patch = json_patch::Patch::try_from(self)?;

        // Apply the patch operations to the dataset
        json_patch::patch(dataset, &patch)?;

        // Validate the modified dataset against the data model
        validate_json(dataset, model, root).map_err(PatchError::Other)
    }
}

/// Represents individual JSON Patch operations as defined in RFC 6902.
///
/// Each variant corresponds to one of the standard operations:
/// add, remove, replace, move, and copy.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", tag = "op")]
pub(crate) enum JSONPatchOp {
    #[schemars(description = "Add a new value to the path")]
    Add { path: String, value: Value },

    #[schemars(description = "Remove a value from the path")]
    Remove { path: String },

    #[schemars(description = "Replace a value at the path")]
    Replace { path: String, value: Value },

    #[schemars(description = "Move a value from one path to another")]
    Move { from: String, path: String },

    #[schemars(description = "Copy a value from one path to another")]
    Copy { from: String, path: String },
}

/// Conversion implementation to transform our JSONPatch into the format
/// expected by the json_patch crate.
impl TryFrom<&JSONPatch> for json_patch::Patch {
    type Error = serde_json::Error;

    fn try_from(patch: &JSONPatch) -> Result<Self, Self::Error> {
        // Convert our patch operations to JSON and then deserialize into json_patch format
        let patches = serde_json::to_value(&patch.patches).unwrap();
        from_value(patches)
    }
}

/// Error types that can occur during JSON patch operations.
#[derive(Debug, Error)]
pub enum PatchError {
    /// Errors during serialization or deserialization of JSON
    #[error("serde error: {0}")]
    Serde(#[from] serde_json::Error),

    /// Errors from the json_patch library during patch application
    #[error("json patch error: {0}")]
    JsonPatch(#[from] json_patch::PatchError),

    /// Other errors, typically from validation after patching
    #[error("other error while patching: {0}")]
    Other(#[from] Box<dyn std::error::Error>),
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    use jsonschema::error::ValidationErrorKind;
    use serde_json::json;

    #[test]
    fn test_schema_generation() {
        // Test that we can generate a valid JSON schema for JSONPatch
        let schema = JSONPatch::schema();
        assert!(schema.is_object());

        // Verify schema contains expected fields
        let schema_obj = schema.as_object().unwrap();
        assert!(schema_obj.contains_key("properties"));
        assert!(schema_obj.contains_key("type"));
    }

    #[test]
    fn test_json_patch_serialization() {
        // Create a patch with multiple operations
        let patch = JSONPatch {
            patches: vec![
                JSONPatchOp::Add {
                    path: "/foo".to_string(),
                    value: json!("bar"),
                },
                JSONPatchOp::Replace {
                    path: "/baz".to_string(),
                    value: json!(42),
                },
            ],
        };

        // Test serialization and deserialization
        let serialized = serde_json::to_string(&patch).unwrap();
        let deserialized: JSONPatch = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.patches.len(), 2);

        // Verify first operation (Add)
        if let JSONPatchOp::Add { path, value } = &deserialized.patches[0] {
            assert_eq!(path, "/foo");
            assert_eq!(value, &json!("bar"));
        } else {
            panic!("Expected Add operation");
        }

        // Verify second operation (Replace)
        if let JSONPatchOp::Replace { path, value } = &deserialized.patches[1] {
            assert_eq!(path, "/baz");
            assert_eq!(value, &json!(42));
        } else {
            panic!("Expected Replace operation");
        }
    }

    #[test]
    fn test_conversion_to_json_patch() {
        // Test conversion from our JSONPatch to json_patch::Patch
        let patch = JSONPatch {
            patches: vec![JSONPatchOp::Add {
                path: "/foo".to_string(),
                value: json!("bar"),
            }],
        };

        json_patch::Patch::try_from(&patch).expect("Failed to convert to json patch");
    }

    #[test]
    fn test_all_patch_operations() {
        // Test creating patches with all operation types
        let patches = vec![
            JSONPatchOp::Add {
                path: "/add".to_string(),
                value: json!("value"),
            },
            JSONPatchOp::Remove {
                path: "/remove".to_string(),
            },
            JSONPatchOp::Replace {
                path: "/replace".to_string(),
                value: json!("new_value"),
            },
        ];

        let patch = JSONPatch { patches };
        let serialized = serde_json::to_string(&patch).unwrap();

        // Verify we can deserialize back
        let _: JSONPatch = serde_json::from_str(&serialized).unwrap();
    }

    #[test]
    fn test_apply() {
        // Arrange
        let path = Path::new("tests/data/model.md");
        let model = DataModel::from_markdown(path).expect("Failed to load model");

        // Create a test dataset that matches our model
        let mut dataset = json!({
            "name": "John Doe",
            "number": 30,
            "test2": [
                {
                    "names": ["Jane Doe", "John Smith"],
                    "number": 25,
                }
            ],
        });

        // Create a patch with multiple operations
        let patch = JSONPatch {
            patches: vec![
                JSONPatchOp::Add {
                    path: "/test2/0/names/-".to_string(),
                    value: json!("Ben Doe"),
                },
                JSONPatchOp::Replace {
                    path: "/name".to_string(),
                    value: json!("Jane Doe"),
                },
                JSONPatchOp::Replace {
                    path: "/test2/0/number".to_string(),
                    value: json!(26),
                },
                JSONPatchOp::Remove {
                    path: "/number".to_string(),
                },
            ],
        };

        // Act
        let result = patch
            .apply(&mut dataset, &model, Some("Test".to_string()))
            .expect("Failed to apply patch");

        // Assert
        assert!(result.is_empty(), "Expected no errors");
        assert_eq!(dataset["name"], "Jane Doe");
        assert_eq!(dataset["number"], json!(null));
        assert_eq!(dataset["test2"][0]["number"], 26);
        assert_eq!(
            dataset["test2"][0]["names"],
            json!(["Jane Doe", "John Smith", "Ben Doe"])
        );
    }

    #[test]
    fn test_apply_invalid_patch() {
        // Arrange
        let path = Path::new("tests/data/model.md");
        let model = DataModel::from_markdown(path).expect("Failed to load model");

        // Create a test dataset
        let mut dataset = json!({
            "name": "John Doe",
            "number": 30,
            "test2": [
                {
                    "names": ["Jane Doe", "John Smith"],
                    "number": 25,
                }
            ],
        });

        // Create a patch that will make the dataset invalid (removing required field)
        let patch = JSONPatch {
            patches: vec![JSONPatchOp::Remove {
                path: "/name".to_string(),
            }],
        };

        // Act
        let result = patch
            .apply(&mut dataset, &model, Some("Test".to_string()))
            .expect("Expected error");

        // Assert
        assert!(result.len() > 0);

        // Verify we get the expected validation error (missing required field)
        let error = result.first().unwrap();
        assert!(matches!(
            &error.kind,
            ValidationErrorKind::Required { property: _ }
        ));
    }
}
