use crate::datamodel::DataModel;
use clap::ValueEnum;
use lazy_static::lazy_static;
use minijinja::{context, Environment};

lazy_static! {
    static ref PYTHON_TYPE_MAPS: std::collections::HashMap<String, String> = {
        let mut m = std::collections::HashMap::new();
        m.insert("string".to_string(), "str".to_string());
        m.insert("integer".to_string(), "int".to_string());
        m.insert("boolean".to_string(), "bool".to_string());
        m
    };
    static ref SHACL_TYPE_MAPS: std::collections::HashMap<String, String> = {
        let mut m = std::collections::HashMap::new();
        m.insert("float".to_string(), "double".to_string());
        m
    };
    static ref XSD_TYPE_MAPS: std::collections::HashMap<String, String> = {
        let mut m = std::collections::HashMap::new();
        m.insert("str".to_string(), "string".to_string());
        m
    };
}

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

pub fn render_jinja_template(
    template: &Templates,
    model: &mut DataModel,
) -> Result<String, minijinja::Error> {
    // Load the template
    let mut env = Environment::new();
    minijinja_embed::load_templates!(&mut env);

    // Type conversions and filtering
    match template {
        Templates::Shacl => {
            convert_model_types(model, &SHACL_TYPE_MAPS);
            filter_objects_wo_terms(model);
        }
        Templates::Shex => {
            convert_model_types(model, &SHACL_TYPE_MAPS);
            filter_objects_wo_terms(model);
        }
        Templates::XmlSchema => convert_model_types(model, &XSD_TYPE_MAPS),
        Templates::PythonDataclass => convert_model_types(model, &PYTHON_TYPE_MAPS),
        Templates::PythonSdrdm => {
            convert_model_types(model, &PYTHON_TYPE_MAPS);
            sort_attributes_by_required(model);
        }
        _ => {}
    }

    // Get the template
    let template = match template {
        Templates::PythonDataclass => env.get_template("python-dataclass.jinja").unwrap(),
        Templates::XmlSchema => env.get_template("xml-schema.jinja").unwrap(),
        Templates::Markdown => env.get_template("markdown.jinja").unwrap(),
        Templates::Shacl => env.get_template("shacl.jinja").unwrap(),
        Templates::JsonSchema => env.get_template("json-schema.jinja").unwrap(),
        Templates::Shex => env.get_template("shex.jinja").unwrap(),
        Templates::PythonSdrdm => env.get_template("python-sdrdm.jinja").unwrap(),
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

fn get_prefixes(model: &mut DataModel) -> Vec<(String, String)> {
    match &model.config {
        Some(config) => config.prefixes().unwrap_or(vec![]),
        None => vec![],
    }
}

fn filter_objects_wo_terms(model: &mut DataModel) {
    model.objects.retain(|o| o.has_any_terms());

    if model.objects.is_empty() {
        panic!("No objects with terms found in the model. Unable to build SHACL or ShEx.");
    }
}

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

    // Helper function to build and convert a template
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
