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

use std::{collections::HashMap, error::Error, path::Path};

use gray_matter::{engine::YAML, Matter};
use serde::{Deserialize, Serialize};

#[cfg(feature = "python")]
use pyo3::pyclass;

#[cfg(feature = "wasm")]
use tsify_next::Tsify;

#[cfg(not(target_arch = "wasm32"))]
use crate::git;
use crate::prelude::DataModel;

/// Represents the front matter data of a markdown file.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "python", pyclass(get_all, from_py_object))]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi))]
pub struct FrontMatter {
    /// Identifier field of the model.
    pub id: Option<String>,
    /// A boolean field with a default value, renamed from `id-field`.
    #[serde(default = "default_id_field", rename = "id-field")]
    pub id_field: bool,
    /// Optional hashmap of prefixes.
    pub prefixes: Option<HashMap<String, String>>,
    /// Optional namespace map.
    pub nsmap: Option<HashMap<String, String>>,
    /// A string field with a default value representing the repository URL.
    #[serde(default = "default_repo", alias = "iri")]
    pub repo: String,
    /// A string field with a default value representing the prefix.
    #[serde(default = "default_prefix")]
    pub prefix: String,
    /// Import remote or local models.
    #[serde(default)]
    pub imports: HashMap<String, ImportType>,
    /// Allow empty models.
    #[serde(default = "default_allow_empty", rename = "allow-empty")]
    pub allow_empty: bool,
}

impl FrontMatter {
    pub fn new() -> Self {
        FrontMatter {
            id: None,
            id_field: default_id_field(),
            prefixes: None,
            nsmap: None,
            repo: placeholder_repo(&default_prefix()),
            prefix: default_prefix(),
            imports: HashMap::new(),
            allow_empty: false,
        }
    }

    /// Whether `repo` is still the `https://example.org/<prefix>/` placeholder,
    /// i.e. the model did not set a `repo` of its own.
    pub fn has_placeholder_repo(&self) -> bool {
        self.repo == placeholder_repo(&self.prefix)
    }

    /// Prefixes whose IRI ends in a letter or digit (sorted by prefix), which usually
    /// means a missing separator: with `schema: https://schema.org`, `schema:name`
    /// expands to `https://schema.orgname`. IRIs ending in `/`, `#`, `_` etc. are fine.
    pub fn prefixes_missing_delimiter(&self) -> Vec<(String, String)> {
        let mut found: Vec<(String, String)> = self
            .prefixes()
            .unwrap_or_default()
            .into_iter()
            .filter(|(_, iri)| iri.ends_with(|c: char| c.is_ascii_alphanumeric()))
            .collect();
        found.sort();
        found
    }

    /// Returns the value of the `id_field`.
    ///
    /// # Returns
    /// A boolean representing the `id_field`.
    pub fn id_field(&self) -> bool {
        self.id_field
    }

    /// Returns the prefixes as declared (IRIs are not modified).
    ///
    /// # Returns
    /// An optional vector of tuples containing the prefixes.
    pub fn prefixes(&self) -> Option<Vec<(String, String)>> {
        self.prefixes.as_ref().map(|prefixes| {
            prefixes
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect()
        })
    }

    /// Returns a reference to the namespace map.
    ///
    /// # Returns
    /// A reference to an optional hashmap of the namespace map.
    pub fn nsmap(&self) -> &Option<HashMap<String, String>> {
        &self.nsmap
    }
}

#[derive(Debug, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "python", pyclass(get_all, from_py_object))]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi))]
/// Represents different types of model imports.
///
/// Can be either a remote URL or a local file path.
pub enum ImportType {
    /// A remote URL pointing to a model
    Remote(String),
    /// A local file path to a model
    Local(String),
}

impl<'de> Deserialize<'de> for ImportType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        // Check if string starts with http:// or https://
        if s.starts_with("http://") || s.starts_with("https://") {
            Ok(ImportType::Remote(s))
        } else {
            Ok(ImportType::Local(s))
        }
    }
}

impl ImportType {
    /// Fetches and parses the model from either remote or local source.
    ///
    /// # Returns
    /// A Result containing the parsed DataModel or an error.
    pub fn fetch(&self, dirpath: Option<&Path>) -> Result<DataModel, Box<dyn Error>> {
        match self {
            ImportType::Remote(url) => self.fetch_remote_model(url),
            ImportType::Local(path) => self.fetch_local_model(path, dirpath),
        }
    }

    /// Fetches and parses a model from a remote URL.
    ///
    /// # Arguments
    /// * `url` - The URL to fetch the model from
    ///
    /// # Returns
    /// A Result containing the parsed DataModel or an error.
    fn fetch_remote_model(&self, url: &str) -> Result<DataModel, Box<dyn Error>> {
        #[cfg(not(target_arch = "wasm32"))]
        if let Some((repo, path)) = git::parse_github_file_url(url) {
            return DataModel::from_github(&repo, &path);
        }

        Err(format!("Unsupported remote import URL: {url}").into())
    }

