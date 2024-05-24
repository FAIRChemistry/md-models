use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::attribute::Attribute;

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

    pub fn create_new_attribute(&mut self, name: String, required: bool) {
        let attribute = Attribute::new(name, required);
        self.attributes.push(attribute);
    }

    pub fn has_attributes(&self) -> bool {
        self.attributes.len() > 0
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_new_object() {
        let object = Object::new("Person".to_string(), ObjectType::Object, None);
        assert_eq!(object.name, "Person");
        assert_eq!(object.attributes.len(), 0);
        assert_eq!(object.docstring, "");
        assert_eq!(object.term, None);
    }

    #[test]
    fn test_add_attribute() {
        let mut object = Object::new("Person".to_string(), ObjectType::Object, None);
        let attribute = Attribute::new("name".to_string(), false);
        object.add_attribute(attribute);
        assert_eq!(object.attributes.len(), 1);
        assert_eq!(object.attributes[0].name, "name");
    }

    #[test]
    fn test_set_docstring() {
        let mut object = Object::new("Person".to_string(), ObjectType::Object, None);
        object.set_docstring("This is a person object".to_string());
        assert_eq!(object.docstring, "This is a person object");
    }

    #[test]
    fn test_get_last_attribute() {
        let mut object = Object::new("Person".to_string(), ObjectType::Object, None);
        let attribute = Attribute::new("name".to_string(), false);
        object.add_attribute(attribute);
        let last_attribute = object.get_last_attribute();
        assert_eq!(last_attribute.name, "name");
    }

    #[test]
    fn test_create_new_attribute() {
        let mut object = Object::new("Person".to_string(), ObjectType::Object, None);
        object.create_new_attribute("name".to_string(), false);
        assert_eq!(object.attributes.len(), 1);
        assert_eq!(object.attributes[0].name, "name");
    }
}
