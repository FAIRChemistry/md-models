use std::{error::Error, fmt::Display, path::Path, str::FromStr};

use crate::{
    attribute::{AttrOption, Attribute},
    datamodel::DataModel,
    markdown::frontmatter::FrontMatter,
    object::Object,
};

static PROP_KEYS: [&str; 9] = [
    "type", "format", "enum", "minimum", "maximum", "minItems", "maxItems", "title", "items",
];

#[derive(Debug)]
enum DataType {
    String,
    Integer,
    Number,
    Boolean,
    Object,
    Array,
}

impl Display for DataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataType::String => write!(f, "string"),
            DataType::Integer => write!(f, "integer"),
            DataType::Number => write!(f, "number"),
            DataType::Boolean => write!(f, "boolean"),
            DataType::Object => write!(f, "object"),
            DataType::Array => write!(f, "array"),
        }
    }
}

impl FromStr for DataType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "string" => Ok(DataType::String),
            "integer" => Ok(DataType::Integer),
            "number" => Ok(DataType::Number),
            "boolean" => Ok(DataType::Boolean),
            "object" => Ok(DataType::Object),
            "array" => Ok(DataType::Array),
            _ => Err(format!("Unknown data type: {}", s)),
        }
    }
}

/// Parse a JSON schema into an MD-Models data model
pub fn parse_json_schema(path: &Path) -> Result<DataModel, Box<dyn Error>> {
    let schema = read_json_schema(path).expect(
        "Could not read the JSON schema file. Make sure the file is a valid JSON schema file.",
    );

    // Create a new data model
    let name = schema
        .get("title")
        .expect("Could not find title in the JSON schema")
        .as_str()
        .expect("Title is not a string")
        .to_string();
    let mut model = DataModel::new(Some(name), None);
    model.config = Some(FrontMatter::default());

    // Create the root object
    let object = create_object(&schema);
    model.objects.push(object);

    // Create the rest of the objects
    let definitions = schema.get("definitions").unwrap();
    for (_, value) in definitions.as_object().unwrap() {
        let object = create_object(value);
        model.objects.push(object);
    }

    Ok(model)
}

/// Read JSON schema from a file
fn read_json_schema(path: &Path) -> Result<serde_json::Value, serde_json::Error> {
    let content = std::fs::read_to_string(path).expect("Could not read the JSON schema file");
    serde_json::from_str(&content)
}

/// Extract properties from a JSON schema
fn create_object(schema: &serde_json::Value) -> Object {
    let name = schema
        .get("title")
        .expect("Could not find title in the JSON schema")
        .as_str()
        .expect("Title is not a string");
    let properties = schema
        .get("properties")
        .expect("Could not find properties in the JSON schema")
        .as_object()
        .expect("Properties is not an object");

    let mut object = Object::new(name.to_string(), None);

    for (key, value) in properties {
        let data_type = value
            .get("type")
            .expect("Could not find type in the property")
            .as_str()
            .expect("Type is not a string")
            .parse::<DataType>()
            .expect("Could not parse data type");

        let mut attribute = match data_type {
            DataType::Object => process_object(key, value),
            DataType::Array => process_array(key, value),
            _ => process_primitive(key, value),
        };

        // Add all other keys as options
        for (key, value) in value.as_object().unwrap() {
            if !PROP_KEYS.contains(&key.as_str()) {
                attribute.add_option(AttrOption::new(
                    key.to_string(),
                    value.as_str().unwrap().to_string(),
                ));
            }
        }

        object.attributes.push(attribute);
    }

    object
}

fn process_array(name: &str, value: &serde_json::Value) -> Attribute {
    // Prepare attribute
    let mut attribute = Attribute::new(name.to_string(), false);
    attribute.is_array = true;

    // Get the items
    let items = value
        .get("items")
        .expect("Could not find items in the array")
        .as_object()
        .expect("Items is not an object");

    // Check whether the items is a ref or any other type
    let data_type = if let Some(ref_name) = items.get("$ref") {
        ref_name
            .as_str()
            .expect("Ref is not a string")
            .split('/')
            .last()
            .expect("Could not get the last part of the ref")
            .to_string()
    } else {
        items
            .get("type")
            .expect("Could not find type in the array items")
            .as_str()
            .expect("Could not parse data type")
            .to_string()
    };

    // Set the data type
    attribute.dtypes = vec![data_type];

    attribute
}

fn process_primitive(name: &str, value: &serde_json::Value) -> Attribute {
    let mut attribute = Attribute::new(name.to_string(), false);
    let data_type = value
        .get("type")
        .expect("Could not find type in the property")
        .as_str()
        .expect("Type is not a string")
        .to_string();

    attribute.dtypes = vec![data_type];

    attribute
}

fn process_object(_name: &str, _value: &serde_json::Value) -> Attribute {
    panic!("Nested object type is not supported yet");
}
