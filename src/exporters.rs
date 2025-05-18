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

use std::{collections::HashMap, error::Error, fmt::Display, str::FromStr};

use crate::{
    attribute::{Attribute, DataType},
    markdown::frontmatter::FrontMatter,
    object::Object,
    option::AttrOption,
    prelude::DataModel,
    tree,
    xmltype::XMLType,
};
use clap::ValueEnum;
use colored::Colorize;
use convert_case::{Case, Casing};
use lazy_static::lazy_static;
use minijinja::{
    context,
    value::{Kwargs, ValueKind, ViaDeserialize},
    Environment, Value,
};
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

    /// Maps MD-Models type names to GraphQL-specific type names.
    static ref GRAPHQL_TYPE_MAPS: std::collections::HashMap<String, String> = {
        let mut m = std::collections::HashMap::new();
        m.insert("integer".to_string(), "Int".to_string());
        m.insert("number".to_string(), "Float".to_string());
        m.insert("float".to_string(), "Float".to_string());
        m.insert("boolean".to_string(), "Boolean".to_string());
        m.insert("string".to_string(), "String".to_string());
        m.insert("bytes".to_string(), "String".to_string());
        m.insert("date".to_string(), "String".to_string());
        m
    };

    /// Forbidden enum variants for Rust (mainly for windows compatibility)
    static ref FORBIDDEN_RUST_ENUM_VARIANTS: Vec<String> = {
        vec![
            "yield".to_string(),
        ]
    };
}

/// Enumeration of available templates.
#[derive(Debug, ValueEnum, Clone, PartialEq)]
#[cfg_attr(feature = "python", pyclass(eq, eq_int))]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub enum Templates {
    /// XML Schema
    XmlSchema,
    /// Markdown
    Markdown,
    /// Compact Markdown
    CompactMarkdown,
    /// SHACL
    Shacl,
    /// JSON Schema
    JsonSchema,
    /// JSON Schema All
    JsonSchemaAll,
    /// SHACL
    Shex,
    /// Python Dataclass
    PythonDataclass,
    /// Python Pydantic XML
    PythonPydanticXML,
    /// Python Pydantic
    PythonPydantic,
    /// MkDocs
    MkDocs,
    /// Internal
    Internal,
    /// Typescript (io-ts)
    Typescript,
    /// Typescript (Zod)
    TypescriptZod,
    /// Rust
    Rust,
    /// Protobuf
    Protobuf,
    /// Graphql
    Graphql,
    /// Golang
    Golang,
    /// Linkml
    Linkml,
    /// Julia
    Julia,
    /// Mermaid class diagram
    Mermaid,
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
            Templates::TypescriptZod => write!(f, "typescript-zod"),
            Templates::Rust => write!(f, "rust"),
            Templates::Protobuf => write!(f, "protobuf"),
            Templates::Graphql => write!(f, "graphql"),
            Templates::Golang => write!(f, "golang"),
            Templates::Linkml => write!(f, "linkml"),
            Templates::Julia => write!(f, "julia"),
            Templates::Mermaid => write!(f, "mermaid"),
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
            "python-pydantic-xml" => Ok(Templates::PythonPydanticXML),
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
            "typescript-zod" => Ok(Templates::TypescriptZod),
            "rust" => Ok(Templates::Rust),
            "protobuf" => Ok(Templates::Protobuf),
            "graphql" => Ok(Templates::Graphql),
            "golang" => Ok(Templates::Golang),
            "linkml" => Ok(Templates::Linkml),
            "julia" => Ok(Templates::Julia),
            "mermaid" => Ok(Templates::Mermaid),
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

    // Keep track of fields that are artificially added,
    // but not part of the original model. Mainly used for
    // Database migrations and objects which do not have
    // a primary key.
    let mut artificial_fields = HashMap::new();

    // Perform type conversions and filtering based on the template
    match template {
        Templates::XmlSchema => convert_model_types(model, &XSD_TYPE_MAPS),
        Templates::Typescript => convert_model_types(model, &TYPESCRIPT_TYPE_MAPS),
        Templates::Graphql => convert_model_types(model, &GRAPHQL_TYPE_MAPS),
        Templates::Shacl | Templates::Shex => {
            convert_model_types(model, &SHACL_TYPE_MAPS);
            if let Err(e) = filter_objects_wo_terms(model) {
                println!(
                    " [{}] {}",
                    template.to_string().yellow().bold(),
                    e.to_string().bold(),
                );
            }
        }
        Templates::PythonDataclass | Templates::PythonPydanticXML | Templates::PythonPydantic => {
            convert_astropy_types(model, &config);
            convert_model_types(model, &PYTHON_TYPE_MAPS);
            sort_attributes_by_required(model);
        }
        Templates::Julia => {
            sort_by_dependency(model);
        }
        Templates::Golang => {
            if let Some(config) = config {
                if config.contains_key("gorm") {
                    add_id_pks(model, &mut artificial_fields);
                }
            }
        }
        Templates::Rust => {
            check_for_forbidden_rust_enum_variants(model);
        }
        _ => {}
    }

    // Add custom functions to the Jinja environment
    env.add_function("wrap", wrap_text);
    env.add_function("replace", replace);
    env.add_function("trim", trim);
    env.add_function("default_value", default_value);
    env.add_filter("enumerate", enumerate);
    env.add_filter("cap_first", cap_first);
    env.add_filter("split_path_pairs", split_path_pairs);
    env.add_filter("pascal_case", pascal_case);
    env.add_filter("camel_case", camel_case);
    env.add_filter("snake_case", snake_case);
    env.add_filter("replace_lower", replace_lower);

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
        Templates::TypescriptZod => env.get_template("typescript-zod.jinja")?,
        Templates::Rust => env.get_template("rust.jinja")?,
        Templates::Protobuf => env.get_template("protobuf.jinja")?,
        Templates::Graphql => env.get_template("graphql.jinja")?,
        Templates::Golang => env.get_template("golang.jinja")?,
        Templates::Julia => env.get_template("julia.jinja")?,
        Templates::Mermaid => env.get_template("mermaid.jinja")?,
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
        nsmap => model.config.as_ref().unwrap().nsmap.clone(),
        config => config,
        objects_with_wrapped => get_objects_with_wrapped(model),
        pk_objects => pk_objects(model),
        artificial_fields => artificial_fields,
        has_union_types => has_union_types(model),
    });

    match rendered {
        Ok(r) => Ok(clean_and_trim(&r)),
        Err(e) => Err(e),
    }
}

