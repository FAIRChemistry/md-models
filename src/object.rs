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

use crate::{attribute::Attribute, markdown::position::Position};
use convert_case::{Case, Casing};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};

#[cfg(feature = "python")]
use pyo3::pyclass;

#[cfg(feature = "wasm")]
use tsify_next::Tsify;

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "python", pyclass(get_all))]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi))]
/// Represents an object with a name, attributes, docstring, and an optional term.
pub struct Object {
    /// Name of the object.
    pub name: String,
    /// List of attributes associated with the object.
    pub attributes: Vec<Attribute>,
    /// Documentation string for the object.
    pub docstring: String,
    /// Optional term associated with the object.
    pub term: Option<String>,
    /// Other objects that this object gets mixed in with.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mixins: Vec<String>,
    /// The line number of the object
    pub position: Option<Position>,
}

impl Object {
    /// Creates a new `Object` with the given name and optional term.
    ///
    /// # Arguments
    ///
    /// * `name` - A string representing the name of the object.
    /// * `term` - An optional string representing a term associated with the object.
    ///
    /// # Returns
    ///
    /// * `Object` - A new instance of the `Object` struct.
    pub fn new(name: String, term: Option<String>) -> Self {
        let name = name.replace(" ", "_").to_case(Case::Pascal);
        Object {
            name,
            attributes: Vec::new(),
            docstring: String::new(),
            term,
            mixins: Vec::new(),
            position: None,
        }
    }

    /// Adds an attribute to the object.
    ///
    /// # Arguments
    ///
    /// * `attribute` - An instance of `Attribute` to be added to the object's attributes.
    pub fn add_attribute(&mut self, attribute: Attribute) {
        self.attributes.push(attribute);
    }

    /// Sets the docstring for the object.
    ///
    /// # Arguments
    ///
    /// * `docstring` - A string representing the documentation string for the object.
    pub fn set_docstring(&mut self, docstring: String) {
        self.docstring = docstring;
    }

    /// Sets the line number of the object.
    ///
    /// # Arguments
    ///
    /// * `position` - The position to set.
    pub fn set_position(&mut self, position: Position) {
        self.position = Some(position);
    }

    /// Retrieves the last attribute added to the object.
    ///
    /// # Returns
    ///
    /// * `&mut Attribute` - A mutable reference to the last attribute.
    ///
    /// # Panics
    ///
    /// This function will panic if there are no attributes in the object.
    pub fn get_last_attribute(&mut self) -> Option<&mut Attribute> {
        self.attributes.last_mut()
    }

    /// Creates and adds a new attribute to the object.
    ///
    /// # Arguments
    ///
    /// * `name` - A string representing the name of the attribute.
    /// * `required` - A boolean indicating whether the attribute is required.
    pub fn create_new_attribute(&mut self, name: String, required: bool) {
        let attribute = Attribute::new(name, required);
        self.attributes.push(attribute);
    }

    /// Checks if the object has any attributes.
    ///
    /// # Returns
    ///
    /// * `bool` - `true` if the object has attributes, `false` otherwise.
    pub fn has_attributes(&self) -> bool {
        !self.attributes.is_empty()
    }

    /// Sets the name of the object.
    ///
    /// # Arguments
    ///
    /// * `name` - A string representing the new name of the object.
    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    /// Checks if any attribute of the object has a term.
    ///
    /// # Returns
    ///
    /// * `bool` - `true` if any attribute has a term, `false` otherwise.
    pub fn has_any_terms(&self) -> bool {
        self.attributes.iter().any(|attr| attr.has_term())
    }

    /// Sorts the attributes of the object by their `required` field in descending order.
    pub fn sort_attrs_by_required(&mut self) {
        let mut top_elements: Vec<Attribute> = vec![];
        let mut bottom_elements: Vec<Attribute> = vec![];

        for attr in self.attributes.iter() {
            if attr.required && attr.default.is_none() && !attr.is_array {
                top_elements.push(attr.clone());
            } else {
                bottom_elements.push(attr.clone());
            }
        }

        self.attributes = top_elements;
        self.attributes.append(&mut bottom_elements);
    }

    /// Checks if this object has the same hash as another object.
    ///
    /// # Arguments
    ///
    /// * `other` - Another `Object` to compare hashes with.
    ///
    /// # Returns
    ///
    /// * `bool` - `true` if both objects have the same hash, `false` otherwise.
    pub(crate) fn same_hash(&self, other: &Object) -> bool {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();
        self.hash(&mut hasher1);
        other.hash(&mut hasher2);
        hasher1.finish() == hasher2.finish()
    }
}

