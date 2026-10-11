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

    /// Maps Python-specific type names to XSD-specific type names.
    static ref XSD_TYPE_MAPS: std::collections::HashMap<String, String> = {
        let mut m = std::collections::HashMap::new();
        m.insert("str".to_string(), "string".to_string());
        m.insert("bytes".to_string(), "base64Binary".to_string());
        m.insert("number".to_string(), "decimal".to_string());
        m.insert("date".to_string(), "date".to_string());
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

    /// Maps MD-Models type names to Owl-specific type names.
    static ref OWL_TYPE_MAPS: std::collections::HashMap<String, String> = {
        let mut m = std::collections::HashMap::new();
        m.insert("integer".to_string(), "xsd:integer".to_string());
        m.insert("number".to_string(), "xsd:decimal".to_string());
        m.insert("float".to_string(), "xsd:decimal".to_string());
        m.insert("boolean".to_string(), "xsd:boolean".to_string());
        m.insert("string".to_string(), "xsd:string".to_string());
        m.insert("bytes".to_string(), "xsd:base64Binary".to_string());
        m.insert("date".to_string(), "xsd:date".to_string());
        m
    };

    /// Maps MD-Models type names to the XSD datatypes JSON-LD produces, for SHACL.
    /// `number` and `float` stay unmapped: the template accepts xsd:integer or xsd:double,
    /// because JSON-LD types whole numbers such as `2.0` as xsd:integer.
    static ref SHACL_TYPE_MAPS: std::collections::HashMap<String, String> = {
        let mut m = std::collections::HashMap::new();
        m.insert("integer".to_string(), "xsd:integer".to_string());
        m.insert("boolean".to_string(), "xsd:boolean".to_string());
        m.insert("string".to_string(), "xsd:string".to_string());
        m.insert("bytes".to_string(), "xsd:string".to_string());
        m.insert("date".to_string(), "xsd:date".to_string());
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
#[cfg_attr(feature = "python", pyclass(eq, eq_int, from_py_object))]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub enum Templates {
    /// XML Schema
    XmlSchema,
    /// Markdown
    Markdown,
    /// Compact Markdown
    CompactMarkdown,
    /// JSON Schema
    JsonSchema,
    /// JSON Schema All
    JsonSchemaAll,
    /// JSON-LD
    JsonLd,
    /// SHACL
    Shacl,
    /// OWL
    Owl,
    /// ShEx
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
            Templates::JsonLd => write!(f, "json-ld"),
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
            Templates::Owl => write!(f, "owl"),
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
            "owl" => Ok(Templates::Owl),
            _ => {
                let err = format!("Invalid template type: {s}");
                Err(err.into())
            }
        }
    }
}

