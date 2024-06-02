use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Represents an XML type, either an attribute or an element.
#[derive(Debug, PartialEq, Clone)]
pub enum XMLType {
    /// An XML attribute with a name.
    Attribute { is_attr: bool, name: String },
    /// An XML element with a name.
    Element { is_attr: bool, name: String },
}

impl FromStr for XMLType {
    type Err = String;

    /// Parses a string to create an `XMLType`. If the string starts with '@', it is considered an attribute;
    /// otherwise, it is considered an element.
    ///
    /// # Arguments
    ///
    /// * `s` - A string slice that holds the XML type definition.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(name) = s.strip_prefix('@') {
            Ok(XMLType::Attribute {
                is_attr: true,
                name: name.to_string(),
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
    /// Deserializes an `XMLType` from a deserializer.
    ///
    /// This is typically used when deserializing an `XMLType` from a format such as JSON.
    ///
    /// # Arguments
    ///
    /// * `deserializer` - A deserializer instance.
    fn deserialize<D>(deserializer: D) -> Result<XMLType, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
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
    /// Serializes an `XMLType` to a serializer.
    ///
    /// This is typically used when serializing an `XMLType` to a format such as JSON.
    ///
    /// # Arguments
    ///
    /// * `serializer` - A serializer instance.
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(Serialize)]
        struct XMLTypeVisitor {
            is_attr: bool,
            name: String,
        }

        let visitor = match self {
            XMLType::Attribute { is_attr, name } | XMLType::Element { is_attr, name } => {
                XMLTypeVisitor {
                    is_attr: *is_attr,
                    name: name.to_string(),
                }
            }
        };
        visitor.serialize(serializer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xmltype_from_str() {
        let attr = XMLType::Attribute {
            is_attr: true,
            name: "id".to_string(),
        };
        let elem = XMLType::Element {
            is_attr: false,
            name: "name".to_string(),
        };
        assert_eq!(XMLType::from_str("@id").unwrap(), attr);
        assert_eq!(XMLType::from_str("name").unwrap(), elem);
    }

    #[test]
    fn test_xmltype_deserialize() {
        let attr = XMLType::Attribute {
            is_attr: true,
            name: "id".to_string(),
        };
        let elem = XMLType::Element {
            is_attr: false,
            name: "name".to_string(),
        };
        let attr_json = r#"{"is_attr":true,"name":"id"}"#;
        let elem_json = r#"{"is_attr":false,"name":"name"}"#;
        assert_eq!(serde_json::from_str::<XMLType>(attr_json).unwrap(), attr);
        assert_eq!(serde_json::from_str::<XMLType>(elem_json).unwrap(), elem);
    }
}