/// Returns a vector of object names that have attributes with XML wrapped types.
///
/// # Arguments
///
/// * `model` - The data model to search for wrapped objects
///
/// # Returns
///
/// A vector of strings containing the names of objects that have attributes with XML wrapped types
fn get_objects_with_wrapped(model: &mut DataModel) -> Vec<String> {
    model
        .objects
        .iter()
        .filter(|o| {
            o.attributes.iter().any(|a| {
                if let Some(xml) = &a.xml {
                    matches!(xml, XMLType::Wrapped { .. })
                } else {
                    false
                }
            })
        })
        .map(|o| o.name.clone())
        .collect()
}

/// Replaces all occurrences of a substring with another substring.
///
/// # Arguments
///
/// * `value` - The string to perform replacements on
/// * `from` - The substring to replace
/// * `to` - The substring to replace with
///
/// # Returns
///
/// A new string with all occurrences of `from` replaced with `to`
fn replace(value: String, from: &str, to: &str) -> String {
    value.replace(from, to)
}

/// Replaces all occurrences of a substring with another substring and converts the result to lowercase.
///
/// # Arguments
///
/// * `value` - The string to perform replacements on
/// * `from` - The substring to replace
/// * `to` - The substring to replace with
///
/// # Returns
///
/// A new string with all occurrences of `from` replaced with `to` and converted to lowercase
fn replace_lower(value: String, from: String, to: String) -> String {
    value.replace(&from, &to).to_lowercase()
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
fn wrap_text(
    text: &str,
    width: usize,
    initial_offset: &str,
    offset: &str,
    delimiter: Option<&str>,
) -> String {
    let delimiter = delimiter.unwrap_or("");
    // Remove multiple spaces
    let options = textwrap::Options::new(width)
        .initial_indent(initial_offset)
        .subsequent_indent(offset)
        .width(width)
        .break_words(false);

    wrap(remove_multiple_spaces(text).as_str(), options).join(&format!("{delimiter}\n"))
}

/// Splits a path into pairs of (current, previous) components.
///
/// # Arguments
///
/// * `path` - The path to split, using '/' as separator
/// * `initial` - The initial previous value to use for the first component
///
/// # Returns
///
/// A vector of tuples containing (current_component, previous_component)
fn split_path_pairs(path: String, initial: Option<String>) -> Vec<Vec<String>> {
    let initial = initial.unwrap_or_default();
    let parts: Vec<&str> = path.split('/').collect();
    let mut pairs = Vec::new();
    let mut prev = initial;

    for part in parts {
        if !part.is_empty() {
            pairs.push(vec![part.to_string(), prev.clone()]);
            prev = part.to_string();
        }
    }

    pairs
}

/// Filter use only for Jinja templates.
/// Converts a string to PascalCase.
fn pascal_case(s: String) -> String {
    if s.ends_with("_") {
        s.to_case(Case::Pascal) + "_"
    } else {
        s.to_case(Case::Pascal)
    }
}

/// Filter use only for Jinja templates.
/// Converts a string to camelCase.
fn camel_case(s: String) -> String {
    s.to_case(Case::Camel)
}

/// Filter use only for Jinja templates.
/// Converts a string to snake_case.
fn snake_case(s: String) -> String {
    s.to_case(Case::Snake)
}

/// Removes leading and trailing whitespace and multiple spaces from a string.
fn remove_multiple_spaces(input: &str) -> String {
    input.split_whitespace().collect::<Vec<&str>>().join(" ")
}

/// Removes trailing underscores from a string.
fn trim(input: &str, prefix: &str) -> String {
    input
        .trim_start_matches(prefix)
        .trim_end_matches(prefix)
        .to_string()
}

/// Checks if an object has a primary key.
fn pk_objects(model: &mut DataModel) -> HashMap<String, (String, String, bool)> {
    let mut pk_objects = HashMap::new();
    for object in &mut model.objects {
        for attribute in &object.attributes {
            for option in &attribute.options {
                if let AttrOption::PrimaryKey(true) = option {
                    pk_objects.insert(
                        object.name.clone(),
                        (attribute.name.clone(), attribute.dtypes[0].clone(), true),
                    );
                    break;
                }
            }
        }
    }
    pk_objects
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

/// Adds an ID field to objects in the model that don't have a primary key.
///
/// This function ensures that each object has a primary key by either:
/// 1. Adding a new 'id' attribute if the object has no primary key and no 'id' field
/// 2. Making an existing 'id' field a primary key if one exists but isn't already a primary key
///
/// # Arguments
///
/// * `model` - The data model to modify
fn add_id_pks(model: &mut DataModel, artificially_added_fields: &mut HashMap<String, String>) {
    for object in &mut model.objects {
        match (has_primary_key(object), has_id(object)) {
            (false, false) => {
                object.attributes.insert(0, id_attribute());
                artificially_added_fields.insert(object.name.clone(), "id".to_string());
            }
            (false, true) => {
                object
                    .attributes
                    .iter_mut()
                    .find(|a| a.name == "id")
                    .unwrap()
                    .options
                    .push(AttrOption::PrimaryKey(true));
            }
            _ => {}
        }
    }
}

/// Creates a new ID attribute with primary key option.
///
/// This function creates a new Attribute instance representing an ID field
/// with the following properties:
/// - Name: "id"
/// - Required: false
/// - Type: integer
/// - Primary key option enabled
///
/// # Returns
///
/// A new Attribute configured as an ID field
fn id_attribute() -> Attribute {
    let mut attr = Attribute::new("id".to_string(), true);
    attr.options.push(AttrOption::PrimaryKey(true));
    attr.set_dtype("integer".to_string()).unwrap();
    attr
}

/// Checks if an object has any attribute marked as a primary key.
///
/// # Arguments
///
/// * `object` - The object to check for primary key attributes
///
/// # Returns
///
/// `true` if the object has an attribute with primary key option, `false` otherwise
fn has_primary_key(object: &Object) -> bool {
    object
        .attributes
        .iter()
        .any(|a| a.options.iter().any(|o| o.key() == "primary key"))
}

/// Checks if an object has an attribute named "id".
///
/// # Arguments
///
/// * `object` - The object to check for an ID attribute
///
/// # Returns
///
/// `true` if the object has an attribute named "id", `false` otherwise
fn has_id(object: &Object) -> bool {
    object.attributes.iter().any(|a| a.name == "id")
}

/// Converts the data types in the model according to the provided type map.
///
/// # Arguments
///
/// * `model` - The data model whose types are to be converted.
fn convert_astropy_types(model: &mut DataModel, config: &Option<&HashMap<String, String>>) {
    if config.is_none() {
        return;
    }

    let config = config.unwrap();
    if !config.contains_key("astropy") {
        return;
    }

    // Replace UnitDefinition with UnitDefinitionAnnot
    for object in &mut model.objects {
        for attribute in &mut object.attributes {
            if attribute.dtypes.contains(&"UnitDefinition".to_string()) {
                attribute.dtypes = vec!["UnitDefinitionAnnot".to_string()];
            }
        }
    }

    model
        .objects
        .retain(|o| o.name != "UnitDefinition" && o.name != "BaseUnit");

    model.enums.retain(|e| e.name != "UnitType");
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
fn filter_objects_wo_terms(model: &mut DataModel) -> Result<(), Box<dyn Error>> {
    model.objects.retain(|o| o.has_any_terms());

    if model.objects.is_empty() {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "No objects with terms found in the model. Unable to build SHACL or ShEx.",
        )));
    }
    Ok(())
}

