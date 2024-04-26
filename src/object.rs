use serde::{Deserialize, Serialize};

use crate::attribute::Attribute;

#[derive(Serialize, Deserialize)]
pub enum ObjectType {
    Object,
    Enum,
}

#[derive(Serialize, Deserialize)]
pub struct Object {
    pub name: String,
    pub attributes: Vec<Attribute>,
    pub docstring: String,
    pub object_type: ObjectType,
}

impl Object {
    pub fn new(name: String, object_type: ObjectType) -> Self {
        Object {
            name,
            attributes: Vec::new(),
            docstring: String::new(),
            object_type,
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
