use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Attribute {
    pub name: String,
    #[serde(rename = "multiple")]
    pub is_array: bool,
    pub dtypes: Vec<String>,
    pub docstring: String,
    pub options: Vec<AttrOption>,
}

impl Attribute {
    pub fn new(name: String) -> Self {
        Attribute {
            name,
            dtypes: Vec::new(),
            docstring: String::new(),
            options: Vec::new(),
            is_array: false,
        }
    }

    pub fn set_docstring(&mut self, docstring: String) {
        self.docstring = docstring;
    }

    pub fn add_option(&mut self, option: AttrOption) {
        if option.key.to_lowercase() == "type" {
            self.set_dtype(option.value);
            return;
        } else {
            self.options.push(option);
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

    pub fn to_json_schema(&self) -> String {
        serde_json::to_string_pretty(&self).unwrap()
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
