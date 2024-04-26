use std::collections::HashMap;

pub struct PrimitiveTypes {
    types: Vec<String>,
    json_mappings: HashMap<String, String>,
}

impl PrimitiveTypes {
    pub fn new() -> Self {
        let mut json_mappings = HashMap::new();

        json_mappings.insert("string".to_string(), "string".to_string());
        json_mappings.insert("float".to_string(), "number".to_string());
        json_mappings.insert("integer".to_string(), "integer".to_string());
        json_mappings.insert("boolean".to_string(), "boolean".to_string());
        json_mappings.insert("bool".to_string(), "boolean".to_string());
        json_mappings.insert("null".to_string(), "null".to_string());

        PrimitiveTypes {
            types: vec![
                "string".to_string(),
                "float".to_string(),
                "integer".to_string(),
                "boolean".to_string(),
                "bool".to_string(),
                "null".to_string(),
            ],
            json_mappings,
        }
    }

    pub fn filter_non_primitives(&self, dtypes: &Vec<String>) -> Vec<String> {
        let mut non_primitive_types: Vec<String> = Vec::new();
        for dtype in dtypes {
            if !self.is_primitive(dtype) {
                non_primitive_types.push(dtype.to_string());
            }
        }

        non_primitive_types
    }

    pub fn filter_primitive(&self, dtypes: &Vec<String>) -> Vec<String> {
        let mut primitive_types: Vec<String> = Vec::new();
        for dtype in dtypes {
            if self.is_primitive(dtype) {
                primitive_types.push(dtype.to_string());
            }
        }

        primitive_types
    }

    fn is_primitive(&self, dtype: &str) -> bool {
        self.types.contains(&dtype.to_string())
    }

    pub fn dtype_to_json(&self, dtype: &String) -> String {
        if !self.json_mappings.contains_key(dtype) {
            panic!("The data type {} is not a primitive type", dtype)
        } else {
            self.json_mappings[dtype].to_string()
        }
    }
}
