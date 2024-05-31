use crate::datamodel::DataModel;
use clap::ValueEnum;
use lazy_static::lazy_static;
use minijinja::{context, Environment};

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
        m
    };

    /// Maps Python-specific type names to XSD-specific type names.
    static ref XSD_TYPE_MAPS: std::collections::HashMap<String, String> = {
        let mut m = std::collections::HashMap::new();
        m.insert("str".to_string(), "string".to_string());
        m
    };
}

/// Enumeration of available templates.
#[derive(Debug, ValueEnum, Clone)]
pub enum Templates {
    PythonDataclass,
    XmlSchema,
    Markdown,
    Shacl,
    JsonSchema,
    Shex,
    PythonSdrdm,
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
) -> Result<String, minijinja::Error> {
    // Load the template environment
    let mut env = Environment::new();
    minijinja_embed::load_templates!(&mut env);

    // Perform type conversions and filtering based on the template
    match template {
        Templates::Shacl | Templates::Shex => {
            convert_model_types(model, &SHACL_TYPE_MAPS);
            filter_objects_wo_terms(model);
        }
        Templates::XmlSchema => convert_model_types(model, &XSD_TYPE_MAPS),
        Templates::PythonDataclass | Templates::PythonSdrdm => {
            convert_model_types(model, &PYTHON_TYPE_MAPS);
            if let Templates::PythonSdrdm = template {
                sort_attributes_by_required(model);
            }
        }
        _ => {}
    }

    // Get the appropriate template
    let template = match template {
        Templates::PythonDataclass => env.get_template("python-dataclass.jinja")?,
        Templates::XmlSchema => env.get_template("xml-schema.jinja")?,
        Templates::Markdown => env.get_template("markdown.jinja")?,
        Templates::Shacl => env.get_template("shacl.jinja")?,
        Templates::JsonSchema => env.get_template("json-schema.jinja")?,
        Templates::Shex => env.get_template("shex.jinja")?,
        Templates::PythonSdrdm => env.get_template("python-sdrdm.jinja")?,
    };

    // Render the template
    let prefixes = get_prefixes(model);
    template.render(context! {
        objects => model.objects,
        object_names => model.objects.iter().map(|o| o.name.clone()).collect::<Vec<String>>(),
        enums => model.enums,
        enum_names => model.enums.iter().map(|e| e.name.clone()).collect::<Vec<String>>(),
        title => model.name,
        prefixes => prefixes,
        repo => model.config.as_ref().unwrap().repo.clone(),
        prefix => model.config.as_ref().unwrap().prefix.clone(),
    })
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
        object
            .attributes
            .sort_by(|a, b| b.required.cmp(&a.required));
    }
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
        let mut model = parse_markdown(path).expect("Failed to parse markdown file");
        render_jinja_template(&template, &mut model)
            .expect("Could not render template")
            .trim()
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
    fn test_convert_to_python_sdrdm() {
        // Arrange
        let rendered = build_and_convert(Templates::PythonSdrdm);

        // Assert
        let expected = fs::read_to_string("tests/data/expected_python_sdrdm.py")
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
}
