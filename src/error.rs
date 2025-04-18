//! Error types for the mdmodels crate.
//!
//! This module defines the error types used throughout the crate,
//! particularly for operations related to data model processing.

use thiserror::Error;

use crate::prelude::Validator;

/// Errors that can occur when working with data models.
///
/// This enum represents the various error conditions that may arise
/// during data model operations such as validation, deserialization,
/// and file I/O.
#[derive(Debug, Error)]
pub enum DataModelError {
    /// Error that occurs when a data model fails validation.
    ///
    /// Contains the validator with detailed validation errors.
    #[error("Validation error: {0}")]
    ValidationError(Validator),

    /// Error that occurs when deserializing JSON data.
    ///
    /// This typically happens when parsing JSON schemas or model data.
    #[error("Deserialize error: {0}")]
    DeserializeError(#[from] serde_json::Error),

    /// Error that occurs when reading files.
    ///
    /// This can happen when attempting to read model files from disk.
    #[error("Read error: {0}")]
    ReadError(#[from] std::io::Error),
}
