use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::xmltype::XMLType;

/// Represents an attribute with various properties and options.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Attribute {
    /// The name of the attribute.
    pub name: String,
    /// Indicates if the attribute is an array.
    #[serde(rename = "multiple")]
    pub is_array: bool,
    /// Is an identifier or not
    pub is_id: bool,
    /// Data types associated with the attribute.
    pub dtypes: Vec<String>,
    /// Documentation string for the attribute.
    pub docstring: String,
    /// List of additional options for the attribute.
    pub options: Vec<AttrOption>,
    /// Term associated with the attribute, if any.
    pub term: Option<String>,
    /// Indicates if the attribute is required.
    pub required: bool,
    /// XML type information for the attribute.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub xml: Option<XMLType>,
}

impl Attribute {
    /// Creates a new `Attribute` with the given name and required status.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the attribute.
    /// * `required` - Indicates if the attribute is required.
    pub fn new(name: String, required: bool) -> Self {
        Attribute {
            name: name.clone(),
            dtypes: Vec::new(),
            docstring: String::new(),
            options: Vec::new(),
            is_array: false,
            is_id: false,
            term: None,
            required,
            xml: Some(XMLType::from_str(name.as_str()).unwrap()),
        }
    }

    /// Sets the documentation string for the attribute.
    ///
    /// # Arguments
    ///
    /// * `docstring` - The documentation string to set.
    pub fn set_docstring(&mut self, docstring: String) {
        self.docstring = docstring;
    }

    /// Adds an option to the attribute.
    ///
    /// # Arguments
    ///
    /// * `option` - The option to add.
    pub fn add_option(&mut self, option: AttrOption) {
        match option.key.to_lowercase().as_str() {
            "type" => self.set_dtype(option.value),
            "term" => self.term = Some(option.value),
            "description" => self.docstring = option.value,
            "xml" => self.set_xml(XMLType::from_str(&option.value).expect("Invalid XML type")),
            _ => self.options.push(option),
        }
    }

    /// Sets the data type for the attribute.
    ///
    /// # Arguments
    ///
    /// * `dtype` - The data type to set.
    fn set_dtype(&mut self, dtype: String) {
        let mut dtype = dtype;
        // Handle special case for identifiers
        if dtype.to_lowercase().starts_with("identifier") {
            self.is_id = true;
            // Regex replace identifier or Identifier with string
            let pattern = regex::Regex::new(r"[I|i]dentifier").unwrap();
            dtype = pattern.replace_all(&dtype, "string").to_string();
        }

        // Handle special case for arrays
        if dtype.ends_with("[]") {
            self.is_array = true;
        }

        self.dtypes.push(dtype.trim_end_matches("[]").to_string());
    }

    /// Converts the attribute to a JSON schema.
    ///
    /// # Returns
    ///
    /// A JSON string representing the attribute schema.
    pub fn to_json_schema(&self) -> String {
        serde_json::to_string_pretty(&self).unwrap()
    }

    /// Checks if the attribute has an associated term.
    ///
    /// # Returns
    ///
    /// `true` if the attribute has a term, `false` otherwise.
    pub fn has_term(&self) -> bool {
        self.term.is_some()
    }

    /// Sets the XML type for the attribute.
    ///
    /// # Arguments
    ///
    /// * `xml` - The XML type to set.
    pub fn set_xml(&mut self, xml: XMLType) {
        self.xml = Some(xml);
    }
}

/// Represents an option for an attribute.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AttrOption {
    /// The key of the option.
    pub key: String,
    /// The value of the option.
    pub value: String,
}

impl AttrOption {
    /// Creates a new `AttrOption` with the given key and value.
    ///
    /// # Arguments
    ///
    /// * `key` - The key of the option.
    /// * `value` - The value of the option.
    pub fn new(key: String, value: String) -> Self {
        Self {
            key: key.to_lowercase(),
            value,
        }
    }