/// Checks for forbidden Rust enum variants and replaces them with a valid variant.
///
/// # Arguments
///
/// * `model` - The data model to check for forbidden Rust enum variants.
fn check_for_forbidden_rust_enum_variants(model: &mut DataModel) {
    for enumeration in &mut model.enums {
        enumeration.mappings = enumeration
            .mappings
            .iter()
            .map(|(key, value)| {
                let new_key = if FORBIDDEN_RUST_ENUM_VARIANTS.contains(&key.to_lowercase()) {
                    format!("{key}_")
                } else {
                    key.to_lowercase()
                };
                (new_key, value.clone())
            })
            .collect();
    }
}

/// Sorts the objects in the model by their dependency.
///
/// This is important for languages like Julia, where forward declarations
/// are not supported.
///
/// # Arguments
///
/// * `model` - The data model whose objects are to be sorted.
fn sort_by_dependency(model: &mut DataModel) {
    let graph = tree::dependency_graph(model);
    let mut class_order = tree::get_topological_order(&graph);
    class_order.reverse();
    model
        .objects
        .sort_by_key(|o| class_order.iter().position(|c| c == &o.name).unwrap());
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

/// Cleans and trims a string by removing trailing whitespace and limiting consecutive empty lines.
///
/// This function processes a string by:
/// 1. Splitting it into lines
/// 2. Trimming trailing whitespace from each line
/// 3. Limiting consecutive empty lines to a maximum of 2
/// 4. Joining the lines back together
///
/// # Arguments
///
/// * `s` - The string to clean and trim
///
/// # Returns
///
/// A cleaned string with trailing whitespace removed and consecutive empty lines limited
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
}

