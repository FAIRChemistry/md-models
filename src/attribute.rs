use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Attribute {
    pub name: String,
    pub is_array: bool,
    pub dtypes: Vec<String>,
    pub docstring: String,
    pub options: HashMap<String, String>,
}

impl Attribute {
    pub fn new(name: String) -> Self {
        Attribute {
            name,
            dtypes: Vec::new(),
            docstring: String::new(),
            options: HashMap::new(),
            is_array: false,
        }
    }

    pub fn set_docstring(&mut self, docstring: String) {
        self.docstring = docstring;
    }

    pub fn add_option(&mut self, key: String, value: String) {
        if key.to_lowercase() == "type" {
            self.set_dtype(value);
            return;
        } else {
            self.options.insert(key.to_lowercase(), value);
        }
    }

    fn set_dtype(&mut self, dtype: String) {
        if dtype.ends_with("[]") {
            self.is_array = true;
            self.dtypes.push(dtype.trim_end_matches("[]").to_string());
            return;
        }

        self.dtypes.push(dtype);
    }
}
