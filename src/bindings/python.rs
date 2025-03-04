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

use std::collections::HashMap;
use std::path::Path;

use crate::attribute::Attribute;
use crate::datamodel;
use crate::exporters::Templates;
use crate::markdown::frontmatter::FrontMatter;
use crate::object::{Enumeration, Object};
use crate::option::AttrOption;
use pyo3::prelude::*;
use pyo3::types::PyType;

/// A Python class that wraps the `datamodel::DataModel` struct.
#[pyclass]
pub struct DataModel {
    /// The underlying Rust `DataModel` instance.
    #[pyo3(get)]
    pub model: datamodel::DataModel,
}

#[pymethods]
impl DataModel {
    /// Creates a new `DataModel` instance from a markdown file.
    ///
    /// # Arguments
    ///
    /// * `path` - A string slice that holds the path to the markdown file.
    ///
    /// # Returns
    ///
    /// A new instance of `DataModel`.
    #[classmethod]
    #[pyo3(signature = (path))]
    fn from_markdown(_cls: &Bound<'_, PyType>, path: String) -> Self {
        Self {
            model: datamodel::DataModel::from_markdown(Path::new(&path)).unwrap(),
        }
    }

    /// Creates a new `DataModel` instance from a markdown string.
    ///
    /// # Arguments
    ///
    /// * `content` - A string slice that holds the markdown content.
    ///
    /// # Returns
    ///
    /// A new instance of `DataModel`.
    #[classmethod]
    #[pyo3(signature = (content))]
    fn from_markdown_string(_cls: &Bound<'_, PyType>, content: String) -> Self {
        Self {
            model: datamodel::DataModel::from_markdown_string(&content).unwrap(),
        }
    }

    /// Returns a string representation of the `DataModel` instance.
    ///
    /// # Returns
    ///
    /// A string that represents the `DataModel` instance.
    fn __repr__(&self) -> String {
        self.model.internal_schema()
    }

    /// Converts the `DataModel` instance to a specified template format.
    ///
    /// # Arguments
    ///
    /// * `template` - The template to convert the `DataModel` to.
    /// * `config` - An optional configuration hashmap.
    ///
    /// # Returns
    ///
    /// A string that represents the converted `DataModel`.
    #[pyo3(signature = (template, config=None))]
    fn convert_to(
        &mut self,
        template: Templates,
        config: Option<HashMap<String, String>>,
    ) -> String {
        let config = config.unwrap_or_default();
        self.model
            .convert_to(&template, Some(&config))
            .expect("Failed to convert to template")
    }
}

#[pymethods]
impl Object {
    /// Returns a string representation of the `Object` instance.
    ///
    /// # Returns
    ///
    /// A string that represents the `Object` instance.
    pub fn __repr__(&self) -> String {
        serde_json::to_string_pretty(&self).unwrap()
    }
}

#[pymethods]
impl Attribute {
    /// Returns a string representation of the `Attribute` instance.
    ///
    /// # Returns
    ///
    /// A string that represents the `Attribute` instance.
    fn __repr__(&self) -> String {
        serde_json::to_string_pretty(&self).unwrap()
    }
}

#[pymethods]
impl FrontMatter {
    /// Returns a string representation of the `FrontMatter` instance.
    ///
    /// # Returns
    ///
    /// A string that represents the `FrontMatter` instance.
    fn __repr__(&self) -> String {
        serde_json::to_string_pretty(&self).unwrap()
    }
}

#[pymethods]
impl Enumeration {
    /// Returns a string representation of the `Enumeration` instance.
    ///
    /// # Returns
    ///
    /// A string that represents the `Enumeration` instance.
    fn __repr__(&self) -> String {
        serde_json::to_string_pretty(&self).unwrap()
    }
}

#[pymethods]
impl AttrOption {
    /// Returns a key-value (RawOption) representation of the `AttrOption` instance.
    ///
    /// # Returns
    ///
    /// A key-value (RawOption) representation of the `AttrOption` instance.
    fn pair(&self) -> (String, String) {
        self.to_pair()
    }

    /// Returns the key of the `AttrOption` instance.
    ///
    /// # Returns
    ///
    /// A string that represents the key of the `AttrOption` instance.
    fn k(&self) -> String {
        self.key().to_string()
    }

    /// Returns the value of the `AttrOption` instance.
    ///
    /// # Returns
    ///
    /// A string that represents the value of the `AttrOption` instance.
    fn v(&self) -> String {
        self.value().to_string()
    }

    /// Returns a string representation of the `AttrOption` instance.
    ///
    /// # Returns
    ///
    /// A string that represents the `AttrOption` instance.
    fn __repr__(&self) -> String {
        serde_json::to_string_pretty(&self).unwrap()
    }
}
