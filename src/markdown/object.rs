use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::markdown::attribute::Attribute;

#[derive(Serialize, Deserialize, Debug)]
pub enum ObjectType {
    Object,
    Enum,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug)]
pub struct Object {
    pub name: String,
    pub attributes: Vec<Attribute>,
    pub docstring: String,
    pub object_type: ObjectType,
    pub term: Option<String>,
}

impl Object {
    pub fn new(name: String, object_type: ObjectType, term: Option<String>) -> Self {
        Object {
            name,
            attributes: Vec::new(),
            docstring: String::new(),
            object_type,
            term,
        }
    }

    pub fn add_attribute(&mut self, attribute: Attribute) {
        self.attributes.push(attribute);
    }

    pub fn set_docstring(&mut self, docstring: String) {
        self.docstring = docstring;
    }

    pub fn get_last_attribute(&mut self) -> &mut Attribute {
        self.attributes.last_mut().unwrap()
    }

    pub fn create_new_attribute(&mut self, name: String) {
        let attribute = Attribute::new(name);
        self.attributes.push(attribute);
    }
}