impl Hash for Object {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let mut attr_names: Vec<&String> = self.attributes.iter().map(|attr| &attr.name).collect();
        attr_names.sort();
        attr_names.hash(state);
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[cfg_attr(feature = "python", pyclass(get_all))]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi))]
/// Represents an enumeration with a name and mappings.
pub struct Enumeration {
    /// Name of the enumeration.
    pub name: String,
    /// Mappings associated with the enumeration.
    pub mappings: BTreeMap<String, String>,
    /// Documentation string for the enumeration.
    pub docstring: String,
    /// The line number of the enumeration
    pub position: Option<Position>,
}

impl Enumeration {
    /// Checks if the enumeration has any values.
    ///
    /// # Returns
    ///
    /// * `bool` - `true` if the enumeration has values, `false` otherwise.
    pub fn has_values(&self) -> bool {
        !self.mappings.is_empty()
    }

    /// Sets the position of the enumeration.
    ///
    /// # Arguments
    ///
    /// * `position` - The position to set.
    pub fn set_position(&mut self, position: Position) {
        self.position = Some(position);
    }

    /// Checks if this enumeration has the same hash as another enumeration.
    ///
    /// # Arguments
    ///
    /// * `other` - Another `Enumeration` to compare hashes with.
    ///
    /// # Returns
    ///
    /// * `bool` - `true` if both enumerations have the same hash, `false` otherwise.
    pub(crate) fn same_hash(&self, other: &Enumeration) -> bool {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();
        self.hash(&mut hasher1);
        other.hash(&mut hasher2);
        hasher1.finish() == hasher2.finish()
    }
}