// Enumerate a collection of objects
pub fn enumerate(v: &Value, _: Kwargs) -> Result<Value, minijinja::Error> {
    if v.kind() != ValueKind::Seq {
        return Err(minijinja::Error::new(
            minijinja::ErrorKind::InvalidOperation,
            "Can only enumerate sequences",
        ));
    }

    // Turn into iterator of (index, value)
    Ok(v.try_iter()
        .expect("Failed to iterate over sequence")
        .enumerate()
        .map(|(i, v)| Value::from(vec![Value::from(i), v]))
        .collect())
}

/// Capitalizes the first character of a string.
///
/// # Arguments
///
/// * `s` - The string to capitalize
///
/// # Returns
///
/// A new string with the first character capitalized
fn cap_first(s: String) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
        None => s.to_string(),
    }
}

/// Formats the default value of an attribute.
///
/// # Arguments
///
/// * `default` - The default value of an attribute.
///
/// # Returns
fn default_value(attribute: ViaDeserialize<Attribute>) -> String {
    match &attribute.default {
        Some(DataType::String(s)) => format!("\"{}\"", s),
        Some(DataType::Integer(i)) => {
            if contains_numeric_type(&attribute) {
                i.to_string()
            } else {
                format!("\"{}\"", i)
            }
        }
        Some(DataType::Float(f)) => {
            if contains_numeric_type(&attribute) {
                f.to_string()
            } else {
                format!("\"{}\"", f)
            }
        }
        _ => "".to_string(),
    }
}

