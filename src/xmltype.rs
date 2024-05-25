use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq)]
pub enum XMLType {
    Attribute { is_attr: bool, name: String },
    Element { is_attr: bool, name: String },
}

impl FromStr for XMLType {
    type Err = String;

    // Parse XMLType from string, if it starts with '@' it's an attribute
    // otherwise it's an element. This is mainly used for parsing the XML
    // option in the attribute definition.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with('@') {
            Ok(XMLType::Attribute {
                is_attr: true,
                name: s[1..].to_string(),
            })
        } else {
            Ok(XMLType::Element {
                is_attr: false,
                name: s.to_string(),
            })
        }
    }
}

impl<'de> Deserialize<'de> for XMLType {
    fn deserialize<D>(deserializer: D) -> Result<XMLType, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Example {"xml": {"is_attr": true, "name": "id"}} for an attribute
        // Example {"xml": {"is_attr": false, "name": "name"}} for an element
        #[derive(Deserialize)]
        struct XMLTypeVisitor {
            is_attr: bool,
            name: String,
        }

        let value = XMLTypeVisitor::deserialize(deserializer)?;
        if value.is_attr {
            Ok(XMLType::Attribute {
                is_attr: true,
                name: value.name,
            })
        } else {
            Ok(XMLType::Element {
                is_attr: false,
                name: value.name,
            })
        }
    }
}

impl Serialize for XMLType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(Serialize)]
        struct XMLTypeVisitor {
            is_attr: bool,
            name: String,
        }
        // Return {"is_attr": true, "name": "id"}
        match self {
            XMLType::Attribute { is_attr, name } => {
                let visitor = XMLTypeVisitor {
                    is_attr: *is_attr,
                    name: name.to_string(),
                };
                visitor.serialize(serializer)
            }
            XMLType::Element { is_attr, name } => {
                let visitor = XMLTypeVisitor {
                    is_attr: *is_attr,
                    name: name.to_string(),
                };
                visitor.serialize(serializer)
            }
        }
    }
}
