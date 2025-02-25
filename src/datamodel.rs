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

use std::collections::HashMap;
use std::path::PathBuf;
use std::{error::Error, fs, path::Path};

use log::error;
use serde::{Deserialize, Serialize};

use crate::exporters::{render_jinja_template, Templates};
use crate::json::export::to_json_schema;
use crate::json::validation::{validate_json, ValidationError};
use crate::linkml::export::serialize_linkml;
use crate::markdown::frontmatter::FrontMatter;
use crate::markdown::parser::parse_markdown;
use crate::object::{Enumeration, Object};
use crate::validation::Validator;
use colored::Colorize;

#[cfg(feature = "python")]
use pyo3::pyclass;

#[cfg(feature = "wasm")]
use tsify_next::Tsify;

// Data model
//
// Contains a list of objects that represent the data model
// written in the markdown format
//
// # Examples
//
// ```
// let model = DataModel::new();
// ```
//
// # Fields
//
// * `objects` - A list of objects
//
// # Methods
//
// * `new` - Create a new data model
// * `parse` - Parse a markdown file and create a data model
// * `json_schema` - Generate a JSON schema from the data model
// * `json_schema_all` - Generate JSON schemas for all objects in the data model
// * `internal_schema` - Generate an internal schema from the data model
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[cfg_attr(feature = "python", pyclass(get_all))]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi))]
pub struct DataModel {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub objects: Vec<Object>,
    pub enums: Vec<Enumeration>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<FrontMatter>,
}

impl DataModel {
    pub fn new(name: Option<String>, config: Option<FrontMatter>) -> Self {
        DataModel {
            name,
            objects: Vec::new(),
            enums: Vec::new(),
            config,
        }
    }

    /// Validates a dataset against the data model.
    ///
    /// This function takes the path to a dataset and validates it against the
    /// current data model. It returns a vector of validation errors if any
    /// validation issues are found, or an empty vector if the validation is successful.
    ///
    /// # Arguments
    ///
    /// * `path` - A reference to the path of the dataset to validate.
    /// * `root` - An optional root path for the schema. Will use the first object if not provided.
    ///
    /// # Returns
    /// A Result containing a vector of `ValidationError` if validation fails,
    /// or an empty vector if successful.
    pub fn validate_json(
        &self,
        path: &Path,
        root: Option<String>,
    ) -> Result<Vec<ValidationError>, Box<dyn Error>> {
        validate_json(path.to_path_buf(), self, root)
    }

    // Get the JSON schema for an object
    //
    // * `obj_name` - Name of the object
    // * `openai` - Whether to remove options from the schema properties. OpenAI does not support options.
    //
    // # Panics
    // If no objects are found in the markdown file
    // If the object is not found in the markdown file
    //
    // # Examples
    //
    // ```
    // let model = DataModel::new();
    // model.parse("path/to/file.md".to_string());
    // let schema = model.json_schema("object_name".to_string());
    // ```
    //
    // # Returns
    //
    // A JSON schema string
    pub fn json_schema(
        &self,
        obj_name: Option<String>,
        openai: bool,
    ) -> Result<String, Box<dyn Error>> {
        if self.objects.is_empty() {
            panic!("No objects found in the markdown file");
        }

        match obj_name {
            Some(name) => {
                if self.objects.iter().all(|o| o.name != name) {
                    panic!("Object '{}' not found in the markdown file", name);
                }
                Ok(serde_json::to_string_pretty(&to_json_schema(
                    self, &name, openai,
                )?)?)
            }
            None => Ok(serde_json::to_string_pretty(&to_json_schema(
                self,
                &self.objects[0].name,
                openai,
            )?)?),
        }
    }

    // Get the JSON schema for all objects in the markdown file
    // and write them to a file
    //
    // * `path` - Path to the directory where the JSON schema files will be written
    // * `openai` - Whether to remove options from the schema properties. OpenAI does not support options.
    //
    // # Panics
    //
    // If no objects are found in the markdown file
    //
    // # Examples
    //
    // ```
    // let model = DataModel::new();
    // model.parse("path/to/file.md".to_string());
    // model.json_schema_all("path/to/directory".to_string());
    // ```
    pub fn json_schema_all(&self, path: PathBuf, openai: bool) -> Result<(), Box<dyn Error>> {
        if self.objects.is_empty() {
            panic!("No objects found in the markdown file");
        }

        // Create the directory if it does not exist
        if !std::path::Path::new(&path).exists() {
            fs::create_dir_all(&path).expect("Could not create directory");
        }

        let base_path = path.to_str().ok_or("Failed to convert path to string")?;
        for object in &self.objects {
            let schema = to_json_schema(self, &object.name, openai)?;
            let file_name = format!("{}/{}.json", base_path, object.name);
            fs::write(file_name, serde_json::to_string_pretty(&schema)?)
                .expect("Could not write file");
        }

        Ok(())
    }

