/*
 * Copyright (c) 2024 Jan Range
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

use std::{collections::HashMap, error::Error, fmt::Display, str::FromStr};

use crate::{datamodel::DataModel, markdown::frontmatter::FrontMatter};
use clap::ValueEnum;
use lazy_static::lazy_static;
use minijinja::{context, Environment};
use textwrap::wrap;

#[cfg(feature = "python")]
use pyo3::pyclass;

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::wasm_bindgen;

lazy_static! {
    /// Maps generic type names to Python-specific type names.
    static ref PYTHON_TYPE_MAPS: std::collections::HashMap<String, String> = {
        let mut m = std::collections::HashMap::new();
        m.insert("string".to_string(), "str".to_string());
        m.insert("integer".to_string(), "int".to_string());
        m.insert("boolean".to_string(), "bool".to_string());
        m.insert("number".to_string(), "float".to_string());
        m
    };

    /// Maps generic type names to SHACL-specific type names.
    static ref SHACL_TYPE_MAPS: std::collections::HashMap<String, String> = {
        let mut m = std::collections::HashMap::new();
        m.insert("float".to_string(), "double".to_string());
        m.insert("bytes".to_string(), "base64Binary".to_string());
        m
    };

    /// Maps Python-specific type names to XSD-specific type names.
    static ref XSD_TYPE_MAPS: std::collections::HashMap<String, String> = {
        let mut m = std::collections::HashMap::new();
        m.insert("str".to_string(), "string".to_string());
        m.insert("bytes".to_string(), "base64Binary".to_string());
        m
    };

    /// Maps MD-Models type names to Typescript-specific type names.
    static ref TYPESCRIPT_TYPE_MAPS: std::collections::HashMap<String, String> = {
        let mut m = std::collections::HashMap::new();
        m.insert("integer".to_string(), "number".to_string());
        m.insert("float".to_string(), "number".to_string());
        m.insert("date".to_string(), "string".to_string());
        m.insert("bytes".to_string(), "string".to_string());
        m
    };
}

/// Enumeration of available templates.
#[derive(Debug, ValueEnum, Clone, PartialEq)]
#[cfg_attr(feature = "python", pyclass(eq, eq_int))]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub enum Templates {
    XmlSchema,
    Markdown,
    CompactMarkdown,
    Shacl,
    JsonSchema,
    JsonSchemaAll,
    Shex,
    PythonDataclass,
    PythonPydanticXML,
    PythonPydantic,
    MkDocs,
    Internal,
    Typescript,
}

impl Display for Templates {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Templates::PythonDataclass => write!(f, "python-dataclass"),
            Templates::PythonPydantic => write!(f, "python-pydantic"),
            Templates::PythonPydanticXML => write!(f, "python-pydantic-xml"),
            Templates::XmlSchema => write!(f, "xml-schema"),
            Templates::Markdown => write!(f, "markdown"),
            Templates::CompactMarkdown => write!(f, "compact-markdown"),
            Templates::Shacl => write!(f, "shacl"),
            Templates::JsonSchema => write!(f, "json-schema"),
            Templates::JsonSchemaAll => write!(f, "json-schema-all"),
            Templates::Shex => write!(f, "shex"),
            Templates::MkDocs => write!(f, "mk-docs"),
            Templates::Internal => write!(f, "internal"),
            Templates::Typescript => write!(f, "typescript"),
        }
    }
}

/// Converts string representation of a template to a `Templates` enum.
/// and returns an error if the string is not a valid template type.
impl FromStr for Templates {
    type Err = Box<dyn Error>;
    fn from_str(s: &str) -> Result<Self, Box<dyn Error>> {
        match s {
            "python-dataclass" => Ok(Templates::PythonDataclass),
            "python-sdrdm" => Ok(Templates::PythonPydanticXML),
            "python-pydantic" => Ok(Templates::PythonPydantic),
            "xml-schema" => Ok(Templates::XmlSchema),
            "markdown" => Ok(Templates::Markdown),
            "compact-markdown" => Ok(Templates::CompactMarkdown),
            "shacl" => Ok(Templates::Shacl),
            "json-schema" => Ok(Templates::JsonSchema),
            "json-schema-all" => Ok(Templates::JsonSchemaAll),
            "shex" => Ok(Templates::Shex),
            "mk-docs" => Ok(Templates::MkDocs),
            "internal" => Ok(Templates::Internal),
            "typescript" => Ok(Templates::Typescript),
            _ => {
                let err = format!("Invalid template type: {}", s);
                Err(err.into())
            }
        }
    }
}

/// Renders a Jinja template based on the provided template type and data model.
///
/// # Arguments
///
/// * `template` - The type of template to render.
/// * `model` - The data model to use for rendering the template.
///
/// # Returns
///
/// A Result containing the rendered template as a String or an error if rendering fails.
pub fn render_jinja_template(
    template: &Templates,
    model: &mut DataModel,
    config: Option<&HashMap<String, String>>,
) -> Result<String, minijinja::Error> {
    // Load the template environment
    let mut env = Environment::new();
    minijinja_embed::load_templates!(&mut env);

    // Perform type conversions and filtering based on the template
    match template {
        Templates::XmlSchema => convert_model_types(model, &XSD_TYPE_MAPS),
        Templates::Typescript => convert_model_types(model, &TYPESCRIPT_TYPE_MAPS),
        Templates::Shacl | Templates::Shex => {
            convert_model_types(model, &SHACL_TYPE_MAPS);
            filter_objects_wo_terms(model);
        }
        Templates::PythonDataclass | Templates::PythonPydanticXML | Templates::PythonPydantic => {
            convert_model_types(model, &PYTHON_TYPE_MAPS);
            sort_attributes_by_required(model);
        }
        _ => {}
    }

    // Add custom functions to the Jinja environment
    env.add_function("wrap", wrap_text);

    // Get the appropriate template
    let template = match template {
        Templates::PythonDataclass => env.get_template("python-dataclass.jinja")?,
        Templates::PythonPydantic => env.get_template("python-pydantic.jinja")?,
        Templates::XmlSchema => env.get_template("xml-schema.jinja")?,
        Templates::Markdown => env.get_template("markdown.jinja")?,
        Templates::CompactMarkdown => env.get_template("markdown-compact.jinja")?,
        Templates::Shacl => env.get_template("shacl.jinja")?,
        Templates::Shex => env.get_template("shex.jinja")?,
        Templates::PythonPydanticXML => env.get_template("python-pydantic-xml.jinja")?,
        Templates::MkDocs => env.get_template("mkdocs.jinja")?,
        Templates::Typescript => env.get_template("typescript.jinja")?,
        _ => {
            panic!(
                "The template is not available as a Jinja Template and should not be used using the jinja exporter.
                Instead, use the dedicated exporter in the DataModel struct."
            )
        }
    };

    // If there is no config, create an empty one
    // This is necessary to avoid errors when rendering the template
    if model.config.is_none() {
        model.config = Some(FrontMatter::default());
    }

    // Render the template
    let prefixes = get_prefixes(model);
    let rendered = template.render(context! {
        objects => model.objects,
        object_names => model.objects.iter().map(|o| o.name.clone()).collect::<Vec<String>>(),
        enums => model.enums,
        enum_names => model.enums.iter().map(|e| e.name.clone()).collect::<Vec<String>>(),
        title => model.name,
        prefixes => prefixes,
        repo => model.config.as_ref().unwrap().repo.clone(),
        prefix => model.config.as_ref().unwrap().prefix.clone(),
        config => config,
    });

    match rendered {
        Ok(r) => Ok(clean_and_trim(&r)),
        Err(e) => Err(e),
    }
}

/// Template function that allows to wrap text at a certain length.
///
/// # Arguments
///
/// * `text` - The text to wrap.
/// * `width` - The maximum length of a line.
/// * `offset` - The offset to use for all lines.
///
/// # Returns
///
/// A string with the wrapped text.
fn wrap_text(text: &str, width: usize, initial_offset: &str, offset: &str) -> String {
    // Remove multiple spaces
    let options = textwrap::Options::new(width)
        .initial_indent(initial_offset)
        .subsequent_indent(offset)
        .width(width)
        .break_words(false);

    wrap(remove_multiple_spaces(text).as_str(), options).join("\n")
}

/// Removes leading and trailing whitespace and multiple spaces from a string.
fn remove_multiple_spaces(input: &str) -> String {
    input.split_whitespace().collect::<Vec<&str>>().join(" ")
}

/// Converts the data types in the model according to the provided type map.
///
/// # Arguments
///
/// * `model` - The data model whose types are to be converted.
/// * `type_map` - A map of generic type names to specific type names.
fn convert_model_types(
    model: &mut DataModel,
    type_map: &std::collections::HashMap<String, String>,
) {
    for object in &mut model.objects {
        for attribute in &mut object.attributes {
            attribute.dtypes = attribute
                .dtypes
                .iter()
                .map(|t| type_map.get(t).unwrap_or(t))
                .map(|t| t.to_string())
                .collect();
        }
    }
}

/// Retrieves the prefixes from the model configuration.
///
/// # Arguments
///
/// * `model` - The data model from which to retrieve the prefixes.
///
/// # Returns
///
/// A vector of prefix tuples (prefix, URI).
fn get_prefixes(model: &mut DataModel) -> Vec<(String, String)> {
    match &model.config {
        Some(config) => config.prefixes().unwrap_or(vec![]),
        None => vec![],
    }
}

/// Filters out objects from the model that do not have any terms.
///
/// # Arguments
///
/// * `model` - The data model to filter.
fn filter_objects_wo_terms(model: &mut DataModel) {
    model.objects.retain(|o| o.has_any_terms());

    if model.objects.is_empty() {
        panic!("No objects with terms found in the model. Unable to build SHACL or ShEx.");
    }
}

/// Sorts the attributes of each object in the model by their 'required' field.
///
/// # Arguments
///
/// * `model` - The data model whose attributes are to be sorted.
fn sort_attributes_by_required(model: &mut DataModel) {
    for object in &mut model.objects {
        object.sort_attrs_by_required();
    }
}

fn clean_and_trim(s: &str) -> String {
    let splitted = s.split('\n').collect::<Vec<&str>>();
    let mut cleaned = vec![];
    let mut consec_empty = 0;

    for line in splitted {
        let trimmed = line.trim_end();
        if !trimmed.is_empty() {
            cleaned.push(trimmed);
            consec_empty = 0;
        } else {
            consec_empty += 1;
            if consec_empty < 3 {
                cleaned.push(trimmed);
            }
        }
    }

    cleaned.join("\n").trim().to_string()
    // s.to_string()
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use std::{fs, path::Path};

    use crate::markdown::parser::parse_markdown;

    use super::*;

    /// Helper function to build and convert a template.
    ///
    /// # Arguments
    ///
    /// * `template` - The template type to use for rendering.
    ///
    /// # Returns
    ///
    /// A string containing the rendered template.
    fn build_and_convert(template: Templates) -> String {
        let path = Path::new("tests/data/model.md");
        let content = fs::read_to_string(path).expect("Could not read markdown file");
        let mut model = parse_markdown(&content).expect("Failed to parse markdown file");
        render_jinja_template(&template, &mut model, None)
            .expect("Could not render template")
            .to_string()
    }

    #[test]
    fn test_convert_to_shex() {
        // Arrange
        let rendered = build_and_convert(Templates::Shex);

        // Assert
        let expected = fs::read_to_string("tests/data/expected_shex.shex")
            .expect("Could not read expected file");
        assert_eq!(rendered, expected);
    }

    #[test]
    fn test_convert_to_shacl() {
        // Arrange
        let rendered = build_and_convert(Templates::Shacl);

        // Assert
        let expected = fs::read_to_string("tests/data/expected_shacl.ttl")
            .expect("Could not read expected file");
        assert_eq!(rendered, expected);
    }

    #[test]
    fn test_convert_to_python_dc() {
        // Arrange
        let rendered = build_and_convert(Templates::PythonDataclass);

        // Assert
        let expected = fs::read_to_string("tests/data/expected_python_dc.py")
            .expect("Could not read expected file");
        assert_eq!(rendered, expected);
    }

    #[test]
    fn test_convert_to_python_pydantic_xml() {
        // Arrange
        let rendered = build_and_convert(Templates::PythonPydanticXML);

        // Assert
        let expected = fs::read_to_string("tests/data/expected_python_pydantic_xml.py")
            .expect("Could not read expected file");
        assert_eq!(rendered, expected);
    }

    #[test]
    fn test_convert_to_xsd() {
        // Arrange
        let rendered = build_and_convert(Templates::XmlSchema);

        // Assert
        let expected = fs::read_to_string("tests/data/expected_xml_schema.xsd")
            .expect("Could not read expected file");
        assert_eq!(rendered, expected);
    }

    #[test]
    fn test_convert_to_mkdocs() {
        // Arrange
        let rendered = build_and_convert(Templates::MkDocs);

        // Assert
        let expected = fs::read_to_string("tests/data/expected_mkdocs.md")
            .expect("Could not read expected file");
        assert_eq!(rendered, expected);
    }

    #[test]
    fn test_convert_to_typescript() {
        // Arrange
        let rendered = build_and_convert(Templates::Typescript);

        // Assert
        let expected = fs::read_to_string("tests/data/expected_typescript.ts")
            .expect("Could not read expected file");
        assert_eq!(rendered, expected);
    }

    #[test]
    fn test_convert_to_pydantic() {
        // Arrange
        let rendered = build_and_convert(Templates::PythonPydantic);

        // Assert
        let expected = fs::read_to_string("tests/data/expected_pydantic.py")
            .expect("Could not read expected file");
        assert_eq!(rendered, expected);
    }
}