    /// Fetches and parses a model from a local file path.
    ///
    /// # Arguments
    /// * `path` - The file path to read the model from
    ///
    /// # Returns
    /// A Result containing the parsed DataModel or an error.
    fn fetch_local_model(
        &self,
        path: &str,
        dirpath: Option<&Path>,
    ) -> Result<DataModel, Box<dyn Error>> {
        let path = if let Some(dirpath) = dirpath {
            dirpath.parent().unwrap().join(path).display().to_string()
        } else {
            path.to_string()
        };
        let path = std::fs::canonicalize(path)?;
        let model = DataModel::from_markdown(&path)?;
        Ok(model)
    }
}

impl Default for FrontMatter {
    /// Provides default values for `FrontMatter`.
    ///
    /// # Returns
    /// A `FrontMatter` instance with default values.
    fn default() -> Self {
        Self::new()
    }
}

/// Provides the default value for the `id_field`.
///
/// # Returns
/// A boolean with the default value `true`.
fn default_id_field() -> bool {
    true
}

/// Provides the default value for the `prefix`.
///
/// # Returns
/// A string with the default value `"md"`.
fn default_prefix() -> String {
    "md".to_string()
}

/// Provides the serde default for a missing `repo`: the bare reserved
/// `https://example.org/`, which `parse_frontmatter` then scopes by prefix
/// (see [`placeholder_repo`]).
fn default_repo() -> String {
    "https://example.org/".to_string()
}

/// Placeholder `repo` for models that do not set one, in the RFC 2606 reserved
/// `example.org` domain and scoped by the model `prefix`.
pub fn placeholder_repo(prefix: &str) -> String {
    format!("https://example.org/{prefix}/")
}

/// Provides the default value for the `allow_empty`.
///
/// # Returns
/// A boolean with the default value `false`.
fn default_allow_empty() -> bool {
    false
}

/// Parses the front matter from the given content.
///
/// # Arguments
/// * `content` - A string slice that holds the content to parse.
///
/// # Returns
/// An optional `FrontMatter` if parsing is successful, otherwise `None`.
pub fn parse_frontmatter(content: &str) -> Option<FrontMatter> {
    let matter = Matter::<YAML>::new();
    let mut fm: FrontMatter = matter.parse(content).ok()?.data?;
    if fm.repo == default_repo() {
        fm.repo = placeholder_repo(&fm.prefix);
    }
    Some(fm)
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use std::path::Path;

    use super::*;

    /// Tests the `parse_frontmatter` function.
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

    #[test]
    fn test_missing_repo_gets_prefix_scoped_placeholder() {
        let fm = parse_frontmatter("---\nprefix: \"ex\"\n---\n").unwrap();
        assert_eq!(fm.repo, "https://example.org/ex/");
        assert!(fm.has_placeholder_repo());
    }

    #[test]
    fn test_explicit_repo_is_kept() {
        let fm =
            parse_frontmatter("---\nprefix: \"ex\"\nrepo: \"https://example.org/jsonld/\"\n---\n")
                .unwrap();
        assert_eq!(fm.repo, "https://example.org/jsonld/");
        assert!(!fm.has_placeholder_repo());
    }

    #[test]
    fn test_default_frontmatter_uses_placeholder_repo() {
        let fm = FrontMatter::default();
        assert_eq!(fm.repo, "https://example.org/md/");
        assert!(fm.has_placeholder_repo());
    }

    #[test]
    fn test_prefixes_are_kept_verbatim() {
        let content = std::fs::read_to_string("tests/data/model_jsonld.md").unwrap();
        let frontmatter = parse_frontmatter(&content).expect("Could not parse frontmatter");
        let prefixes: HashMap<String, String> =
            frontmatter.prefixes().unwrap().into_iter().collect();
        assert_eq!(prefixes["rdfs"], "http://www.w3.org/2000/01/rdf-schema#");
        assert_eq!(prefixes["CHEBI"], "http://purl.obolibrary.org/obo/CHEBI_");
        assert_eq!(prefixes["schema"], "https://schema.org/");
    }

    #[test]
    fn test_prefixes_missing_delimiter() {
        let content = "---\nprefix: ex\nprefixes:\n  schema: https://schema.org\n  ok: https://schema.org/\n  rdfs: http://www.w3.org/2000/01/rdf-schema#\n  CHEBI: http://purl.obolibrary.org/obo/CHEBI_\n  foaf: http://xmlns.com/foaf/0.1\n---\n";
        let frontmatter = parse_frontmatter(content).expect("Could not parse frontmatter");
        assert_eq!(
            frontmatter.prefixes_missing_delimiter(),
            vec![
                ("foaf".to_string(), "http://xmlns.com/foaf/0.1".to_string()),
                ("schema".to_string(), "https://schema.org".to_string()),
            ]
        );
    }
}
