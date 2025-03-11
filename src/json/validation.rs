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

use std::error::Error;
use std::path::PathBuf;

use colored::Colorize;
use jsonschema::error::ValidationErrorKind;
use serde_json::Value;
use std::convert::TryFrom;

use crate::datamodel::DataModel;
use jsonschema::validator_for;

/// Represents a validation error that occurs during dataset validation.
#[derive(Debug)]
pub struct ValidationError {
    pub instance_path: String,
    pub schema_path: String,
    pub message: String,
    pub kind: ValidationErrorKind,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Validation Error: Instance {} violates schema at {}: {}",
            self.instance_path.red().bold(),
            self.schema_path.green().bold(),
            self.message.yellow().bold()
        )
    }
}

impl From<jsonschema::ValidationError<'_>> for ValidationError {
    fn from(err: jsonschema::ValidationError) -> Self {
        ValidationError {
            instance_path: err.instance_path.to_string(),
            schema_path: err.schema_path.to_string(),
            message: err.to_string(),
            kind: err.kind,
        }
    }
}

/// Validates a dataset against a given DataModel.
///
/// # Arguments
///
/// * `dataset` - The dataset to validate, which can be provided in various forms.
/// * `model` - A reference to the DataModel against which the dataset will be validated.
/// * `root` - An optional root path for the schema.
///
/// # Returns
///
/// A Result containing a vector of ValidationErrors if validation fails, or an empty vector if successful.
pub fn validate_json<T: Into<DatasetInput>>(
    dataset: T,
    model: &DataModel,
    root: Option<String>,
) -> Result<Vec<ValidationError>, Box<dyn Error>> {
    // Convert the dataset input to a Value
    let dataset_input: DatasetInput = dataset.into();
    let value: Value = dataset_input.try_into()?;

    // Get the JSON Schema from the model
    let schema = model.json_schema(root, false)?;
    let schema_value: Value = serde_json::from_str(&schema)?;

    // Create a validator for the schema
    let validator = validator_for(&schema_value)?;

    // Validate the dataset against the schema
    let result = validator.iter_errors(&value);
    let mut errors: Vec<ValidationError> = Vec::new();

    for err in result {
        errors.push(ValidationError::from(err));
    }

    Ok(errors)
}

/// Enum representing the different types of dataset inputs.
pub enum DatasetInput {
    Path(PathBuf),
    Value(Value),
    String(String),
}

impl From<PathBuf> for DatasetInput {
    /// Converts a PathBuf into a DatasetInput.
    fn from(path: PathBuf) -> Self {
        DatasetInput::Path(path)
    }
}

impl From<Value> for DatasetInput {
    /// Converts a Value into a DatasetInput.
    fn from(value: Value) -> Self {
        DatasetInput::Value(value)
    }
}

impl From<&mut Value> for DatasetInput {
    /// Converts a Value into a DatasetInput.
    fn from(value: &mut Value) -> Self {
        DatasetInput::Value(value.clone())
    }
}

impl From<String> for DatasetInput {
    /// Converts a String into a DatasetInput.
    fn from(string: String) -> Self {
        DatasetInput::String(string)
    }
}

impl TryFrom<DatasetInput> for Value {
    type Error = Box<dyn Error>;

    fn try_from(input: DatasetInput) -> Result<Self, Self::Error> {
        match input {
            DatasetInput::Path(path) => {
                // Logic to read from the path and convert to Value
                let content = std::fs::read_to_string(path)?;
                let value: Value = serde_json::from_str(&content)?;
                Ok(value)
            }
            DatasetInput::Value(value) => Ok(value),
            DatasetInput::String(string) => {
                let value: Value = serde_json::from_str(&string)?;
                Ok(value)
            }
        }
    }
}
