/*
 * Copyright (c) 2024 Jan Range
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

use crate::{
    attribute::Attribute,
    datamodel::DataModel,
    markdown::position::Position,
    object::{Enumeration, Object},
};
use colored::Colorize;
use log::error;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt::{Display, Formatter};

#[cfg(feature = "wasm")]
use tsify_next::Tsify;

// Basic types that are ignored in the validation process
pub(crate) const BASIC_TYPES: [&str; 7] = [
    "string", "number", "integer", "boolean", "float", "date", "bytes",
];

/// Represents a validation error in the data model.
#[derive(Debug, Clone, Serialize, PartialEq)]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi))]
pub struct ValidationError {
    pub message: String,
    pub object: Option<String>,
    pub attribute: Option<String>,
    pub location: String,
    pub error_type: ErrorType,
    pub positions: Vec<Position>,
}

impl Display for ValidationError {
    /// Formats the validation error for display.
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let lines: Vec<String> = self.positions.iter().map(|p| p.line.to_string()).collect();
        let mut line = lines.join(", ");

        if !lines.is_empty() {
            line = format!("[line: {}]", line);
        } else {
            line = "".to_string();
        }

        write!(
            f,
            "{}[{}{}] {}: {}",
            line,
            self.object.clone().unwrap_or("Global".into()).bold(),
            match &self.attribute {
                Some(attr) => format!(".{}", attr),
                None => "".into(),
            },
            self.error_type.to_string().bold(),
            self.message.red().bold(),
        )?;
        Ok(())
    }
}

/// Enum representing the type of validation error.
#[derive(Debug, Clone, Serialize, PartialEq, Deserialize)]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi))]
pub enum ErrorType {
    NameError,
    TypeError,
    DuplicateError,
    GlobalError,
}

impl Display for ErrorType {
    /// Formats the error type for display.
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorType::NameError => write!(f, "NameError"),
            ErrorType::TypeError => write!(f, "TypeError"),
            ErrorType::DuplicateError => write!(f, "DuplicateError"),
            ErrorType::GlobalError => write!(f, "GlobalError"),
        }
    }
}

/// Validator for checking the integrity of a data model.
#[derive(Debug, Clone, Serialize, PartialEq)]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi))]
pub struct Validator {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
    #[serde(skip_serializing)]
    pub object_positions: HashMap<String, Vec<Position>>,
    #[serde(skip_serializing)]
    pub enum_positions: HashMap<String, Vec<Position>>,
}

impl Error for Validator {}

impl Display for Validator {
    /// Formats the validator for display.
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for error in &self.errors {
            error.fmt(f)?;
        }
        Ok(())
    }
}

impl Validator {
    /// Creates a new instance of `Validator`.
    pub fn new() -> Self {
        Self {
            is_valid: true,
            errors: vec![],
            object_positions: HashMap::new(),
            enum_positions: HashMap::new(),
        }
    }
    pub fn reset(&mut self) {
        self.is_valid = true;
        self.errors.clear();
        self.object_positions.clear();
        self.enum_positions.clear();
    }

    /// Adds a validation error to the validator.
    ///
    /// # Arguments
    ///
    /// * `error` - The validation error to be added.
    pub fn add_error(&mut self, error: ValidationError) {
        self.errors.push(error);
        self.is_valid = false;
    }

    /// Prints all validation errors to the log.
    ///
    /// This method iterates over the `errors` vector and logs each error using the `error!` macro.
    pub fn log_result(&self) {
        for error in &self.errors {
            error!("{}", error);
        }
    }

    /// Validates the provided `DataModel`.
    ///
    /// # Arguments
    ///
    /// * `model` - A reference to the `DataModel` to be validated.
    ///
    /// # Returns
    ///
    /// A `Result` which is:
    /// - `Ok(())` if the model is valid.
    /// - `Err(Box<dyn Error>)` if the model is invalid.
    pub fn validate(&mut self, model: &DataModel) {
        // If there are errors from a previous validation, reset the validator
        self.reset();

        // Extract the positions of all objects, enums, and attributes
        self.object_positions = extract_object_positions(model);
        self.enum_positions = extract_enum_positions(model);

        // Extract the type names from the model
        let types = Self::extract_type_names(model);

        // Check for duplicate object names
        self.check_duplicate_objects(&model.objects);
        self.check_duplicate_enums(&model.enums);
        self.check_has_no_objects(model);

        // Validate the objects and enums
        for object in &model.objects {
            self.validate_object(object, &types);
        }

        self.sort_errors();
    }

    /// Checks for duplicate object names within the model.
    ///
    /// # Arguments
    ///
    /// * `collection` - A slice of `Object` instances to be checked.
    fn check_duplicate_objects(&mut self, collection: &[Object]) {
        let unique = collection
            .iter()
            .map(|object| object.name.as_str())
            .collect::<Vec<&str>>();

        let duplicates = unique_elements(&get_duplicates(&unique));

        if !duplicates.is_empty() {
            for name in duplicates {
                self.add_error(ValidationError {
                    message: format!("Object '{}' is defined more than once.", name),
                    object: Some(name.to_string()),
                    attribute: None,
                    location: "Global".into(),
                    error_type: ErrorType::DuplicateError,
                    positions: self
                        .object_positions
                        .get(name)
                        .cloned()
                        .unwrap_or_default()
                        .clone(),
                });
            }
        }
    }

    /// Checks for duplicate enum names within the model.
    ///
    /// # Arguments
    ///
    /// * `collection` - A slice of `Enumeration` instances to be checked.
    fn check_duplicate_enums(&mut self, collection: &[Enumeration]) {
        let unique = collection
            .iter()
            .map(|object| object.name.as_str())
            .collect::<Vec<&str>>();

        // Find duplicates
        let duplicates = unique
            .iter()
            .cloned()
            .filter(|&name| unique.iter().filter(|&n| n == &name).count() > 1)
            .collect::<Vec<&str>>();

        let duplicates = unique_elements(&duplicates);

        if !duplicates.is_empty() {
            for name in duplicates {
                self.add_error(ValidationError {
                    message: format!("Enumeration '{}' is defined more than once.", name),
                    object: Some(name.to_string()),
                    attribute: None,
                    location: "Global".into(),
                    error_type: ErrorType::DuplicateError,
                    positions: self
                        .enum_positions
                        .get(name)
                        .cloned()
                        .unwrap_or_default()
                        .clone(),
                });
            }
        }
    }

    /// Validates a single object within the data model.
    ///
    /// # Arguments
    ///
    /// * `object` - A reference to the `Object` to be validated.
    /// * `types` - A slice of type names that are valid within the model.
    fn validate_object(&mut self, object: &Object, types: &[&str]) {
        self.validate_object_name(&object.name);
        self.check_has_attributes(object);
        self.check_duplicate_attributes(object);

        // Validate the attributes of the object
        object.attributes.iter().for_each(|attribute| {
            self.validate_attribute(attribute, types, object);
        });
    }

    /// Checks for duplicate attribute names within an object.
    ///
    /// # Arguments
    ///
    /// * `object` - A reference to the `Object` to be checked.
    fn check_duplicate_attributes(&mut self, object: &Object) {
        // Check if the object has duplicate attributes
        let attr_names = object
            .attributes
            .iter()
            .map(|attribute| attribute.name.as_str())
            .collect::<Vec<&str>>();

        let attribute_positions = extract_attribute_positions(object);

        let unique = unique_elements(&attr_names);
        if attr_names.len() != unique.len() {
            let duplicates = unique_elements(&get_duplicates(&attr_names));

            for name in duplicates {
                self.add_error(ValidationError {
                    message: format!("Property '{}' is defined more than once.", name),
                    object: Some(object.name.clone()),
                    attribute: Some(name.to_string()),
                    location: "Global".into(),
                    error_type: ErrorType::DuplicateError,
                    positions: attribute_positions.get(name).cloned().unwrap_or_default(),
                });
            }
        }
    }

    /// Checks if an object has attributes.
    ///
    /// # Arguments
    ///
    /// * `object` - A reference to the `Object` to be checked.
    fn check_has_attributes(&mut self, object: &Object) {
        if !object.has_attributes() {
            self.add_error(ValidationError {
                message: format!("Type '{}' is empty and has no properties.", object.name),
                object: Some(object.name.clone()),
                attribute: None,
                location: "Global".into(),
                error_type: ErrorType::TypeError,
                positions: self
                    .object_positions
                    .get(&object.name)
                    .cloned()
                    .unwrap_or_default(),
            });
        }
    }

    /// Validates the name of an object.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the object to be validated.
    fn validate_object_name(&mut self, name: &str) {
        let checks = vec![
            starts_with_character,
            contains_white_space,
            contains_special_characters,
        ];

        for check in checks {
            if let Err(e) = check(name) {
                self.add_error(ValidationError {
                    message: e,
                    object: Some(name.to_string()),
                    attribute: None,
                    location: "Global".into(),
                    error_type: ErrorType::NameError,
                    positions: self.object_positions.get(name).cloned().unwrap_or_default(),
                });
            }
        }
    }

    /// Checks if the model has no objects.
    ///
    /// # Arguments
    ///
    /// * `model` - A reference to the `DataModel` to be checked.
    fn check_has_no_objects(&mut self, model: &DataModel) {
        if model.objects.is_empty() {
            self.add_error(ValidationError {
                message: "This model has no definitions.".into(),
                object: Some("Model".into()),
                attribute: None,
                location: "Global".into(),
                error_type: ErrorType::GlobalError,
                positions: vec![],
            });
        }
    }

    /// Validates a single attribute within an object.
    ///
    /// # Arguments
    ///
    /// * `attribute` - A reference to the `Attribute` to be validated.
    /// * `types` - A slice of type names that are valid within the model.
    /// * `obj_name` - The name of the object that contains the attribute.
    fn validate_attribute(&mut self, attribute: &Attribute, types: &[&str], object: &Object) {
        self.validate_attribute_name(&attribute.name, object);

        let attribute_positions = extract_attribute_positions(object);

        if attribute.dtypes.is_empty() {
            self.add_error(ValidationError {
                message: format!("Property '{}' has no type specified.", attribute.name),
                object: Some(object.name.clone()),
                attribute: Some(attribute.name.clone()),
                location: "Global".into(),
                error_type: ErrorType::TypeError,
                positions: attribute_positions
                    .get(&attribute.name)
                    .cloned()
                    .unwrap_or_default(),
            })
        }

        for dtype in &attribute.dtypes {
            self.check_attr_dtype(attribute, types, object, dtype);
        }
    }

    /// Checks the data type of attribute.
    ///
    /// # Arguments
    ///
    /// * `attribute` - A reference to the `Attribute` to be checked.
    /// * `types` - A slice of type names that are valid within the model.
    /// * `obj_name` - The name of the object that contains the attribute.
    /// * `dtype` - The data type of the attribute to be checked.
    fn check_attr_dtype(
        &mut self,
        attribute: &Attribute,
        types: &[&str],
        object: &Object,
        dtype: &str,
    ) {
        let attribute_positions = extract_attribute_positions(object);

        if dtype.is_empty() {
            self.add_error(ValidationError {
                message: format!(
                    "Property '{}' has no type defined. Either define a type or use a base type.",
                    attribute.name
                ),
                object: Some(object.name.clone()),
                attribute: Some(attribute.name.clone()),
                location: "Global".into(),
                error_type: ErrorType::TypeError,
                positions: attribute_positions
                    .get(&attribute.name)
                    .cloned()
                    .unwrap_or_default(),
            });

            return;
        }

        if !types.contains(&dtype) && !BASIC_TYPES.contains(&dtype) {
            self.add_error(ValidationError {
                message: format!(
                    "Type '{}' of property '{}' not found. Either define the type or use a base type.",
                    dtype, attribute.name
                ),
                object: Some(object.name.clone()),
                attribute: Some(attribute.name.clone()),
                location: "Global".into(),
                error_type: ErrorType::TypeError,
                positions: attribute_positions
                    .get(&attribute.name)
                    .cloned()
                    .unwrap_or_default(),
            })
        }
    }

    /// Validates the name of an attribute.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the attribute to be validated.
    /// * `obj_name` - The name of the object that contains the attribute.
    fn validate_attribute_name(&mut self, name: &str, object: &Object) {
        let checks = vec![
            starts_with_character,
            contains_white_space,
            contains_special_characters,
        ];

        let attribute_positions = extract_attribute_positions(object);

        for check in checks {
            if let Err(e) = check(name) {
                self.add_error(ValidationError {
                    message: e,
                    object: Some(object.name.clone()),
                    attribute: Some(name.to_string()),
                    location: "Global".into(),
                    error_type: ErrorType::NameError,
                    positions: attribute_positions.get(name).cloned().unwrap_or_default(),
                });
            }
        }
    }

    /// Extracts the type names from the data model.
    ///
    /// # Arguments
    ///
    /// * `model` - A reference to the `DataModel` to extract type names from.
    ///
    /// # Returns
    ///
    /// A vector of type names.
    fn extract_type_names(model: &DataModel) -> Vec<&str> {
        let types = model
            .objects
            .iter()
            .map(|object| object.name.as_str())
            .chain(model.enums.iter().map(|enum_| enum_.name.as_str()))
            .collect::<Vec<&str>>();
        types
    }

    /// Sorts the validation errors by their line number, allowing for easier identification
    /// of issues in the source code. The sorting is done in-place on the `errors` vector.
    fn sort_errors(&mut self) {
        self.errors.sort_by(|a, b| {
            let line_a = a.positions.first().map(|pos| pos.line);
            let line_b = b.positions.first().map(|pos| pos.line);
            line_a.cmp(&line_b)
        });
    }
}

impl Default for Validator {
    /// Provides a default implementation for `Validator`.
    fn default() -> Self {
        Self::new()
    }
}

/// Returns a list of unique elements from a slice.
///
/// # Arguments
///
/// * `input` - A slice of elements to be checked for uniqueness.
///
/// # Returns
///
/// A vector of unique elements.
fn unique_elements<T: Eq + std::hash::Hash + Clone>(input: &[T]) -> Vec<T> {
    let mut set = HashSet::new();

    for item in input {
        set.insert(item.clone());
    }

    set.into_iter().collect()
}

/// Returns a list of duplicate elements from a slice.
///
/// # Arguments
///
/// * `collection` - A slice of elements to be checked for duplicates.
///
/// # Returns
///
/// A vector of duplicate elements.
fn get_duplicates<'a>(collection: &'a [&'a str]) -> Vec<&'a str> {
    let mut seen = HashSet::new();
    let mut duplicates = HashSet::new();

    for &item in collection {
        if !seen.insert(item) {
            duplicates.insert(item);
        }
    }

    duplicates.into_iter().collect()
}

/// Checks if the given name starts with an alphabetic character.
///
/// # Arguments
///
/// * `name` - A string slice that holds the name to be checked.
///
/// # Returns
///
/// A `Result` which is:
/// - `Ok(())` if the name starts with an alphabetic character.
/// - `Err(String)` if the name does not start with an alphabetic character.
fn starts_with_character(name: &str) -> Result<(), String> {
    match name.chars().next() {
        Some(c) if c.is_alphabetic() => Ok(()),
        _ => Err(format!("Name '{}' must start with a letter.", name)),
    }
}

/// Checks if the given name contains whitespace.
///
/// # Arguments
///
/// * `name` - A string slice that holds the name to be checked.
///
/// # Returns
///
/// A `Result` which is:
/// - `Ok(())` if the name does not contain whitespace.
/// - `Err(String)` if the name contains whitespace.
fn contains_white_space(name: &str) -> Result<(), String> {
    name.contains(' ')
        .then(|| {
            Err(format!(
                "Name '{}' contains whitespace, which is not valid. Use underscores instead.",
                name
            ))
        })
        .unwrap_or(Ok(()))
}

/// Checks if the given name contains special characters, except for underscores.
///
/// # Arguments
///
/// * `name` - A string slice that holds the name to be checked.
///
/// # Returns
///
/// A `Result` which is:
/// - `Ok(())` if the name does not contain special characters.
/// - `Err(String)` if the name contains special characters.
fn contains_special_characters(name: &str) -> Result<(), String> {
    name.chars().any(|c| !c.is_alphanumeric() && c != '_' && c != ' ').then(
        || Err(format!("Name '{}' contains special characters, which are not valid except for underscores.", name))
    ).unwrap_or(Ok(()))
}

/// Extracts the positions of all objects in the data model.
///
/// # Arguments
///
/// * `model` - A reference to the `DataModel` to extract positions from.
///
/// # Returns
///
/// A `HashMap` mapping object names to their positions in the source code.
fn extract_object_positions(model: &DataModel) -> HashMap<String, Vec<Position>> {
    let mut positions: HashMap<String, Vec<Position>> = HashMap::new();
    for object in &model.objects {
        if object.position.is_none() {
            continue;
        }

        if let Some(pos) = positions.get_mut(&object.name) {
            pos.push(object.position.unwrap());
        } else {
            positions.insert(object.name.clone(), vec![object.position.unwrap()]);
        }
    }
    positions
}

/// Extracts the positions of all enums in the data model.
///
/// # Arguments
///
/// * `model` - A reference to the `DataModel` to extract positions from.
///
/// # Returns
///
/// A `HashMap` mapping enum names to their positions in the source code.
fn extract_enum_positions(model: &DataModel) -> HashMap<String, Vec<Position>> {
    let mut positions: HashMap<String, Vec<Position>> = HashMap::new();
    for enum_ in &model.enums {
        if enum_.position.is_none() {
            continue;
        }

        if let Some(pos) = positions.get_mut(&enum_.name) {
            pos.push(enum_.position.unwrap());
        } else {
            positions.insert(enum_.name.clone(), vec![enum_.position.unwrap()]);
        }
    }
    positions
}

/// Extracts the positions of all attributes across all objects in the data model.
///
/// # Arguments
///
/// * `model` - A reference to the `DataModel` to extract positions from.
///
/// # Returns
///
/// A `HashMap` mapping attribute names to their positions in the source code.
fn extract_attribute_positions(object: &Object) -> HashMap<String, Vec<Position>> {
    let mut positions: HashMap<String, Vec<Position>> = HashMap::new();
    for attribute in &object.attributes {
        if attribute.position.is_none() {
            continue;
        }

        if let Some(pos) = positions.get_mut(&attribute.name) {
            pos.push(attribute.position.unwrap());
        } else {
            positions.insert(attribute.name.clone(), vec![attribute.position.unwrap()]);
        }
    }
    positions
}
