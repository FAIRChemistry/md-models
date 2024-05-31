use std::{collections::HashSet, error::Error};

use crate::{
    attribute::Attribute,
    datamodel::DataModel,
    object::{Enumeration, Object},
};
use colored::Colorize;
use log::error;

// Basic types that are ignored in the validation process
const BASIC_TYPES: [&str; 6] = ["string", "number", "integer", "boolean", "float", "date"];

/// Validator for checking the integrity of a data model.
pub struct Validator {
    is_valid: bool,
}

impl Validator {
    /// Creates a new instance of `Validator`.
    pub fn new() -> Self {
        Self { is_valid: true }
    }

    /// Validates the provided `DataModel`.
    ///
    /// # Arguments
    ///
    /// * `model` - A reference to the `DataModel` to be validated.
    pub fn validate(&mut self, model: &DataModel) -> Result<(), Box<dyn Error>> {
        // Check for duplicate object names
        let result_objs = check_duplicate_objects(&model.objects);
        if result_objs.is_err() {
            self.is_valid = false;
        }

        // Check for duplicate enum names
        let result_enums = check_duplicate_enums(&model.enums);
        if result_enums.is_err() {
            self.is_valid = false;
        }

        // Get the list of object types
        let types = model
            .objects
            .iter()
            .map(|object| object.name.as_str())
            .collect::<Vec<&str>>();

        // Extend the list of object types with the enum names
        let types = model
            .enums
            .iter()
            .map(|enum_| enum_.name.as_str())
            .chain(types.iter().cloned())
            .collect::<Vec<&str>>();

        // Check if there are any objects in the model
        if model.objects.is_empty() {
            error!(
                "[Global] {}: This model has no definitions.",
                "ModelError".bold(),
            );
            self.is_valid = false;
        }

        // Validate the objects and enums
        for object in &model.objects {
            let result = validate_object(object, &types);
            if result.is_err() {
                self.is_valid = false;
            }
        }

        if !self.is_valid {
            Err("Invalid Markdown Model".into())
        } else {
            Ok(())
        }
    }
}

/// Checks for duplicate object names within the model.
///
/// # Arguments
///
/// * `collection` - A slice of `Object` instances to be checked.
fn check_duplicate_objects(collection: &[Object]) -> Result<(), ()> {
    let mut valid = Ok(());
    let unique = collection
        .iter()
        .map(|object| object.name.as_str())
        .collect::<Vec<&str>>();

    // Find duplicates
    let duplicates = unique
        .iter()
        .filter(|&name| unique.iter().filter(|&n| n == name).count() > 1)
        .collect::<Vec<&&str>>();

    let duplicates = unique_elements(&duplicates);

    if !duplicates.is_empty() {
        for name in duplicates {
            error!(
                "[{}] {}: Object {} is defined more than once.",
                "Global".bold(),
                "DuplicateError".bold(),
                name.red().bold(),
            );
        }
        valid = Err(());
    }

    valid
}

/// Checks for duplicate enum names within the model.
///
/// # Arguments
///
/// * `collection` - A slice of `Enumeration` instances to be checked.
fn check_duplicate_enums(collection: &[Enumeration]) -> Result<(), ()> {
    let mut valid = Ok(());
    let unique = collection
        .iter()
        .map(|object| object.name.as_str())
        .collect::<Vec<&str>>();

    // Find duplicates
    let duplicates = unique
        .iter()
        .filter(|&name| unique.iter().filter(|&n| n == name).count() > 1)
        .collect::<Vec<&&str>>();

    let duplicates = unique_elements(&duplicates);

    if !duplicates.is_empty() {
        for name in duplicates {
            error!(
                "[{}] {}: Enumeration {} is defined more than once.",
                "Global".bold(),
                "DuplicateError".bold(),
                name.red().bold(),
            );
        }
        valid = Err(());
    }

    valid
}

/// Returns a list of unique elements from a slice.
fn unique_elements<T: std::cmp::Eq + std::hash::Hash + Clone>(input: &[T]) -> Vec<T> {
    let mut set = HashSet::new();
    input
        .iter()
        .cloned()
        .filter(|item| set.insert(item.clone()))
        .collect()
}

/// Validates a single object within the data model.
///
/// # Arguments
///
/// * `object` - A reference to the `Object` to be validated.
/// * `types` - A slice of type names that are valid within the model.
fn validate_object(object: &Object, types: &[&str]) -> Result<(), ()> {
    let mut valid = Ok(());

    // Check if the object has fields
    if !object.has_attributes() {
        error!(
            "[{}] {}: Type {} is empty and has no properties.",
            object.name.bold(),
            "TypeError".bold(),
            object.name.red().bold(),
        );
        valid = Err(());
    }

    // Validate the attributes of the object
    object.attributes.iter().for_each(|attribute| {
        let result = validate_attribute(attribute, types, &object.name);
        if result.is_err() {
            valid = Err(());
        }
    });

    valid
}

/// Validates a single attribute within an object.
///
/// # Arguments
///
/// * `attribute` - A reference to the `Attribute` to be validated.
/// * `types` - A slice of type names that are valid within the model.
/// * `obj_name` - The name of the object that contains the attribute.
fn validate_attribute(attribute: &Attribute, types: &[&str], obj_name: &str) -> Result<(), ()> {
    // Check if the types given in the attributes
    // are part of the model
    let mut valid = Ok(());

    if attribute.dtypes.is_empty() {
        error!(
            "[{}] {}: Property {} has no type specified.",
            obj_name.bold(),
            "TypeError".bold(),
            attribute.name.red().bold()
        );
        return Err(());
    }

    for dtype in &attribute.dtypes {
        if !types.contains(&dtype.as_str()) && !BASIC_TYPES.contains(&dtype.as_str()) {
            error!(
                "[{}] {}: Type {} of property {} not found. Either define the type or use a base type.",
                obj_name.bold(),
                "TypeError".bold(),
                dtype.red().bold(),
                attribute.name.red().bold(),
            );

            valid = Err(());
        }
    }

    valid
}
