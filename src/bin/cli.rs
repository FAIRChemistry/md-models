use clap::Parser;
use mdmodels::{
    exporters::{render_jinja_template, Templates},
    markdown::parser::parse_markdown,
};
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

fn main() -> Result<(), minijinja::Error> {
    // Parse the command line arguments
    let args = Args::parse();

    // Parse the markdown model
    let path = resolve_input_path(&args.input);
    let mut model = parse_markdown(&path).expect("Failed to parse markdown");
    model.sort_attrs();

    // Render the template
    let rendered = match args.template {
        Templates::JsonSchema => model.json_schema(args.root.expect(
            "Root object name is required. Please add --root <object_name> or -r <object_name>",
        )),
        _ => render_jinja_template(&args.template, &mut model)?,
    };

    match args.output {
        Some(ref output) => {
            std::fs::write(output, rendered.trim()).expect("Failed to write output");
        }
        None => {
            println!("{}", rendered.trim());
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

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_resolve_input_path() {
        let path = resolve_input_path(&InputType::Local("tests/data/markdown.md".to_string()));
        assert_eq!(path.to_str().unwrap(), "tests/data/markdown.md");
    }
}
