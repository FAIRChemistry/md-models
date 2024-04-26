use pulldown_cmark::Parser;
use std::fs;

pub mod attribute;
pub mod object;
pub mod parser;
pub mod primitives;
pub mod schema;

//! Data model
//!
//! Contains a list of objects that represent the data model
//! written in the markdown format
//!
//! # Examples
//!
//! ```
//! let model = DataModel::new();
//! ```
//!
//! # Fields
//!
//! * `objects` - A list of objects
//!
//! # Methods
//!
//! * `new` - Create a new data model
//! * `parse` - Parse a markdown file and create a data model
//! * `json_schema` - Generate a JSON schema from the data model
//! * `json_schema_all` - Generate JSON schemas for all objects in the data model
//! * `sdrdm_schema` - Generate a SDRDM schema from the data model
pub struct DataModel {
    pub objects: Vec<object::Object>,
}

impl DataModel {
    pub fn new() -> Self {
        DataModel {
            objects: Vec::new(),
        }
    }

    //! Parse a markdown file and create a data model
    //!
    //! * `path` - Path to the markdown file
    //!
    //! # Panics
    //!
    //! If the file does not exist
    //!
    //! # Examples
    //!
    //! ```
    //! let model = DataModel::new();
    //! model.parse("path/to/file.md".to_string());
    //! ```
    //!
    //! # Returns
    //!
    //! A data model
    pub fn parse(path: String) -> Self {
        if !std::path::Path::new(&path).exists() {
            panic!("File does not exist");
        }

        let mut model = DataModel::new();
        let content = fs::read_to_string(path).expect("Could not read file");
        let parser = Parser::new(&content);
        let mut iterator = parser.into_iter();
        let mut objects = Vec::new();

        while let Some(event) = iterator.next() {
            parser::process_event(&mut iterator, &mut objects, event);
        }

        model.objects = objects;

        return model;
    }

    //! Get the JSON schema for an object
    //!
    //! * `obj_name` - Name of the object
    //!
    //! # Panics
    //!
    //! If no objects are found in the markdown file
    //! If the object is not found in the markdown file
    //!
    //! # Examples
    //!
    //! ```
    //! let model = DataModel::new();
    //! model.parse("path/to/file.md".to_string());
    //! let schema = model.json_schema("object_name".to_string());
    //! ```
    //!
    //! # Returns
    //!
    //! A JSON schema string
    pub fn json_schema(&self, obj_name: String) -> String {
        if self.objects.len() == 0 {
            panic!("No objects found in the markdown file");
        }

        if self.objects.iter().all(|o| o.name != obj_name) {
            panic!("Object not found in the markdown file");
        }

        return schema::to_json_schema(&obj_name, &self.objects);
    }

    //! Get the JSON schema for all objects in the markdown file
    //! and write them to a file
    //!
    //! * `path` - Path to the directory where the JSON schema files will be written
    //!
    //! # Panics
    //!
    //! If no objects are found in the markdown file
    //!
    //! # Examples
    //!
    //! ```
    //! let model = DataModel::new();
    //! model.parse("path/to/file.md".to_string());
    //! model.json_schema_all("path/to/directory".to_string());
    //! ```
    pub fn json_schema_all(&self, path: String) {
        if self.objects.len() == 0 {
            panic!("No objects found in the markdown file");
        }

        //! Create the directory if it does not exist
        if !std::path::Path::new(&path).exists() {
            fs::create_dir_all(&path).expect("Could not create directory");
        }

        for object in &self.objects {
            let schema = schema::to_json_schema(&object.name, &self.objects);
            let file_name = format!("{}/{}.json", path, object.name);
            fs::write(file_name, schema).expect("Could not write file");
        }
    }

    //! Get the SDRDM schema for the markdown file
    //!
    //! # Panics
    //!
    //! If no objects are found in the markdown file
    //!
    //! # Examples
    //!
    //! ```
    //! let model = DataModel::new();
    //! model.parse("path/to/file.md".to_string());
    //! let schema = model.sdrdm_schema();
    //! ```
    //!
    //! # Returns
    //!
    //! A SDRDM schema string
    pub fn sdrdm_schema(&self) -> String {
        if self.objects.len() == 0 {
            panic!("No objects found in the markdown file");
        }

        return serde_json::to_string_pretty(&self.objects).unwrap();
    }
}
