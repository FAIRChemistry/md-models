use clap::{Parser, ValueEnum};
use lazy_static::lazy_static;
use mdmodels::{datamodel::DataModel, markdown::parser::parse_markdown};
use minijinja::{context, Environment};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    input: PathBuf,

    #[arg(short, long)]
    output: Option<PathBuf>,

    #[arg(short, long)]
    template: Templates,

    #[arg(short, long)]
    root: Option<String>,
}

#[derive(Debug, ValueEnum, Clone)]
enum Templates {
    PythonDataclass,
    XmlSchema,
    Markdown,
    Shacl,
    JsonSchema,
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
    let (mut model, object_names) = parse_markdown_model(&args.input);

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
    };

    // Type conversions
    match args.template {
        Templates::Shacl => convert_model_types(model, &SHACL_TYPE_MAPS),
        Templates::PythonDataclass => convert_model_types(model, &PYTHON_TYPE_MAPS),
        _ => {}
    }

    // Render the template
    template.render(context! {
        objects => model.objects,
        object_names => object_names,
        title => model.name,
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
