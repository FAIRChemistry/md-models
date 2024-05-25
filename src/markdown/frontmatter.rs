use std::collections::HashMap;

use gray_matter::{engine::YAML, Matter};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct FrontMatter {
    #[serde(default = "default_id_field", rename = "id-field")]
    pub id_field: bool,
    pub prefixes: Option<HashMap<String, String>>,
    pub nsmap: Option<HashMap<String, String>>,
    pub repo: Option<String>,
}

impl FrontMatter {
    pub fn id_field(&self) -> bool {
        self.id_field
    }

    pub fn prefixes(&self) -> Option<Vec<(String, String)>> {
        match &self.prefixes {
            Some(prefixes) => Some(
                prefixes
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect(),
            ),
            None => None,
        }
    }

    pub fn nsmap(&self) -> &Option<HashMap<String, String>> {
        &self.nsmap
    }
}

impl Default for FrontMatter {
    fn default() -> Self {
        FrontMatter {
            id_field: default_id_field(),
            prefixes: None,
            repo: None,
            nsmap: None,
        }
    }
}

fn default_id_field() -> bool {
    true
}

pub fn parse_frontmatter(content: &str) -> Option<FrontMatter> {
    let matter = Matter::<YAML>::new();
    let result = matter.parse(content);

    match result.data {
        None => None,
        Some(data) => {
            let matter = data
                .deserialize()
                .expect("Could not deserialize frontmatter");
            Some(matter)
        }
    }
}

#[cfg(test)]
mod tests {

    use std::path::Path;

    use super::*;

    #[test]
    fn test_parse_frontmatter() {
        // Arrange
        let path = Path::new("tests/data/model.md");
        let content = std::fs::read_to_string(path).expect("Could not read file");

        // Act
        let frontmatter = parse_frontmatter(&content)
            .expect("Could not parse frontmatter from file. Please check the file content.");

        // Assert
        assert_eq!(frontmatter.id_field, true);
        assert_eq!(
            frontmatter.prefixes.unwrap().get("schema").unwrap(),
            "http://schema.org/"
        );
        assert_eq!(
            frontmatter.nsmap.unwrap().get("tst").unwrap(),
            "http://example.com/test/"
        );
    }
}
