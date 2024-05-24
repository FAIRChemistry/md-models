use clap::{Parser, ValueEnum};
use lazy_static::lazy_static;
use mdmodels::{datamodel::DataModel, markdown::parser::parse_markdown};
use minijinja::{context, Environment};
use serde::{Deserialize, Serialize};
use std::{io::Write, path::PathBuf, str::FromStr};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, help = "Path or URL to the markdown file")]
    input: InputType,

    #[arg(short, long, help = "Path to the output file")]
    output: Option<PathBuf>,

    #[arg(short, long, help = "Template to use for rendering")]
    template: Templates,

    #[arg(
        short,
        long,
        help = "Root object to start rendering from (required for JSON Schema)"
    )]
    root: Option<String>,
}

#[derive(Debug, ValueEnum, Clone)]
enum Templates {
    PythonDataclass,
    XmlSchema,
    Markdown,
    Shacl,
    JsonSchema,
    Shex,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
enum InputType {
    Remote(String),
    Local(String),
}

impl FromStr for InputType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.starts_with("http") {
            true => Ok(InputType::Remote(s.to_string())),
            false => Ok(InputType::Local(s.to_string())),
        }
    }
}

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
}

fn main() -> Result<(), minijinja::Error> {
    // Parse the command line arguments
    let args = Args::parse();

    // Parse the markdown model
    let path = resolve_input_path(&args.input);
    let (mut model, object_names) = parse_markdown_model(&path);

    // Render the template
    let rendered = match args.template {
        Templates::JsonSchema => {
            model.json_schema(args.root.expect("Root object name is required"))
        }
        _ => render_jinja_template(&args, &mut model, object_names)?,
    };

    match args.output {
        Some(ref output) => {
            std::fs::write(output, rendered).expect("Failed to write output");
        }
        None => {
            println!("{}", rendered);
        }
    }

    Ok(())
}

fn resolve_input_path(input: &InputType) -> PathBuf {
    match input {
        InputType::Remote(url) => {
            let mut path = std::env::temp_dir();
            path.push("markdown.md");
            let mut file = std::fs::File::create(&path).expect("Failed to create file");
            let content = reqwest::blocking::get(url)
                .expect("Failed to fetch URL")
                .text()
                .expect("Failed to read response");
            file.write_all(content.as_bytes())
                .expect("Failed to write to file");
            path
        }
        InputType::Local(path) => PathBuf::from(path),
    }
}

fn parse_markdown_model(path: &PathBuf) -> (DataModel, Vec<String>) {
    let model = parse_markdown(path).expect("Failed to parse markdown");
    let object_names = model
        .objects
        .iter()
        .map(|o| o.name.clone())
        .collect::<Vec<String>>();

    (model, object_names)
}

fn render_jinja_template(
    args: &Args,
    model: &mut DataModel,
    object_names: Vec<String>,
) -> Result<String, minijinja::Error> {
    // Load the template
    let mut env = Environment::new();
    minijinja_embed::load_templates!(&mut env);
    let template = match args.template {
        Templates::PythonDataclass => env.get_template("python-dataclass.jinja").unwrap(),
        Templates::XmlSchema => env.get_template("xml-schema.jinja").unwrap(),
        Templates::Markdown => env.get_template("markdown.jinja").unwrap(),
        Templates::Shacl => env.get_template("shacl.jinja").unwrap(),
        Templates::JsonSchema => env.get_template("json-schema.jinja").unwrap(),
        Templates::Shex => env.get_template("shex.jinja").unwrap(),
    };

    // Type conversions and filtering
    match args.template {
        Templates::Shacl => {
            convert_model_types(model, &SHACL_TYPE_MAPS);
            filter_objects_wo_terms(model);
        }
        Templates::Shex => {
            convert_model_types(model, &SHACL_TYPE_MAPS);
            filter_objects_wo_terms(model);
        }
        Templates::PythonDataclass => convert_model_types(model, &PYTHON_TYPE_MAPS),
        _ => {}
    }

    // Render the template
    let prefixes = get_prefixes(model);
    template.render(context! {
        objects => model.objects,
        object_names => object_names,
        title => model.name,
        prefixes => prefixes,
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
