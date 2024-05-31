use clap::{Parser, Subcommand};
use colored::Colorize;
use mdmodels::{
    exporters::{render_jinja_template, Templates},
    markdown::parser::parse_markdown,
};
use serde::{Deserialize, Serialize};
use std::{error::Error, io::Write, path::PathBuf, str::FromStr};

/// Command-line interface for MD-Models CLI.
#[derive(Parser)]
#[command(name = "MD-Models CLI", version = "1.0")]
#[command(about = "Validate and convert Markdown Data Models", long_about = None)]
struct Cli {
    /// Subcommands for the CLI.
    #[command(subcommand)]
    cmd: Commands,
}

/// Enum representing the subcommands.
#[derive(Subcommand)]
enum Commands {
    /// Convert a markdown model to another format.
    Convert(ConvertArgs),
    /// Validate a markdown model.
    Validate(ValidateArgs),
}

/// Arguments for the validate subcommand.
#[derive(Parser, Debug)]
struct ValidateArgs {
    /// Path or URL to the markdown file.
    #[arg(short, long, help = "Path or URL to the markdown file")]
    input: InputType,
}

/// Arguments for the convert subcommand.
#[derive(Parser, Debug)]
struct ConvertArgs {
    /// Path or URL to the markdown file.
    #[arg(short, long, help = "Path or URL to the markdown file")]
    input: InputType,

    /// Path to the output file.
    #[arg(short, long, help = "Path to the output file")]
    output: Option<PathBuf>,

    /// Template to use for rendering.
    #[arg(short, long, help = "Template to use for rendering")]
    template: Templates,

    /// Root object to start rendering from (required for JSON Schema).
    #[arg(
        short,
        long,
        help = "Root object to start rendering from (required for JSON Schema)"
    )]
    root: Option<String>,
}

/// Represents the input type, either remote URL or local file path.
#[derive(Deserialize, Serialize, Clone, Debug)]
enum InputType {
    /// Input from a remote URL.
    Remote(String),
    /// Input from a local file path.
    Local(String),
}

impl FromStr for InputType {
    type Err = String;

    /// Converts a string to an InputType (Remote or Local).
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("http") {
            Ok(InputType::Remote(s.to_string()))
        } else {
            Ok(InputType::Local(s.to_string()))
        }
    }
}

impl ToString for InputType {
    /// Converts an InputType to a string.
    fn to_string(&self) -> String {
        match self {
            InputType::Remote(url) => url.to_string(),
            InputType::Local(path) => path.to_string(),
        }
    }
}

/// Main entry point of the application.
fn main() -> Result<(), Box<dyn Error>> {
    // Initialize the logger.
    pretty_env_logger::init();

    // Parse the command line arguments.
    let args = Cli::parse();

    match args.cmd {
        Commands::Validate(args) => validate(args),
        Commands::Convert(args) => convert(args),
    }
}

/// Validates the markdown model specified in the arguments.
///
/// # Arguments
///
/// * `args` - Arguments for the validate subcommand.
fn validate(args: ValidateArgs) -> Result<(), Box<dyn Error>> {
    println!("\n Validating model {} ...", args.input.to_string().bold());

    let path = resolve_input_path(&args.input);
    let model = parse_markdown(&path);

    match model {
        Ok(_) => print_validation_result(true),
        Err(_) => print_validation_result(false),
    }

    Ok(())
}

/// Prints the result of the validation.
///
/// # Arguments
/// * `result` - The result of the validation.
fn print_validation_result(result: bool) {
    let message = if result {
        "Model is valid".green().bold().to_string()
    } else {
        "Model is invalid".red().bold().to_string()
    };

    println!(" └── {}\n", message);
}

/// Converts the markdown model specified in the arguments to another format.
///
/// # Arguments
///
/// * `args` - Arguments for the convert subcommand.
fn convert(args: ConvertArgs) -> Result<(), Box<dyn Error>> {
    // Parse the markdown model.
    let path = resolve_input_path(&args.input);
    let mut model = parse_markdown(&path)?;
    model.sort_attrs();

    // Render the template.
    let rendered = match args.template {
        Templates::JsonSchema => model.json_schema(args.root.expect(
            "Root object name is required. Please add --root <object_name> or -r <object_name>",
        )),
        _ => render_jinja_template(&args.template, &mut model)?,
    };

    // Output the rendered content.
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

/// Resolves the input path based on the InputType.
///
/// If the input is a remote URL, it fetches the content and saves it to a temporary file.
/// If the input is a local path, it returns the corresponding PathBuf.
///
/// # Arguments
///
/// * `input` - The input type (Remote or Local).
///
/// # Returns
///
/// PathBuf representing the local path to the input file.
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

    /// Test for resolving local input paths.
    #[test]
    fn test_resolve_input_path() {
        let path = resolve_input_path(&InputType::Local("tests/data/markdown.md".to_string()));
        assert_eq!(path.to_str().unwrap(), "tests/data/markdown.md");
    }
}
