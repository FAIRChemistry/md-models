use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::attribute::Attribute;

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Object {
    pub name: String,
    pub attributes: Vec<Attribute>,
    pub docstring: String,
    pub term: Option<String>,
}

impl Object {
    pub fn new(name: String, term: Option<String>) -> Self {
        Object {
            name,
            attributes: Vec::new(),
            docstring: String::new(),
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
        !self.attributes.is_empty()
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn has_any_terms(&self) -> bool {
        self.attributes.iter().any(|attr| attr.has_term())
    }

    pub fn sort_attrs_by_required(&mut self) {
        self.attributes.sort_by(|a, b| b.required.cmp(&a.required))
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Enumeration {
    pub name: String,
    pub mappings: BTreeMap<String, String>,
}

impl Enumeration {
    pub fn has_values(&self) -> bool {
        !self.mappings.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_create_new_object() {
        let object = Object::new("Person".to_string(), None);
        assert_eq!(object.name, "Person");
        assert_eq!(object.attributes.len(), 0);
        assert_eq!(object.docstring, "");
        assert_eq!(object.term, None);
    }

    #[test]
    fn test_add_attribute() {
        let mut object = Object::new("Person".to_string(), None);
        let attribute = Attribute::new("name".to_string(), false);
        object.add_attribute(attribute);
        assert_eq!(object.attributes.len(), 1);
        assert_eq!(object.attributes[0].name, "name");
    }

    #[test]
    fn test_set_docstring() {
        let mut object = Object::new("Person".to_string(), None);
        object.set_docstring("This is a person object".to_string());
        assert_eq!(object.docstring, "This is a person object");
    }

    #[test]
    fn test_get_last_attribute() {
        let mut object = Object::new("Person".to_string(), None);
        let attribute = Attribute::new("name".to_string(), false);
        object.add_attribute(attribute);
        let last_attribute = object.get_last_attribute();
        assert_eq!(last_attribute.name, "name");
    }

    #[test]
    fn test_create_new_attribute() {
        let mut object = Object::new("Person".to_string(), None);
        object.create_new_attribute("name".to_string(), false);
        assert_eq!(object.attributes.len(), 1);
        assert_eq!(object.attributes[0].name, "name");
    }
}
