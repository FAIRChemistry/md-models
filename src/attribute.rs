use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Attribute {
    pub name: String,
    #[serde(rename = "multiple")]
    pub is_array: bool,
    pub dtypes: Vec<String>,
    pub docstring: String,
    pub options: Vec<AttrOption>,
    pub term: Option<String>,
    pub required: bool,
}

impl Attribute {
    pub fn new(name: String, required: bool) -> Self {
        Attribute {
            name,
            dtypes: Vec::new(),
            docstring: String::new(),
            options: Vec::new(),
            is_array: false,
            term: None,
            required,
        }
    }

    pub fn set_docstring(&mut self, docstring: String) {
        self.docstring = docstring;
    }

    pub fn add_option(&mut self, option: AttrOption) {
        if option.key.to_lowercase() == "type" {
            self.set_dtype(option.value);
            return;
        } else if option.key.to_lowercase() == "term" {
            self.term = Some(option.value);
            return;
        }

        self.options.push(option);
    }

    fn set_dtype(&mut self, dtype: String) {
        if dtype.ends_with("[]") {
            self.is_array = true;
            self.dtypes.push(dtype.trim_end_matches("[]").to_string());
            return;
        }

        self.dtypes.push(dtype);
    }

    pub fn to_json_schema(&self) -> String {
        serde_json::to_string_pretty(&self).unwrap()
    }

    pub fn has_term(&self) -> bool {
        self.term.is_some()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AttrOption {
    key: String,
    value: String,
}

impl AttrOption {
    pub fn new(key: String, value: String) -> Self {
        Self {
            key: key.to_lowercase(),
            value,
        }
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}

#[cfg(test)]
mod tests {
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
        assert_eq!(attr.options.len(), 1);
        assert_eq!(attr.options[0].key, "description");
        assert_eq!(attr.options[0].value, "This is a test");
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
}