    // Get the internal schema for the markdown file
    //
    // # Panics
    //
    // If no objects are found in the markdown file
    //
    // # Examples
    //
    // ```
    // let model = DataModel::new();
    // model.parse("path/to/file.md".to_string());
    // let schema = model.internal_schema();
    // ```
    //
    // # Returns
    //
    // An internal schema string
    pub fn internal_schema(&self) -> String {
        if self.objects.is_empty() {
            panic!("No objects found in the markdown file");
        }

        serde_json::to_string_pretty(&self).expect("Could not serialize to internal schema")
    }

    // Parse a markdown file and create a data model
    //
    // * `path` - Path to the markdown file
    //
    // # Examples
    //
    // ```
    // let path = Path::new("path/to/file.md");
    // let model = DataModel::from_internal_schema(path);
    // ```
    //
    // # Returns
    //
    // A data model
    //
    pub fn from_internal_schema(path: &Path) -> Result<Self, Box<dyn Error>> {
        if !path.exists() {
            return Err("File does not exist".into());
        }

        let contents = fs::read_to_string(path)?;
        let model: DataModel = serde_json::from_str(&contents)?;

        Ok(model)
    }

    /// Sort the attributes of all objects by required
    pub fn sort_attrs(&mut self) {
        for obj in &mut self.objects {
            obj.sort_attrs_by_required();
        }
    }

    // Convert the data model to a template using Jinja
    //
    // * `template` - The Jinja template
    //
    // # Returns
    //
    // A string containing the Jinja template
    //
    // # Errors
    //
    // If the Jinja template is invalid
    //
    pub fn convert_to(
        &mut self,
        template: &Templates,
        config: Option<&HashMap<String, String>>,
    ) -> Result<String, minijinja::Error> {
        self.sort_attrs();

        match template {
            Templates::JsonSchema => Ok(self.json_schema(None, false).unwrap()),
            Templates::Linkml => Ok(serialize_linkml(self.clone(), None).unwrap()),
            _ => render_jinja_template(template, self, config),
        }
    }

    // Merge two data models
    //
    // * `other` - The other data model to merge
    pub fn merge(&mut self, other: &Self) {
        // Initialize a variable to check if the merge is valid
        let mut valid = true;

        // Check if there are any duplicate objects or enums
        for obj in &other.objects {
            if self.objects.iter().any(|o| o.name == obj.name) {
                error!(
                    "[{}] {}: Object {} is defined more than once.",
                    "Merge".bold(),
                    "DuplicateError".bold(),
                    obj.name.red().bold(),
                );

                valid = false;
            }
        }

        for enm in &other.enums {
            if self.enums.iter().any(|e| e.name == enm.name) {
                error!(
                    "[{}] {}: Enumeration {} is defined more than once.",
                    "Merge".bold(),
                    "DuplicateError".bold(),
                    enm.name.red().bold(),
                );

                valid = false;
            }
        }

        // If the merge is not valid, panic
        if !valid {
            panic!("Merge is not valid");
        }

        // Merge the objects and enums
        self.objects.extend(other.objects.clone());
        self.enums.extend(other.enums.clone());
    }

    /// Parse a markdown file and create a data model
    ///
    /// * `path` - Path to the markdown file
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::Path;
    /// use mdmodels::datamodel::DataModel;
    ///
    /// let path = Path::new("tests/data/model.md");
    /// let model = DataModel::from_markdown(path);
    /// ```
    /// # Returns
    /// A data model
    #[allow(clippy::result_large_err)]
    pub fn from_markdown(path: &Path) -> Result<Self, Validator> {
        let content = fs::read_to_string(path).expect("Could not read file");
        parse_markdown(&content, Some(path))
    }