impl Hash for Enumeration {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let mut keys: Vec<&String> = self.mappings.keys().collect();
        keys.sort();
        keys.hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

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
        assert_eq!(last_attribute.unwrap().name, "name");
    }

    #[test]
    fn test_create_new_attribute() {
        let mut object = Object::new("Person".to_string(), None);
        object.create_new_attribute("name".to_string(), false);
        assert_eq!(object.attributes.len(), 1);
        assert_eq!(object.attributes[0].name, "name");
    }

    fn hash<T: Hash>(t: &T) -> u64 {
        let mut s = DefaultHasher::new();
        t.hash(&mut s);
        s.finish()
    }

    #[test]
    fn test_object_hash_identical() {
        let mut object1 = Object::new("Person".to_string(), None);
        object1.create_new_attribute("name".to_string(), false);
        object1.create_new_attribute("age".to_string(), false);

        let mut object2 = Object::new("Person".to_string(), None);
        // Add attributes in different order - should still hash the same
        object2.create_new_attribute("age".to_string(), false);
        object2.create_new_attribute("name".to_string(), false);

        assert_eq!(hash(&object1), hash(&object2));
    }

    #[test]
    fn test_object_hash_different() {
        let mut object1 = Object::new("Person".to_string(), None);
        object1.create_new_attribute("name".to_string(), false);
        object1.create_new_attribute("age".to_string(), false);

        let mut object2 = Object::new("Person".to_string(), None);
        object2.create_new_attribute("name".to_string(), false);
        object2.create_new_attribute("email".to_string(), false);

        assert_ne!(hash(&object1), hash(&object2));
    }

    #[test]
    fn test_enumeration_hash_identical() {
        let mut enum1 = Enumeration::default();
        enum1
            .mappings
            .insert("active".to_string(), "Active".to_string());
        enum1
            .mappings
            .insert("inactive".to_string(), "Inactive".to_string());

        let mut enum2 = Enumeration::default();
        // Add mappings in different order - should still hash the same
        enum2
            .mappings
            .insert("inactive".to_string(), "Inactive".to_string());
        enum2
            .mappings
            .insert("active".to_string(), "Active".to_string());

        assert_eq!(hash(&enum1), hash(&enum2));
    }

    #[test]
    fn test_enumeration_hash_different() {
        let mut enum1 = Enumeration::default();
        enum1
            .mappings
            .insert("active".to_string(), "Active".to_string());
        enum1
            .mappings
            .insert("inactive".to_string(), "Inactive".to_string());

        let mut enum2 = Enumeration::default();
        enum2
            .mappings
            .insert("pending".to_string(), "Pending".to_string());
        enum2
            .mappings
            .insert("active".to_string(), "Active".to_string());

        assert_ne!(hash(&enum1), hash(&enum2));
    }

    #[test]
    fn test_object_hash_reference_identical() {
        let mut object1 = Object::new("Person".to_string(), None);
        object1.create_new_attribute("name".to_string(), false);
        object1.create_new_attribute("age".to_string(), false);

        let mut object2 = Object::new("Person".to_string(), None);
        object2.create_new_attribute("age".to_string(), false);
        object2.create_new_attribute("name".to_string(), false);

        let ref1: &Object = &object1;
        let ref2: &Object = &object2;

        assert_eq!(hash(ref1), hash(ref2));
        assert_eq!(hash(&object1), hash(ref1));
    }

    #[test]
    fn test_object_hash_reference_different() {
        let mut object1 = Object::new("Person".to_string(), None);
        object1.create_new_attribute("name".to_string(), false);
        object1.create_new_attribute("age".to_string(), false);

        let mut object2 = Object::new("Person".to_string(), None);
        object2.create_new_attribute("name".to_string(), false);
        object2.create_new_attribute("email".to_string(), false);

        let ref1: &Object = &object1;
        let ref2: &Object = &object2;

        assert_ne!(hash(ref1), hash(ref2));
        assert_eq!(hash(&object1), hash(ref1));
    }

    #[test]
    fn test_enumeration_hash_reference_identical() {
        let mut enum1 = Enumeration::default();
        enum1
            .mappings
            .insert("active".to_string(), "Active".to_string());
        enum1
            .mappings
            .insert("inactive".to_string(), "Inactive".to_string());

        let mut enum2 = Enumeration::default();
        enum2
            .mappings
            .insert("inactive".to_string(), "Inactive".to_string());
        enum2
            .mappings
            .insert("active".to_string(), "Active".to_string());

        let ref1: &Enumeration = &enum1;
        let ref2: &Enumeration = &enum2;

        assert_eq!(hash(ref1), hash(ref2));
        assert_eq!(hash(&enum1), hash(ref1));
    }

    #[test]
    fn test_enumeration_hash_reference_different() {
        let mut enum1 = Enumeration::default();
        enum1
            .mappings
            .insert("active".to_string(), "Active".to_string());
        enum1
            .mappings
            .insert("inactive".to_string(), "Inactive".to_string());

        let mut enum2 = Enumeration::default();
        enum2
            .mappings
            .insert("pending".to_string(), "Pending".to_string());
        enum2
            .mappings
            .insert("active".to_string(), "Active".to_string());

        let ref1: &Enumeration = &enum1;
        let ref2: &Enumeration = &enum2;

        assert_ne!(hash(ref1), hash(ref2));
        assert_eq!(hash(&enum1), hash(ref1));
    }

    #[test]
    fn test_object_has_same_hash_identical() {
        let mut object1 = Object::new("Person".to_string(), None);
        object1.create_new_attribute("name".to_string(), false);
        object1.create_new_attribute("age".to_string(), false);

        let mut object2 = Object::new("Person".to_string(), None);
        // Add attributes in different order - should still hash the same
        object2.create_new_attribute("age".to_string(), false);
        object2.create_new_attribute("name".to_string(), false);

        assert!(object1.same_hash(&object2));
    }

    #[test]
    fn test_object_has_same_hash_different() {
        let mut object1 = Object::new("Person".to_string(), None);
        object1.create_new_attribute("name".to_string(), false);
        object1.create_new_attribute("age".to_string(), false);

        let mut object2 = Object::new("Person".to_string(), None);
        object2.create_new_attribute("name".to_string(), false);
        object2.create_new_attribute("email".to_string(), false);

        assert!(!object1.same_hash(&object2));
    }

    #[test]
    fn test_enumeration_has_same_hash_identical() {
        let mut enum1 = Enumeration::default();
        enum1
            .mappings
            .insert("active".to_string(), "Active".to_string());
        enum1
            .mappings
            .insert("inactive".to_string(), "Inactive".to_string());

        let mut enum2 = Enumeration::default();
        // Add mappings in different order - should still hash the same
        enum2
            .mappings
            .insert("inactive".to_string(), "Inactive".to_string());
        enum2
            .mappings
            .insert("active".to_string(), "Active".to_string());

        assert!(enum1.same_hash(&enum2));
    }

    #[test]
    fn test_enumeration_has_same_hash_different() {
        let mut enum1 = Enumeration::default();
        enum1
            .mappings
            .insert("active".to_string(), "Active".to_string());
        enum1
            .mappings
            .insert("inactive".to_string(), "Inactive".to_string());

        let mut enum2 = Enumeration::default();
        enum2
            .mappings
            .insert("pending".to_string(), "Pending".to_string());
        enum2
            .mappings
            .insert("active".to_string(), "Active".to_string());

        assert!(!enum1.same_hash(&enum2));
    }
}
