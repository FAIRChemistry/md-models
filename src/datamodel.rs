use std::{error::Error, fs, path::Path};

use serde::{Deserialize, Serialize};

use crate::exporters::{render_jinja_template, Templates};
use crate::markdown::frontmatter::FrontMatter;
use crate::object::{Enumeration, Object};
use crate::{markdown, schema};

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
// * `sdrdm_schema` - Generate a SDRDM schema from the data model
#[derive(Debug, Serialize, Deserialize)]
pub struct DataModel {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub objects: Vec<Object>,
    pub enums: Vec<Enumeration>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<markdown::frontmatter::FrontMatter>,
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

    // Get the JSON schema for an object
    //
    // * `obj_name` - Name of the object
    //
    // # Panics
    //
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
    pub fn json_schema(&self, obj_name: String) -> String {
        if self.objects.len() == 0 {
            panic!("No objects found in the markdown file");
        }

        if self.objects.iter().all(|o| o.name != obj_name) {
            panic!("Object not found in the markdown file");
        }

        return schema::to_json_schema(&obj_name, &self);
    }

    // Get the JSON schema for all objects in the markdown file
    // and write them to a file
    //
    // * `path` - Path to the directory where the JSON schema files will be written
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
    pub fn json_schema_all(&self, path: String) {
        if self.objects.len() == 0 {
            panic!("No objects found in the markdown file");
        }

        // Create the directory if it does not exist
        if !std::path::Path::new(&path).exists() {
            fs::create_dir_all(&path).expect("Could not create directory");
        }

        for object in &self.objects {
            let schema = schema::to_json_schema(&object.name, &self);
            let file_name = format!("{}/{}.json", path, object.name);
            fs::write(file_name, schema).expect("Could not write file");
        }
    }

    // Get the SDRDM schema for the markdown file
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
    // let schema = model.sdrdm_schema();
    // ```
    //
    // # Returns
    //
    // A SDRDM schema string
    pub fn sdrdm_schema(&self) -> String {
        if self.objects.len() == 0 {
            panic!("No objects found in the markdown file");
        }

        return serde_json::to_string_pretty(&self).unwrap();
    }

    // Parse a markdown file and create a data model
    //
    // * `path` - Path to the markdown file
    //
    // # Examples
    //
    // ```
    // let path = Path::new("path/to/file.md");
    // let model = DataModel::from_sdrdm_schema(path);
    // ```
    //
    // # Returns
    //
    // A data model
    //
    pub fn from_sdrdm_schema(path: &Path) -> Result<Self, Box<dyn Error>> {
        if !path.exists() {
            return Err("File does not exist".into());
        }

        let contents = fs::read_to_string(path)?;
        let model: DataModel = serde_json::from_str(&contents)?;

        Ok(model)
    }

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
    pub fn convert_to(&mut self, template: &Templates) -> Result<String, minijinja::Error> {
        self.sort_attrs();
        render_jinja_template(template, self)
    }
}