/// Warns on stderr when `config` still uses the placeholder `repo`, because
/// IRIs generated for `target` would point into a namespace the user does not own.
pub(crate) fn warn_placeholder_repo(config: &FrontMatter, target: &str) {
    if config.has_placeholder_repo() {
        eprintln!(
            " [{}] {}",
            target.yellow().bold(),
            format!(
                "No `repo` set in the front matter; using placeholder {} for IRIs. Set `repo` to a namespace you control.",
                config.repo
            )
            .bold(),
        );
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

    // If there is no config, create an empty one
    // This is necessary to avoid errors when rendering the template
    let config = config.cloned().unwrap_or_default();

    // Perform type conversions and filtering based on the template
    match template {
        Templates::XmlSchema => convert_model_types(model, &XSD_TYPE_MAPS),
        Templates::Typescript => convert_model_types(model, &TYPESCRIPT_TYPE_MAPS),
        Templates::Graphql => convert_model_types(model, &GRAPHQL_TYPE_MAPS),
        // Every field is in the JSON-LD graph (unmapped ones as Object:attr), so SHACL keeps all objects
        Templates::Shacl => convert_model_types(model, &SHACL_TYPE_MAPS),
        Templates::Shex => {
            convert_model_types(model, &OWL_TYPE_MAPS);
            if let Err(e) = filter_objects_wo_terms(model) {
                eprintln!(
                    " [{}] {}",
                    template.to_string().yellow().bold(),
                    e.to_string().bold(),
                );
            }
        }
        Templates::Owl => {
            convert_model_types(model, &OWL_TYPE_MAPS);
            remove_default_prefixes(model);
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
            if config.contains_key("gorm") {
                add_id_pks(model, &mut artificial_fields);
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
    env.add_function("json_string", json_string);
    env.add_function("xml_escape", xml_escape);
    env.add_test("gen_delim_iri", |iri: String| {
        crate::jsonld::export::ends_with_gen_delim(&iri)
    });
    env.add_filter("enumerate", enumerate);
    env.add_filter("cap_first", cap_first);
    env.add_filter("split_path_pairs", split_path_pairs);
    env.add_filter("pascal_case", pascal_case);
    env.add_filter("camel_case", camel_case);
    env.add_filter("snake_case", snake_case);
    env.add_filter("to_identifier", to_identifier);
    env.add_filter("replace_lower", replace_lower);

    // Semantic formats mint IRIs from `repo`, so warn when it is the placeholder
    let template_name = template.to_string();
    let semantic = matches!(
        template,
        Templates::Owl
            | Templates::Shacl
            | Templates::Shex
            | Templates::PythonPydantic
            | Templates::PythonDataclass
    ) || (matches!(template, Templates::Rust) && config.contains_key("jsonld"));

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
        Templates::Owl => env.get_template("owl.jinja")?,
        _ => {
            panic!(
                "The template is not available as a Jinja Template and should not be used using the jinja exporter.
                Instead, use the dedicated exporter in the DataModel struct (e.g. `DataModel::json_ld_header`, DataModel::json_schema)."
            )
        }
    };

    // If there is no config, create an empty one
    // This is necessary to avoid errors when rendering the template
    if model.config.is_none() {
        model.config = Some(FrontMatter::default());
    }

    if semantic {
        warn_placeholder_repo(model.config.as_ref().unwrap(), &template_name);
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
        repo_ns => crate::jsonld::export::model_namespace(&model.config.as_ref().unwrap().repo),
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

/// Filter used only for Jinja templates.
///
/// Converts a name into a native-safe identifier by replacing any character that
/// is not alphanumeric or an underscore (most importantly dashes, e.g. in
/// `coupling-scheme`) with an underscore. The original name is preserved
/// separately in the templates as a serialization alias, so this only affects the
/// generated programming-language identifier, never the wire/serialized name.
fn to_identifier(s: String) -> String {
    s.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// Removes leading and trailing whitespace and multiple spaces from a string.
fn remove_multiple_spaces(input: &str) -> String {
    input.split_whitespace().collect::<Vec<&str>>().join(" ")
}

/// Escapes text for use in XML attribute values and text nodes.
fn xml_escape(text: Value) -> String {
    let raw = text.to_string();
    let mut out = String::with_capacity(raw.len());
    for c in raw.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(c),
        }
    }
    out
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
fn convert_astropy_types(model: &mut DataModel, config: &HashMap<String, String>) {
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
    let mut prefixes = match &model.config {
        Some(config) => config.prefixes().unwrap_or(vec![]),
        None => vec![],
    };

    prefixes.sort_by(|a, b| a.0.cmp(&b.0));
    prefixes
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
            "No objects with terms found in the model. Unable to build ShEx.",
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

/// Renders a string as a JSON string literal, which is also a valid Python literal.
fn json_string(value: String) -> String {
    serde_json::to_string(&value).expect("a string always serializes")
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
        Some(DataType::String(s)) => serde_json::to_string(s).expect("a string always serializes"),
        Some(DataType::Integer(i)) => {
            if contains_numeric_type(&attribute) {
                i.to_string()
            } else {
                format!("\"{i}\"")
            }
        }
        Some(DataType::Float(f)) => {
            // `{:?}` keeps the decimal point: 2.0 renders as "2.0", not "2"
            if contains_numeric_type(&attribute) {
                format!("{f:?}")
            } else {
                format!("\"{f:?}\"")
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

/// Removes the default prefixes from the model.
///
/// # Arguments
///
/// * `model` - The data model whose prefixes are to be removed.
fn remove_default_prefixes(model: &mut DataModel) {
    if let Some(prefixes) = &mut model.config.as_mut().unwrap().prefixes {
        prefixes.remove("xsd");
        prefixes.remove("rdfs");
        prefixes.remove("owl");
    }
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
    fn test_pydantic_emits_unquoted_pattern() {
        let rendered = build_and_convert(
            "tests/data/model_jsonld.md",
            Templates::PythonPydantic,
            None,
        );
        assert!(
            rendered.contains(r#"pattern="^(\\d+)\\.\\d+$","#),
            "pattern missing in:\n{rendered}"
        );
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
    fn test_shacl_matches_jsonld() {
        let rendered = build_and_convert("tests/data/model_jsonld.md", Templates::Shacl, None);
        for expected in [
            "@prefix ex: <https://example.org/jsonld/> .",
            "@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .",
            "@prefix Document: <https://example.org/jsonld/Document/> .",
            "@prefix Molecule: <https://example.org/jsonld/Molecule/> .",
            "@prefix Kind: <https://example.org/jsonld/Kind/> .",
            "sh:path schema:identifier ;",
            "sh:path ( Molecule:values [ sh:zeroOrMorePath rdf:rest ] rdf:first ) ;",
            "sh:path ( schema:citation [ sh:zeroOrMorePath rdf:rest ] rdf:first ) ;",
            "sh:path ( Document:molecules [ sh:zeroOrMorePath rdf:rest ] rdf:first ) ;",
            "sh:nodeKind sh:IRI ;",
            r#"sh:in ( "assay" "simulation" ) ;"#,
            "sh:or ( [ sh:datatype xsd:integer ] [ sh:datatype xsd:double ] ) ;",
            "sh:datatype xsd:date ;",
            "sh:node ex:MoleculeShape ;",
        ] {
            assert!(
                rendered.contains(expected),
                "missing {expected:?} in:\n{rendered}"
            );
        }
        // The IRI-valued homepage carries no datatype, only a node kind
        let homepage = rendered
            .split("sh:property [")
            .find(|p| p.contains("sh:path schema:url ;"))
            .expect("homepage shape");
        assert!(homepage.contains("sh:nodeKind sh:IRI ;") && !homepage.contains("sh:datatype"));
        // Lists have no upper bound
        let values = rendered
            .split("sh:property [")
            .find(|p| p.contains("Molecule:values"))
            .expect("values shape");
        assert!(!values.contains("sh:maxCount"));
        // One path per property shape, no path alternatives
        assert!(
            !rendered.contains("[ sh:path"),
            "path alternatives in:\n{rendered}"
        );
        assert!(!rendered.contains("decimal"));
        // Whole-number floats are integers to a JSON-LD processor
        assert!(!rendered.contains("sh:datatype xsd:double ;"), "{rendered}");
    }

    #[test]
    fn test_shacl_bytes_are_plain_strings() {
        let rendered = render_inline(
            "---\nrepo: \"https://ex.org/\"\nprefix: ex\n---\n\n### A\n\n- blob\n  - Type: bytes\n",
            Templates::Shacl,
        );
        assert!(rendered.contains("sh:datatype xsd:string ;"), "{rendered}");
        assert!(!rendered.contains("base64Binary"), "{rendered}");
    }

    #[test]
    fn test_date_coercion_only_for_pure_date_fields() {
        let md = "---\nrepo: \"https://ex.org/\"\nprefix: ex\n---\n\n### A\n\n- both\n  - Type: date, string\n- only\n  - Type: date\n";
        let date = "http://www.w3.org/2001/XMLSchema#date";
        for template in [Templates::PythonPydantic, Templates::PythonDataclass] {
            let rendered = render_inline(md, template);
            assert_eq!(rendered.matches(date).count(), 1, "{rendered}");
            assert!(
                rendered.contains("\"only\": {\n                \"@id\": \"ex:A/only\",\n                \"@type\""),
                "{rendered}"
            );
            assert!(rendered.contains("\"both\": \"ex:A/both\","), "{rendered}");
        }
        let mut model = parse_markdown(md, None).expect("parse");
        let rust = render_jinja_template(
            &Templates::Rust,
            &mut model,
            Some(&HashMap::from([("jsonld".to_string(), "true".to_string())])),
        )
        .expect("render");
        assert_eq!(rust.matches(date).count(), 1, "{rust}");
    }

    fn render_inline(content: &str, template: Templates) -> String {
        let mut model = parse_markdown(content, None).expect("Failed to parse markdown");
        render_jinja_template(&template, &mut model, None).expect("Could not render template")
    }

    #[test]
    fn test_shacl_keeps_objects_without_terms() {
        let rendered = render_inline(
            "---\nrepo: \"https://ex.org/\"\nprefix: ex\n---\n\n### A\n\n- b\n  - Type: B\n  - Term: schema:b\n\n### B\n\n- x\n  - Type: string\n",
            Templates::Shacl,
        );
        assert!(
            rendered.contains("ex:BShape\n"),
            "no BShape in:\n{rendered}"
        );
        assert!(rendered.contains("sh:node ex:BShape ;"), "{rendered}");
        assert!(rendered.contains("sh:path B:x ;"), "{rendered}");
        assert!(!rendered.contains("sh:datatype B"), "{rendered}");
    }

    #[test]
    fn test_shacl_shared_term_has_no_max_count() {
        let rendered = render_inline(
            "---\nrepo: \"https://ex.org/\"\nprefix: ex\n---\n\n### A\n\n- id\n  - Type: string\n  - Term: schema:identifier\n- vessel_id\n  - Type: string\n  - Term: schema:identifier\n- name\n  - Type: string\n",
            Templates::Shacl,
        );
        let shapes: Vec<&str> = rendered.split("sh:property [").skip(1).collect();
        assert_eq!(shapes.len(), 3, "{rendered}");
        for shape in &shapes[..2] {
            assert!(shape.contains("sh:path schema:identifier ;"), "{shape}");
            assert!(!shape.contains("sh:maxCount"), "{shape}");
        }
        assert!(shapes[2].contains("sh:path A:name ;"), "{rendered}");
        assert!(shapes[2].contains("sh:maxCount 1 ;"), "{rendered}");
    }

    /// Block of the OWL property `name` (up to its terminating " .").
    fn owl_block<'a>(rendered: &'a str, name: &str) -> &'a str {
        let start = rendered
            .find(&format!("\n{name}\n"))
            .unwrap_or_else(|| panic!("no {name} in:\n{rendered}"));
        let rest = &rendered[start + 1..];
        &rest[..rest.find(" .\n").expect("unterminated block")]
    }

    #[test]
    fn test_owl_lists_and_iris_are_object_properties_without_range() {
        let rendered = build_and_convert("tests/data/model_jsonld.md", Templates::Owl, None);

        let values = owl_block(&rendered, "Molecule:values");
        assert!(values.contains("a owl:ObjectProperty"), "{values}");
        assert!(!values.contains("rdfs:range"), "{values}");
        assert!(
            values.contains(r#"rdfs:comment "Ordered list (rdf:List) of xsd:decimal.""#),
            "{values}"
        );

        let molecules = owl_block(&rendered, "Document:molecules");
        assert!(molecules.contains("a owl:ObjectProperty"), "{molecules}");
        assert!(!molecules.contains("rdfs:range"), "{molecules}");
        assert!(
            molecules.contains("Ordered list (rdf:List) of ex:Molecule."),
            "{molecules}"
        );

        let homepage = owl_block(&rendered, "Document:homepage");
        assert!(homepage.contains("a owl:ObjectProperty"), "{homepage}");
        assert!(!homepage.contains("rdfs:range"), "{homepage}");

        let label = owl_block(&rendered, "Document:label");
        assert!(label.contains("a owl:DatatypeProperty"), "{label}");
        assert!(label.contains("rdfs:range xsd:string"), "{label}");
    }

    #[test]
    fn test_owl_required_arrays_use_min_cardinality_without_filler() {
        let rendered = render_inline(
            "---\nrepo: \"https://ex.org/\"\nprefix: ex\n---\n\n### A\n\n- **xs**\n  - Type: string[]\n- **bs**\n  - Type: B[]\n\n### B\n\n- x\n  - Type: string\n",
            Templates::Owl,
        );
        for prop in ["A:xs", "A:bs"] {
            let restriction = format!(
                "owl:onProperty {prop}\n        ; owl:minCardinality \"1\"^^xsd:nonNegativeInteger"
            );
            assert!(rendered.contains(&restriction), "{prop} in:\n{rendered}");
        }
        assert!(!rendered.contains("owl:someValuesFrom"), "{rendered}");
    }

    #[test]
    fn test_owl_list_comment_keeps_docstring() {
        let rendered = render_inline(
            "---\nrepo: \"https://ex.org/\"\nprefix: ex\n---\n\n### A\n\n- xs\n  - Type: string[]\n  - Description: Some values.\n",
            Templates::Owl,
        );
        let xs = owl_block(&rendered, "A:xs");
        assert!(
            xs.contains(r#"rdfs:comment "Ordered list (rdf:List) of xsd:string. Some values.""#),
            "{xs}"
        );
    }

    #[test]
    fn test_hash_terminated_repo_has_single_hash_in_owl_and_shex() {
        for template in [Templates::Owl, Templates::Shex] {
            let rendered = render_inline(
                "---\nrepo: \"https://example.org/x#\"\nprefix: ex\n---\n\n### A\n\n- b\n  - Type: string\n  - Term: schema:b\n",
                template,
            );
            assert!(
                rendered.contains("ex: <https://example.org/x#>"),
                "{rendered}"
            );
            assert!(
                rendered.contains("<https://example.org/x#A/>"),
                "{rendered}"
            );
            assert!(!rendered.contains("x##"), "{rendered}");
        }
    }

    #[test]
    fn test_convert_to_owl() {
        // Arrange
        let rendered = build_and_convert("tests/data/model.md", Templates::Owl, None);

        // Assert
        let expected = fs::read_to_string("tests/data/expected_owl.ttl")
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
    fn test_convert_to_typescript_zod_json_ld() {
        // Arrange
        let rendered = build_and_convert(
            "tests/data/model.md",
            Templates::TypescriptZod,
            Some(&HashMap::from([(
                "json-ld".to_string(),
                "true".to_string(),
            )])),
        );

        // Assert
        let expected = fs::read_to_string("tests/data/expected_typescript_zod_json_ld.ts")
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
    fn test_convert_to_rust_ld() {
        // Arrange
        let rendered = build_and_convert(
            "tests/data/model.md",
            Templates::Rust,
            Some(&HashMap::from([("jsonld".to_string(), "true".to_string())])),
        );

        // Assert
        let expected = fs::read_to_string("tests/data/expected_rust_ld.rs")
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
    fn test_markdown_export_marks_arrays() {
        // Array attributes must render the `[]` marker so the exported markdown
        // round-trips back to an array (e.g. `Type2[]`, `string[]`), rather than
        // silently dropping the multiplicity.
        let rendered = build_and_convert("tests/data/model.md", Templates::Markdown, None);

        assert!(
            rendered.contains("Type: Test2[]"),
            "array of object-typed attribute should render with []"
        );
        assert!(
            rendered.contains("Type: string[]"),
            "array of scalar-typed attribute should render with []"
        );

        // A re-parse of the exported markdown must preserve the array flag.
        let reparsed = parse_markdown(&rendered, None).expect("exported markdown should re-parse");
        let test = reparsed
            .objects
            .iter()
            .find(|o| o.name == "Test")
            .expect("Test object");
        let test2 = test
            .attributes
            .iter()
            .find(|a| a.name == "test2")
            .expect("test2 attribute");
        assert!(
            test2.is_array,
            "array flag must survive a markdown round-trip"
        );
    }

    #[test]
    fn test_markdown_export_marks_required_and_defaults() {
        // Required attributes render as bold (`__name__`) and default values render
        // as a `- Default:` line, so neither is silently dropped.
        let rendered = build_and_convert("tests/data/model.md", Templates::Markdown, None);

        assert!(
            rendered.contains("- __name__"),
            "required attribute should render in bold"
        );
        assert!(
            rendered.contains("- Default: 2.0"),
            "default value should render"
        );

        // Required flag and default survive a re-parse of the exported markdown.
        let reparsed = parse_markdown(&rendered, None).expect("exported markdown should re-parse");
        let name = reparsed
            .objects
            .iter()
            .find(|o| o.name == "Test")
            .and_then(|o| o.attributes.iter().find(|a| a.name == "name"))
            .expect("name attribute");
        assert!(
            name.required,
            "required flag must survive a markdown round-trip"
        );
        assert!(
            name.default.is_some(),
            "default must survive a markdown round-trip"
        );
    }

    #[test]
    fn test_dashed_attribute_names_rust() {
        // Attribute names containing dashes (e.g. `coupling-scheme`) are converted
        // to native-safe Rust identifiers, with a serde rename preserving the
        // original wire name for both serialization and deserialization.
        let rendered = build_and_convert("tests/data/model_dashed_names.md", Templates::Rust, None);

        assert!(
            rendered.contains("pub coupling_scheme:"),
            "dash should be converted to an underscore in the Rust field name"
        );
        assert!(
            rendered.contains("#[serde(rename = \"coupling-scheme\""),
            "original dashed name should be preserved via serde rename"
        );
        // Union-typed dashed attribute produces a valid enum type name.
        assert!(rendered.contains("pub enum ConfigurationData_valueType"));
        // Builder setter for the dashed list attribute uses a safe identifier.
        assert!(rendered.contains("name = \"to_sub_items\""));
        // The dashed name must never leak into a Rust identifier position.
        assert!(!rendered.contains("coupling-scheme:"));
    }

    #[test]
    fn test_dashed_attribute_names_pydantic() {
        // Pydantic models use a safe field name plus a Field alias that maps to the
        // original dashed wire name.
        let rendered = build_and_convert(
            "tests/data/model_dashed_names.md",
            Templates::PythonPydantic,
            None,
        );

        assert!(rendered.contains("coupling_scheme: "));
        assert!(rendered.contains("alias=\"coupling-scheme\""));
        // populate_by_name lets the model still be constructed via the safe name.
        assert!(rendered.contains("populate_by_name = True"));
    }

    #[test]
    fn test_dashed_attribute_names_dataclass() {
        // Dataclasses map the dashed wire name through dataclasses-json `field_name`.
        let rendered = build_and_convert(
            "tests/data/model_dashed_names.md",
            Templates::PythonDataclass,
            None,
        );

        assert!(rendered.contains("coupling_scheme:"));
        assert!(rendered.contains("field_name=\"coupling-scheme\""));
    }

    #[test]
    fn test_dashed_attribute_names_typescript() {
        // TypeScript preserves the wire name by quoting the property key.
        let rendered = build_and_convert(
            "tests/data/model_dashed_names.md",
            Templates::Typescript,
            None,
        );

        assert!(rendered.contains("\"coupling-scheme\"?:"));
        assert!(rendered.contains("\"coupling-scheme\": D.nullable"));
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

    #[test]
    fn test_string_defaults_keep_their_text() {
        let pydantic = build_and_convert(
            "tests/data/model_jsonld.md",
            Templates::PythonPydantic,
            None,
        );
        assert!(pydantic.contains(r#"default= "2.0","#), "{pydantic}");

        let rust = build_and_convert("tests/data/model.md", Templates::Rust, None);
        assert!(
            rust.contains(r#"#[builder(default = "\"2.0\".to_string().into()", setter(into))]"#),
            "{rust}"
        );
    }

    #[test]
    fn test_string_defaults_are_escaped_in_rust_and_python() {
        let md = "### Doc\n\n- path\n  - Type: string\n  - Default: \"say \"hi\" C:\\tmp\"\n";
        let mut model = parse_markdown(md, None).expect("Failed to parse markdown");
        let rust = render_jinja_template(&Templates::Rust, &mut model, None)
            .expect("Could not render template")
            .to_string();
        // The Rust string literal is escaped once, the attribute argument a second time
        assert!(
            rust.contains(
                r#"#[builder(default = "\"say \\\"hi\\\" C:\\\\tmp\".to_string().into()", setter(into))]"#
            ),
            "{rust}"
        );
        assert!(
            rust.contains(
                r#"#[derivative(Default(value = "\"say \\\"hi\\\" C:\\\\tmp\".to_string()"))]"#
            ),
            "{rust}"
        );

        let mut model = parse_markdown(md, None).expect("Failed to parse markdown");
        let py = render_jinja_template(&Templates::PythonPydantic, &mut model, None)
            .expect("Could not render template")
            .to_string();
        assert!(py.contains(r#"default= "say \"hi\" C:\\tmp","#), "{py}");
    }

    #[test]
    fn test_pydantic_pattern_only_on_string_fields() {
        let md = "---\nprefix: ex\n---\n\n### Doc\n\n- name\n  - Type: string\n  - Pattern: ^a\n- count\n  - Type: integer\n  - Pattern: ^b\n- value\n  - Type: float, string\n  - Pattern: ^c\n";
        let mut model = parse_markdown(md, None).expect("Failed to parse markdown");
        let py = render_jinja_template(&Templates::PythonPydantic, &mut model, None)
            .expect("Could not render template")
            .to_string();
        assert!(py.contains(r#"pattern="^a","#), "{py}");
        assert!(!py.contains(r#"pattern="^b""#), "{py}");
        assert!(!py.contains(r#"pattern="^c""#), "{py}");
    }

    #[test]
    fn test_acronym_type_names_are_kept() {
        let md = "---\nprefix: ex\n---\n\n### EnzymeMLDocument\n\n- measurements\n  - Type: SBML measurement[]\n- role\n  - Type: ModifierRoleML\n\n### SBML measurement\n\n- id\n  - Type: string\n\n## Enumerations\n\n### ModifierRoleML\n\n```\nACTIVATOR = \"activator\"\n```\n";
        let mut model = parse_markdown(md, None).expect("Failed to parse markdown");

        let names: Vec<&str> = model.objects.iter().map(|o| o.name.as_str()).collect();
        assert_eq!(names, vec!["EnzymeMLDocument", "SBMLMeasurement"]);
        assert_eq!(model.enums[0].name, "ModifierRoleML");
        assert_eq!(
            model.objects[0].attributes[0].dtypes,
            vec!["SBMLMeasurement"]
        );
        assert_eq!(
            model.objects[0].attributes[1].dtypes,
            vec!["ModifierRoleML"]
        );

        let py = render_jinja_template(&Templates::PythonPydantic, &mut model, None)
            .expect("Could not render template")
            .to_string();
        assert!(py.contains("class EnzymeMLDocument(BaseModel):"), "{py}");
        assert!(py.contains("class ModifierRoleML(Enum):"), "{py}");
    }

    #[test]
    fn test_slashless_repo_gets_hash_namespace() {
        let md = "---\nrepo: \"https://example.org/repo\"\nprefix: ex\n---\n\n### Doc\n\n- id\n  - Type: string\n";
        let mut model = parse_markdown(md, None).expect("Failed to parse markdown");
        let py = render_jinja_template(&Templates::PythonPydantic, &mut model, None)
            .expect("Could not render template")
            .to_string();
        assert!(py.contains(r#""ex": "https://example.org/repo#","#), "{py}");
        assert!(
            py.contains(r#""@vocab": "https://example.org/repo#","#),
            "{py}"
        );
    }

    #[test]
    fn test_jsonld_prefixes_without_gen_delim_opt_in() {
        for template in [Templates::PythonPydantic, Templates::PythonDataclass] {
            let rendered = build_and_convert("tests/data/model_jsonld.md", template, None);
            assert!(
                rendered.contains(
                    r#""CHEBI": {"@id": "http://purl.obolibrary.org/obo/CHEBI_", "@prefix": True},"#
                ),
                "{rendered}"
            );
            assert!(
                rendered.contains(r#""rdfs": "http://www.w3.org/2000/01/rdf-schema#","#),
                "{rendered}"
            );
        }

        let rust = build_and_convert(
            "tests/data/model_jsonld.md",
            Templates::Rust,
            Some(&HashMap::from([("jsonld".to_string(), "true".to_string())])),
        );
        assert!(
            rust.contains(r#"context.terms.insert("CHEBI".to_string(), TermDef::Detailed(TermDetail { id: Some("http://purl.obolibrary.org/obo/CHEBI_".to_string()), prefix: Some(true), ..Default::default() }));"#),
            "{rust}"
        );
    }

    #[test]
    fn test_jsonld_contexts_map_every_field() {
        for template in [Templates::PythonPydantic, Templates::PythonDataclass] {
            let rendered = build_and_convert("tests/data/model_jsonld.md", template, None);
            for expected in [
                r#""@vocab": "https://example.org/jsonld/","#,
                r#""id": "schema:identifier","#,
                r#""vessel_id": "ex:Molecule/vessel_id","#,
                r#""@id": "ex:Document/molecules","#,
                r#""@id": "schema:url","#,
                r#""@id": "schema:citation","#,
            ] {
                assert!(
                    rendered.contains(expected),
                    "missing `{expected}` in:\n{rendered}"
                );
            }
            assert!(
                !rendered.contains(r#""@id": "schema:identifier","#),
                "identifier coerced to @id:\n{rendered}"
            );
        }

        let rust = build_and_convert(
            "tests/data/model_jsonld.md",
            Templates::Rust,
            Some(&HashMap::from([("jsonld".to_string(), "true".to_string())])),
        );
        for expected in [
            r#"context.vocab = Some("https://example.org/jsonld/".to_string());"#,
            r#"context.terms.insert("id".to_string(), TermDef::Simple("schema:identifier".to_string()));"#,
            r#"context.terms.insert("vessel_id".to_string(), TermDef::Simple("ex:Molecule/vessel_id".to_string()));"#,
            r#"id: Some("schema:url".to_string()),"#,
        ] {
            assert!(rust.contains(expected), "missing `{expected}` in:\n{rust}");
        }
    }

    #[test]
    fn test_pydantic_reads_jsonld_keywords() {
        let rendered = build_and_convert(
            "tests/data/model_jsonld.md",
            Templates::PythonPydantic,
            None,
        );
        for expected in [
            r#"alias="@id","#,
            r#"alias="@type","#,
            r#"alias="@context","#,
            r#"@field_validator("ld_context", mode="before")"#,
            r#"@field_validator("ld_type", mode="before")"#,
            r#"@field_validator("ld_id", mode="before")"#,
        ] {
            assert!(
                rendered.contains(expected),
                "missing `{expected}` in:\n{rendered}"
            );
        }
        assert!(!rendered.contains("serialization_alias=\"@"), "{rendered}");
    }

    #[test]
    fn test_jsonld_contexts_use_lists_and_typed_dates() {
        for template in [Templates::PythonPydantic, Templates::PythonDataclass] {
            let rendered = build_and_convert("tests/data/model_jsonld.md", template, None);
            for expected in [
                "\"values\": {\n                \"@id\": \"ex:Molecule/values\",\n                \"@container\": \"@list\",\n            },",
                "\"created\": {\n                \"@id\": \"ex:Document/created\",\n                \"@type\": \"http://www.w3.org/2001/XMLSchema#date\",\n            },",
            ] {
                assert!(
                    rendered.contains(expected),
                    "missing `{expected}` in:\n{rendered}"
                );
            }
        }

        let rust = build_and_convert(
            "tests/data/model_jsonld.md",
            Templates::Rust,
            Some(&HashMap::from([("jsonld".to_string(), "true".to_string())])),
        );
        assert!(
            rust.contains(r#"type_: Some("http://www.w3.org/2001/XMLSchema#date".to_string()),"#),
            "{rust}"
        );
    }
}