    /// Parse a markdown file and create a data model
    ///
    /// * `path` - Path to the markdown file
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::Path;
    /// use std::fs;
    /// use mdmodels::datamodel::DataModel;
    ///
    /// let path = Path::new("tests/data/model.md");
    /// let content = fs::read_to_string(path).unwrap();
    /// let model = DataModel::from_markdown_string(content.as_str());
    /// ```
    /// # Returns
    /// A data model
    #[allow(clippy::result_large_err)]
    pub fn from_markdown_string(content: &str) -> Result<Self, Validator> {
        parse_markdown(content, None)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::attribute::DataType;

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_merge() {
        // Arrange
        let mut model1 = DataModel::new(None, None);
        let mut model2 = DataModel::new(None, None);

        let mut obj1 = Object::new("Object1".to_string(), None);
        obj1.add_attribute(crate::attribute::Attribute {
            name: "test1".to_string(),
            is_array: false,
            is_id: false,
            dtypes: vec!["string".to_string()],
            docstring: "".to_string(),
            options: vec![],
            term: None,
            required: false,
            xml: None,
            default: None,
            is_enum: false,
            position: None,
            import_prefix: None,
        });

        let mut obj2 = Object::new("Object2".to_string(), None);
        obj2.add_attribute(crate::attribute::Attribute {
            name: "test2".to_string(),
            is_array: false,
            is_id: false,
            dtypes: vec!["string".to_string()],
            docstring: "".to_string(),
            options: vec![],
            term: None,
            required: false,
            xml: None,
            default: None,
            is_enum: false,
            position: None,
            import_prefix: None,
        });

        let enm1 = Enumeration {
            name: "Enum1".to_string(),
            mappings: BTreeMap::from([("key1".to_string(), "value1".to_string())]),
            docstring: "".to_string(),
            position: None,
        };

        let enm2 = Enumeration {
            name: "Enum2".to_string(),
            mappings: BTreeMap::from([("key2".to_string(), "value2".to_string())]),
            docstring: "".to_string(),
            position: None,
        };

        model1.objects.push(obj1);
        model1.enums.push(enm1);
        model2.objects.push(obj2);
        model2.enums.push(enm2);

        // Act
        model1.merge(&model2);

        // Assert
        assert_eq!(model1.objects.len(), 2);
        assert_eq!(model1.enums.len(), 2);
        assert_eq!(model1.objects[0].name, "Object1");
        assert_eq!(model1.objects[1].name, "Object2");
        assert_eq!(model1.enums[0].name, "Enum1");
        assert_eq!(model1.enums[1].name, "Enum2");
    }

    #[test]
    fn test_sort_attrs() {
        // Arrange
        let mut model = DataModel::new(None, None);
        let mut obj = Object::new("Object1".to_string(), None);
        obj.add_attribute(crate::attribute::Attribute {
            name: "not_required".to_string(),
            is_array: false,
            is_id: false,
            dtypes: vec!["string".to_string()],
            docstring: "".to_string(),
            options: vec![],
            term: None,
            required: false,
            xml: None,
            default: Some(DataType::String("".to_string())),
            is_enum: false,
            position: None,
            import_prefix: None,
        });

        obj.add_attribute(crate::attribute::Attribute {
            name: "required".to_string(),
            is_array: false,
            is_id: false,
            dtypes: vec!["string".to_string()],
            docstring: "".to_string(),
            options: vec![],
            term: None,
            required: true,
            xml: None,
            default: None,
            is_enum: false,
            position: None,
            import_prefix: None,
        });

        model.objects.push(obj);

        // Act
        model.sort_attrs();

        // Assert
        assert_eq!(model.objects[0].attributes[0].name, "required");
        assert_eq!(model.objects[0].attributes[1].name, "not_required");
    }

    #[test]
    fn test_from_internal_schema() {
        // Arrange
        let path = Path::new("tests/data/expected_internal_schema.json");

        // Act
        let model = DataModel::from_internal_schema(path).expect("Failed to parse internal schema");

        // Assert
        assert_eq!(model.objects.len(), 2);
        assert_eq!(model.enums.len(), 1);
    }

    #[test]
    fn test_from_markdown_w_html() {
        // Arrange
        let path = Path::new("tests/data/model_w_html.md");

        // Act
        let model = DataModel::from_markdown(path).expect("Failed to parse markdown");

        // Assert
        assert_eq!(model.objects.len(), 2);
        assert_eq!(model.enums.len(), 1);
    }

    #[test]
    fn test_from_markdown_string() {
        // Arrange
        let path = Path::new("tests/data/model.md");
        let content = fs::read_to_string(path).unwrap();

        // Act
        let model =
            DataModel::from_markdown_string(content.as_str()).expect("Failed to parse markdown");

        // Assert
        assert_eq!(model.objects.len(), 2);
        assert_eq!(model.enums.len(), 1);
    }
}