/// Checks if an attribute contains a numeric type.
///
/// # Arguments
///
/// * `attribute` - The attribute to check.
///
/// # Returns
///
/// `true` if the attribute contains a numeric type, `false` otherwise.
fn contains_numeric_type(attribute: &Attribute) -> bool {
    attribute
        .dtypes
        .iter()
        .any(|t| t == "integer" || t == "float")
}

/// Checks if an object has multiple types.
///
/// # Arguments
///
/// * `object` - The object to check.
///
/// # Returns
///
/// `true` if the object has union types, `false` otherwise.
fn has_union_types(model: &mut DataModel) -> bool {
    model
        .objects
        .iter()
        .any(|o| o.attributes.iter().any(|a| a.dtypes.len() > 1))
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use std::fs;

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
    fn build_and_convert(
        path: &str,
        template: Templates,
        config: Option<&HashMap<String, String>>,
    ) -> String {
        let content = fs::read_to_string(path).expect("Could not read markdown file");
        let mut model = parse_markdown(&content, None).expect("Failed to parse markdown file");
        render_jinja_template(&template, &mut model, config)
            .expect("Could not render template")
            .to_string()
    }

    #[test]
    fn test_convert_to_shex() {
        // Arrange
        let rendered = build_and_convert("tests/data/model.md", Templates::Shex, None);

        // Assert
        let expected = fs::read_to_string("tests/data/expected_shex.shex")
            .expect("Could not read expected file");
        assert_eq!(rendered, expected);
    }

    #[test]
    fn test_convert_to_shacl() {
        // Arrange
        let rendered = build_and_convert("tests/data/model.md", Templates::Shacl, None);

        // Assert
        let expected = fs::read_to_string("tests/data/expected_shacl.ttl")
            .expect("Could not read expected file");
        assert_eq!(rendered, expected);
    }

    #[test]
    fn test_convert_to_python_dc() {
        // Arrange
        let rendered = build_and_convert("tests/data/model.md", Templates::PythonDataclass, None);

        // Assert
        let expected = fs::read_to_string("tests/data/expected_python_dc.py")
            .expect("Could not read expected file");
        assert_eq!(rendered, expected);
    }

    #[test]
    fn test_convert_to_python_pydantic_xml() {
        // Arrange
        let rendered = build_and_convert("tests/data/model.md", Templates::PythonPydanticXML, None);

        // Assert
        let expected = fs::read_to_string("tests/data/expected_python_pydantic_xml.py")
            .expect("Could not read expected file");
        assert_eq!(rendered, expected);
    }

    #[test]
    fn test_convert_to_xsd() {
        // Arrange
        let rendered = build_and_convert("tests/data/model.md", Templates::XmlSchema, None);

        // Assert
        let expected = fs::read_to_string("tests/data/expected_xml_schema.xsd")
            .expect("Could not read expected file");
        assert_eq!(rendered, expected);
    }

    #[test]
    fn test_convert_to_mkdocs() {
        // Arrange
        let rendered = build_and_convert("tests/data/model.md", Templates::MkDocs, None);

        // Assert
        let expected = fs::read_to_string("tests/data/expected_mkdocs.md")
            .expect("Could not read expected file");
        assert_eq!(rendered, expected);
    }

    #[test]
    fn test_convert_to_typescript() {
        // Arrange
        let rendered = build_and_convert("tests/data/model.md", Templates::Typescript, None);

        // Assert
        let expected = fs::read_to_string("tests/data/expected_typescript.ts")
            .expect("Could not read expected file");
        assert_eq!(rendered, expected);
    }

    #[test]
    fn test_convert_to_typescript_zod() {
        // Arrange
        let rendered = build_and_convert("tests/data/model.md", Templates::TypescriptZod, None);

        // Assert
        let expected = fs::read_to_string("tests/data/expected_typescript_zod.ts")
            .expect("Could not read expected file");
        assert_eq!(rendered, expected);
    }

    #[test]
    fn test_convert_to_pydantic() {
        // Arrange
        let rendered = build_and_convert("tests/data/model.md", Templates::PythonPydantic, None);

        // Assert
        let expected = fs::read_to_string("tests/data/expected_pydantic.py")
            .expect("Could not read expected file");
        assert_eq!(rendered, expected);
    }

    #[test]
    fn test_convert_to_pydantic_unitdef() {
        // Arrange
        let rendered = build_and_convert(
            "tests/data/model_unitdef.md",
            Templates::PythonPydantic,
            Some(&HashMap::from([(
                "astropy".to_string(),
                "true".to_string(),
            )])),
        );

        // Assert
        let expected = fs::read_to_string("tests/data/expected_pydantic_unitdef.py")
            .expect("Could not read expected file");
        assert_eq!(rendered, expected);
    }

    #[test]
    fn test_convert_to_graphql() {
        // Arrange
        let rendered = build_and_convert("tests/data/model.md", Templates::Graphql, None);

        // Assert
        let expected = fs::read_to_string("tests/data/expected_graphql.graphql")
            .expect("Could not read expected file");
        assert_eq!(rendered, expected);
    }

    #[test]
    fn test_convert_to_golang() {
        // Arrange
        let rendered = build_and_convert("tests/data/model.md", Templates::Golang, None);

        // Assert
        let expected = fs::read_to_string("tests/data/expected_golang.go")
            .expect("Could not read expected file");
        assert_eq!(rendered, expected);
    }

    #[test]
    fn test_convert_to_golang_gorm() {
        // Arrange
        let rendered = build_and_convert(
            "tests/data/model_golang_gorm.md",
            Templates::Golang,
            Some(&HashMap::from([("gorm".to_string(), "true".to_string())])),
        );

        // Assert
        let expected = fs::read_to_string("tests/data/expected_golang_gorm.go")
            .expect("Could not read expected file");
        assert_eq!(rendered, expected);
    }

    #[test]
    fn test_convert_to_rust() {
        // Arrange
        let rendered = build_and_convert("tests/data/model.md", Templates::Rust, None);

        // Assert
        let expected = fs::read_to_string("tests/data/expected_rust.rs")
            .expect("Could not read expected file");
        assert_eq!(rendered, expected);
    }

    #[test]
    fn test_convert_to_rust_forbidden_names() {
        // Arrange
        let rendered =
            build_and_convert("tests/data/model_forbidden_names.md", Templates::Rust, None);

        // Assert
        let expected = fs::read_to_string("tests/data/expected_rust_forbidden.rs")
            .expect("Could not read expected file");
        assert_eq!(rendered, expected);
    }

    #[test]
    fn test_convert_to_protobuf() {
        // Arrange
        let rendered = build_and_convert("tests/data/model.md", Templates::Protobuf, None);

        // Assert
        let expected = fs::read_to_string("tests/data/expected_protobuf.proto")
            .expect("Could not read expected file");
        assert_eq!(rendered, expected);
    }

    #[test]
    fn test_convert_to_mermaid() {
        // Arrange
        let rendered = build_and_convert("tests/data/model.md", Templates::Mermaid, None);

        // Assert
        let expected = fs::read_to_string("tests/data/expected_mermaid.md")
            .expect("Could not read expected file");
        assert_eq!(rendered, expected);
    }
}