    /// Gets the key of the option.
    ///
    /// # Returns
    ///
    /// A reference to the key.
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Gets the value of the option.
    ///
    /// # Returns
    ///
    /// A reference to the value.
    pub fn value(&self) -> &str {
        &self.value
    }
}

#[cfg(test)]
mod tests {
    use crate::xmltype::XMLType;
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_attribute_new() {
        let attr = Attribute::new("name".to_string(), false);
        assert_eq!(attr.name, "name");
        assert_eq!(attr.dtypes.len(), 0);
        assert_eq!(attr.docstring, "");
        assert_eq!(attr.options.len(), 0);
        assert_eq!(attr.is_array, false);
        assert_eq!(attr.term, None);
        assert_eq!(attr.required, false);
    }

    #[test]
    fn test_attribute_set_docstring() {
        let mut attr = Attribute::new("name".to_string(), false);
        attr.set_docstring("This is a test".to_string());
        assert_eq!(attr.docstring, "This is a test");
        assert_eq!(attr.required, false);
    }

    #[test]
    fn test_attribute_add_type_option() {
        let mut attr = Attribute::new("name".to_string(), false);
        let option = AttrOption::new("type".to_string(), "string".to_string());
        attr.add_option(option);
        assert_eq!(attr.dtypes.len(), 1);
        assert_eq!(attr.dtypes[0], "string");
    }

    #[test]
    fn test_attribute_add_term_option() {
        let mut attr = Attribute::new("name".to_string(), false);
        let option = AttrOption::new("term".to_string(), "string".to_string());
        attr.add_option(option);
        assert_eq!(attr.term, Some("string".to_string()));
    }

    #[test]
    fn test_attribute_add_option() {
        let mut attr = Attribute::new("name".to_string(), false);
        let option = AttrOption::new("description".to_string(), "This is a test".to_string());
        attr.add_option(option);
        let option = AttrOption::new("something".to_string(), "something".to_string());
        attr.add_option(option);

        assert_eq!(attr.options.len(), 1);
        assert_eq!(attr.options[0].key, "something");
        assert_eq!(attr.options[0].value, "something");
        assert_eq!(attr.docstring, "This is a test");
    }

    #[test]
    fn test_attribute_set_dtype() {
        let mut attr = Attribute::new("name".to_string(), false);
        attr.set_dtype("string".to_string());
        assert_eq!(attr.dtypes.len(), 1);
        assert_eq!(attr.dtypes[0], "string");
        assert_eq!(attr.is_array, false);
    }

    #[test]
    fn test_attribute_set_array_dtype() {
        let mut attr = Attribute::new("name".to_string(), false);
        attr.set_dtype("string[]".to_string());
        assert_eq!(attr.dtypes.len(), 1);
        assert_eq!(attr.dtypes[0], "string");
        assert_eq!(attr.is_array, true);
    }

    #[test]
    fn test_attribute_set_xml_attr() {
        let mut attr = Attribute::new("name".to_string(), false);
        let xml = XMLType::from_str("@name").expect("Could not parse XMLType");
        attr.set_xml(xml);
        assert_eq!(
            attr.xml.expect("Could not find XML option"),
            XMLType::Attribute {
                is_attr: true,
                name: "name".to_string()
            },
            "XMLType is not correct. Expected an attribute type."
        );
    }

    #[test]
    fn test_attribute_set_xml_element() {
        let mut attr = Attribute::new("name".to_string(), false);
        let xml = XMLType::from_str("name").expect("Could not parse XMLType");
        attr.set_xml(xml);
        assert_eq!(
            attr.xml.expect("Could not find XML option"),
            XMLType::Element {
                is_attr: false,
                name: "name".to_string()
            },
            "XMLType is not correct. Expected an element type."
        );
    }

    #[test]
    fn test_default_xml_type() {
        let attr = Attribute::new("name".to_string(), false);
        assert_eq!(
            attr.xml.unwrap(),
            XMLType::Element {
                is_attr: false,
                name: "name".to_string()
            }
        );
    }
}
