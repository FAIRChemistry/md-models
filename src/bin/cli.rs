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

use clap::{Parser, Subcommand};
use colored::Colorize;
use mdmodels::{
    datamodel::DataModel,
    exporters::{render_jinja_template, Templates},
    pipeline::process_pipeline,
};
use serde::{Deserialize, Serialize};
use std::{error::Error, fmt::Display, fs, io::Write, path::PathBuf, str::FromStr};

/// Command-line interface for MD-Models CLI.
#[derive(Parser)]
#[command(name = "MD-Models CLI", version = "0.1.0")]
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
    /// Pipeline for generating multiple files.
    Pipeline(PipelineArgs),
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

/// Arguments for the pipeline subcommand.
#[derive(Parser, Debug)]
struct PipelineArgs {
    /// Path to the pipeline configuration file.
    #[arg(short, long, help = "Path to the pipeline configuration YAML file")]
    input: PathBuf,
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

impl Display for InputType {
    /// Display the input type.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InputType::Remote(url) => write!(f, "{}", url),
            InputType::Local(path) => write!(f, "{}", path),
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
        Commands::Pipeline(args) => process_pipeline(&args.input),
    }
}

/// Validates the markdown model specified in the arguments.
///
/// # Arguments
///
/// * `args` - Arguments for the 'validate' subcommand.
fn validate(args: ValidateArgs) -> Result<(), Box<dyn Error>> {
    println!("\n Validating model {} ...", args.input.to_string().bold());

    let path = resolve_input_path(&args.input);

    match DataModel::from_markdown(&path) {
        Ok(_) => {
            print_validation_result(true);
            Ok(())
        }
        Err(result) => {
            result.log_result();
            print_validation_result(false);
            Err("Model is invalid".into())
        }
    }
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
    let mut model = DataModel::from_markdown(&path)?;

    // Special case JSON Schema all
    if let Templates::JsonSchemaAll = args.template {
        render_all_json_schemes(&model, &args.output)?;
        return Ok(()); // Early return
    }

    // Render the template.
    let rendered = match args.template {
        Templates::JsonSchema => model.json_schema(args.root),
        _ => render_jinja_template(&args.template, &mut model, None)?,
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

/// Renders all JSON Schemas for the model.
fn render_all_json_schemes(
    model: &DataModel,
    outdir: &Option<PathBuf>,
) -> Result<(), Box<dyn Error>> {
    if outdir.is_none() {
        panic!("Output directory is required for JSON Schema all");
    }

    let outdir = outdir.as_ref().unwrap();

    // Check if the output is a directory
    if !outdir.is_dir() && outdir.exists() {
        panic!("Output must be a directory");
    }

    // If the output directory does not exist, create it
    fs::create_dir_all(outdir)?;

    // Render the JSON Schema for each entity
    model.json_schema_all(outdir.to_str().unwrap().to_string());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_cmd::Command;
    use pretty_assertions::assert_eq;

    /// Test for resolving local input paths.
    #[test]
    fn test_resolve_input_path() {
        let path = resolve_input_path(&InputType::Local("tests/data/markdown.md".to_string()));
        assert_eq!(path.to_str().unwrap(), "tests/data/markdown.md");
    }

    /// Test Display for InputType
    #[test]
    fn test_display_input_type() {
        let remote = InputType::Remote("https://example.com".to_string());
        let local = InputType::Local("tests/data/markdown.md".to_string());
        assert_eq!(remote.to_string(), "https://example.com");
        assert_eq!(local.to_string(), "tests/data/markdown.md");
    }

    #[test]
    fn test_successful_validation_result() {
        let mut cmd = Command::cargo_bin("md-models").unwrap();
        let assert = cmd
            .arg("validate")
            .arg("-i")
            .arg("tests/data/model.md")
            .assert();
        assert.success();
    }

    #[test]
    fn test_failed_validation_result() {
        let mut cmd = Command::cargo_bin("md-models").unwrap();
        let assert = cmd
            .arg("validate")
            .arg("-i")
            .arg("tests/data/model_missing_types.md")
            .assert();
        assert.failure();
    }

    #[test]
    fn test_successful_conversion() {
        let mut cmd = Command::cargo_bin("md-models").unwrap();
        let assert = cmd
            .arg("convert")
            .arg("-i")
            .arg("tests/data/model.md")
            .arg("-t")
            .arg("markdown")
            .assert();
        assert.success();
    }

    #[test]
    fn test_json_schema_no_root() {
        let mut cmd = Command::cargo_bin("md-models").unwrap();
        let assert = cmd
            .arg("convert")
            .arg("-i")
            .arg("tests/data/model.md")
            .arg("-t")
            .arg("json-schema")
            .assert();
        assert.success();
    }

    #[test]
    fn test_pipeline_single_model() {
        let mut cmd = Command::cargo_bin("md-models").unwrap();
        let assert = cmd
            .arg("pipeline")
            .arg("-i")
            .arg("tests/test_pipeline.toml")
            .assert();
        assert.success();
    }

    #[test]
    fn test_pipeline_multiple_models() {
        let mut cmd = Command::cargo_bin("md-models").unwrap();
        let assert = cmd
            .arg("pipeline")
            .arg("-i")
            .arg("tests/test_pipeline_per_spec.toml")
            .assert();
        assert.success();
    }

    #[test]
    fn test_pipeline_multiple_models_invalid() {
        let mut cmd = Command::cargo_bin("md-models").unwrap();
        let assert = cmd
            .arg("pipeline")
            .arg("-i")
            .arg("tests/test_pipeline_per_spec_invalid.toml")
            .assert();
        assert.failure();
    }
}
