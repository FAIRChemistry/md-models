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

use clap::{Parser, Subcommand};
use colored::Colorize;
use log::error;
use mdmodels_core::{
    datamodel::DataModel,
    error::DataModelError,
    exporters::{render_jinja_template, Templates},
    json::validation::validate_json,
    linkml::export::serialize_linkml,
    llm::extraction::query_openai,
    pipeline::process_pipeline,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    error::Error,
    fmt::Display,
    fs,
    io::Write,
    path::{Path, PathBuf},
    str::FromStr,
};

/// Command-line interface for MD-Models CLI.
#[derive(Parser)]
#[command(name = "MD-Models CLI", version)]
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
    /// Large Language Model Extraction
    Extract(ExtractArgs),
    /// Validate a dataset against a markdown model.
    Dataset(DatasetArgs),
}

/// Arguments for the validate subcommand.
#[derive(Parser, Debug)]
struct ValidateArgs {
    /// Path or URL to the markdown file.
    #[arg(short, long, help = "Path or URL to the markdown file")]
    input: InputType,

    /// GitHub repository in pip-style shorthand: owner/repo[@ref]
    ///
    /// The ref can be a branch, tag (including release tags), or commit SHA.
    /// Example: acme/example-models@v1.0.0
    #[arg(long = "git", help = "GitHub repository in the form owner/repo[@ref]")]
    git: Option<String>,
}

/// Arguments for the convert subcommand.
#[derive(Parser, Debug)]
struct ConvertArgs {
    /// Path or URL to the markdown file.
    #[arg(short, long, help = "Path or URL to the markdown file")]
    input: InputType,

    /// GitHub repository in pip-style shorthand: owner/repo[@ref]
    ///
    /// The ref can be a branch, tag (including release tags), or commit SHA.
    /// Example: acme/example-models@main
    #[arg(long = "git", help = "GitHub repository in the form owner/repo[@ref]")]
    git: Option<String>,

    /// Path to the output file.
    ///
    /// For the `json-schema-all` target this is a *directory*: one schema file
    /// is written per object.
    #[arg(
        short,
        long,
        help = "Path to the output file (a directory for the 'json-schema-all' target)"
    )]
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

    /// Options to pass to the template.
    #[arg(
        short = 'O',
        long,
        value_parser,
        num_args = 1.., value_delimiter = ',',
        help = "Options to pass to the template"
    )]
    options: Vec<String>,
}

/// Arguments for the pipeline subcommand.
#[derive(Parser, Debug)]
struct PipelineArgs {
    /// Path to the pipeline configuration file.
    #[arg(short, long, help = "Path to the pipeline configuration YAML file")]
    input: PathBuf,
}

/// Arguments for the extract subcommand.
#[derive(Parser, Debug)]
struct ExtractArgs {
    /// Path or URL to the markdown model.
    #[arg(short, long, help = "Path or URL to the markdown model")]
    model: InputType,

    /// GitHub repository in pip-style shorthand: owner/repo[@ref]
    ///
    /// The ref can be a branch, tag (including release tags), or commit SHA.
    /// Example: acme/example-models@a1b2c3d
    #[arg(long = "git", help = "GitHub repository in the form owner/repo[@ref]")]
    git: Option<String>,

    /// Prompt to use for extraction.
    #[arg(short, long, help = "Path to the file to parse")]
    input: PathBuf,

    /// Pre-prompt to use for extraction.
    #[arg(
        short,
        long,
        default_value = "You are a helpful assistant that extracts data from text input.",
        help = "Pre-prompt to use for extraction"
    )]
    pre_prompt: String,

    /// OpenAI model to use for extraction.
    #[arg(
        short,
        long,
        default_value = "gpt-4o",
        help = "OpenAI model to use for extraction. Defaults to 'gpt-4o'."
    )]
    llm_model: String,

    /// Root object to parse into. Defaults to the first entity in the model.
    #[arg(
        short,
        long,
        help = "Root object to parse into. Defaults to the first entity in the model."
    )]
    root: Option<String>,

    /// Output file to write the extracted data to.
    #[arg(short, long, help = "Output file to write the extracted data to")]
    output: Option<PathBuf>,

    /// Whether to extract multiple objects.
    #[arg(long, help = "Whether to extract multiple objects")]
    multiple: bool,
}

/// Arguments for the dataset subcommand.
#[derive(Parser, Debug)]
struct DatasetArgs {
    #[command(subcommand)]
    command: DatasetCommands,
}

/// Subcommands for dataset operations
#[derive(Subcommand, Debug)]
enum DatasetCommands {
    /// Validate a dataset against a markdown model.
    Validate(ValidateDatasetArgs),
    // Add more dataset subcommands here as needed
}

/// Arguments for the validate dataset subcommand.
#[derive(Parser, Debug)]
struct ValidateDatasetArgs {
    /// Path to the dataset file.
    #[arg(short, long, help = "Path to the dataset file")]
    input: InputType,

    /// Path to the markdown model.
    #[arg(short, long, help = "Path to the markdown model")]
    model: InputType,

    /// GitHub repository in pip-style shorthand: owner/repo[@ref]
    ///
    /// The ref can be a branch, tag (including release tags), or commit SHA.
    #[arg(long = "git", help = "GitHub repository in the form owner/repo[@ref]")]
    git: Option<String>,
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
            InputType::Remote(url) => write!(f, "{url}"),
            InputType::Local(path) => write!(f, "{path}"),
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
        Commands::Extract(args) => query_llm(args),
        Commands::Dataset(args) => match args.command {
            DatasetCommands::Validate(args) => validate_ds(args),
        },
    }
}

/// Validates the markdown model specified in the arguments.
///
/// # Arguments
///
/// * `args` - Arguments for the 'validate' subcommand.
fn validate(args: ValidateArgs) -> Result<(), Box<dyn Error>> {
    println!("\n Validating model {} ...", args.input.to_string().bold());

    if args.git.is_some() {
        let model = load_markdown_model(&args.input, args.git.as_deref());
        return match model {
            Ok(_) => {
                print_validation_result(true);
                Ok(())
            }
            Err(err) => {
                print_validation_result(false);
                Err(err)
            }
        };
    }

    let path = resolve_input_path(&args.input);

    if is_json_schema(&path)? {
        return validate_from_json_schema(&path);
    }

    let model = DataModel::from_markdown(&path);

    match model {
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

/// Validates a JSON schema file.
///
/// # Arguments
///
/// * `path` - Path to the JSON schema file.
fn validate_from_json_schema(path: &Path) -> Result<(), Box<dyn Error>> {
    if let Err(err) = DataModel::from_json_schema(path) {
        match err {
            DataModelError::ValidationError(validator) => {
                validator.log_result();
                Err("Model is invalid".into())
            }
            _ => Err(err.into()),
        }
    } else {
        print_validation_result(true);
        Ok(())
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

    println!(" └── {message}\n");
}

fn query_llm(args: ExtractArgs) -> Result<(), Box<dyn Error>> {
    let model = load_markdown_model(&args.model, args.git.as_deref())?;
    let prompt = std::fs::read_to_string(&args.input)?;
    let pre_prompt = args.pre_prompt;
    let llm_model = args.llm_model;
    let root = match args.root {
        Some(root) => root,
        None => model
            .objects
            .first()
            .ok_or("No objects found in model".to_string())?
            .name
            .clone(),
    };

    let response = tokio::runtime::Runtime::new()?.block_on(query_openai(
        &prompt,
        &pre_prompt,
        &model,
        &root,
        &llm_model,
        args.multiple,
        None,
    ))?;

    match args.output {
        Some(ref output) => {
            let json_string = serde_json::to_string_pretty(&response)?;
            std::fs::write(output, json_string).expect("Failed to write output");
        }
        None => {
            let json_string = serde_json::to_string_pretty(&response)?;
            println!("{json_string}");
        }
    }

    Ok(())
}

/// Converts the markdown model specified in the arguments to another format.
///
/// # Arguments
///
/// * `args` - Arguments for the convert subcommand.
fn convert(args: ConvertArgs) -> Result<(), Box<dyn Error>> {
    // Parse the model.
    let mut model = if let Some(repo) = args.git.as_deref() {
        load_markdown_model(&args.input, Some(repo))?
    } else {
        let path = resolve_input_path(&args.input);
        if is_json_schema(&path)? {
            DataModel::from_json_schema(&path)?
        } else {
            DataModel::from_markdown(&path)?
        }
    };

    // Special case JSON Schema all
    if let Templates::JsonSchemaAll = args.template {
        render_all_json_schemes(&model, &args.output)?;
        return Ok(()); // Early return
    }

    // Render the template.
    let config: HashMap<String, String> = args
        .options
        .iter()
        .map(|s| (s.clone(), "true".to_string()))
        .collect();
    let rendered = match args.template {
        Templates::JsonSchema => {
            model.json_schema(args.root, args.options.contains(&"openai".to_string()))?
        }
        Templates::Linkml => serialize_linkml(model, args.output.as_ref())?,
        Templates::Internal => render_internal_schema(&model)?,
        Templates::JsonLd => {
            let root = args.root;
            serde_json::to_string_pretty(&model.json_ld_header(root.as_deref())?).unwrap()
        }
        _ => render_jinja_template(&args.template, &mut model, Some(&config))?,
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

/// Checks if the input is a JSON Schema.
///
/// # Arguments
///
/// * `path` - The path to the input file.
///
/// # Returns
///
/// True if the input is a JSON Schema, false otherwise.
fn is_json_schema(path: &PathBuf) -> Result<bool, Box<dyn Error>> {
    let content = std::fs::read_to_string(path)?;
    let parsed = serde_json::from_str::<serde_json::Value>(&content);

    match parsed {
        Ok(value) => Ok(value.is_object()),
        Err(_) => Ok(false),
    }
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

/// Loads a markdown model either from local/remote input or from a GitHub repository cache.
fn load_markdown_model(input: &InputType, git: Option<&str>) -> Result<DataModel, Box<dyn Error>> {
    if let Some(repo) = git {
        let path = match input {
            InputType::Local(path) => path.as_str(),
            InputType::Remote(_) => {
                return Err(
                    "When using --git, provide --input as a repository-root path, not a URL".into(),
                )
            }
        };

        return DataModel::from_github(repo, path);
    }

    let path = resolve_input_path(input);
    let model = DataModel::from_markdown(&path)?;
    Ok(model)
}

/// Renders all JSON Schemas for the model.
fn render_all_json_schemes(
    model: &DataModel,
    outdir: &Option<PathBuf>,
) -> Result<(), Box<dyn Error>> {
    // This target writes one schema per object, so `--output` names a
    // directory rather than a file.
    let outdir = match outdir {
        Some(outdir) => outdir,
        None => {
            return Err(
                "The 'json-schema-all' target writes one schema per object and \
                        therefore requires an output directory. Pass -o/--output <DIRECTORY>."
                    .into(),
            )
        }
    };

    if outdir.exists() && !outdir.is_dir() {
        return Err(format!(
            "The 'json-schema-all' target writes one schema per object, so -o/--output must \
             be a directory, but '{}' is an existing file.",
            outdir.display()
        )
        .into());
    }

    // If the output directory does not exist, create it
    fs::create_dir_all(outdir)?;

    // Render the JSON Schema for each entity
    model.json_schema_all(outdir.to_path_buf(), false)?;

    Ok(())
}

/// Renders the internal schema for the model.
///
/// # Arguments
///
/// * `model` - The DataModel to render.
///
/// # Returns
///
/// A Result containing the rendered internal schema as a String or an error if rendering fails.
fn render_internal_schema(model: &DataModel) -> Result<String, Box<dyn Error>> {
    serde_json::to_string_pretty(&model).map_err(|e| e.into())
}

/// Validates a dataset against a markdown model.
fn validate_ds(args: ValidateDatasetArgs) -> Result<(), Box<dyn Error>> {
    let model = load_markdown_model(&args.model, args.git.as_deref())?;
    let dataset_path = resolve_input_path(&args.input);
    let result = validate_json(dataset_path, &model, None)?;

    for error in result {
        error!("{}", error);
    }

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

    //
    // Recursive models
    //
    // A stack overflow cannot be caught in-process, so recursive models are
    // driven as subprocesses where an abort is observable as an exit status.
    //

    #[test]
    fn test_json_ld_recursive_model_does_not_abort() {
        let mut cmd = Command::cargo_bin("md-models").unwrap();
        let assert = cmd
            .arg("convert")
            .arg("-i")
            .arg("tests/data/model_recursive.md")
            .arg("-t")
            .arg("json-ld")
            .arg("-r")
            .arg("Experiment")
            .assert();
        assert.success();
    }

    /// The cycle here runs Experiment -> Condition -> Experiment.
    #[test]
    fn test_json_ld_mutual_recursion_does_not_abort() {
        let mut cmd = Command::cargo_bin("md-models").unwrap();
        let assert = cmd
            .arg("convert")
            .arg("-i")
            .arg("tests/data/model_recursive.md")
            .arg("-t")
            .arg("json-ld")
            .arg("-r")
            .arg("Condition")
            .assert();
        assert.success();
    }

    /// Every target must survive a recursive model, not just JSON-LD.
    #[test]
    fn test_all_targets_handle_recursive_model() {
        for template in [
            "json-schema",
            "json-ld",
            "shacl",
            "owl",
            "shex",
            "xml-schema",
            "linkml",
            "protobuf",
            "graphql",
            "rust",
            "python-pydantic",
            "typescript",
        ] {
            let mut cmd = Command::cargo_bin("md-models").unwrap();
            let assert = cmd
                .arg("convert")
                .arg("-i")
                .arg("tests/data/model_recursive.md")
                .arg("-t")
                .arg(template)
                .arg("-r")
                .arg("Experiment")
                .assert();
            assert.success();
        }
    }

    //
    // json-schema-all writes one file per object, so --output names a
    // directory.
    //

    #[test]
    fn test_json_schema_all_missing_output_is_clean_error() {
        let mut cmd = Command::cargo_bin("md-models").unwrap();
        let assert = cmd
            .arg("convert")
            .arg("-i")
            .arg("tests/data/model.md")
            .arg("-t")
            .arg("json-schema-all")
            .assert()
            .failure();

        let stderr = String::from_utf8_lossy(&assert.get_output().stderr).to_string();
        assert!(
            !stderr.contains("panicked at"),
            "a missing output directory must be a diagnostic, not a panic: {stderr}"
        );
        assert!(
            stderr.contains("directory"),
            "the error must say an output directory is required: {stderr}"
        );
    }

    #[test]
    fn test_json_schema_all_output_is_existing_file_is_clean_error() {
        let dir = std::path::Path::new("tests/intermediates/json_schema_all");
        fs::create_dir_all(dir).unwrap();
        let file = dir.join("not_a_directory.txt");
        fs::write(&file, "placeholder").unwrap();

        let mut cmd = Command::cargo_bin("md-models").unwrap();
        let assert = cmd
            .arg("convert")
            .arg("-i")
            .arg("tests/data/model.md")
            .arg("-t")
            .arg("json-schema-all")
            .arg("-o")
            .arg(file.to_str().unwrap())
            .assert()
            .failure();

        let stderr = String::from_utf8_lossy(&assert.get_output().stderr).to_string();
        assert!(
            !stderr.contains("panicked at"),
            "an existing file as output must be a diagnostic, not a panic: {stderr}"
        );
        assert!(stderr.contains("directory"), "{stderr}");
    }

    /// Each generated schema must be non-empty and parse as JSON.
    #[test]
    fn test_json_schema_all_writes_valid_non_empty_schemas() {
        let dir = std::path::Path::new("tests/intermediates/json_schema_all/generated");
        fs::remove_dir_all(dir).ok();

        let mut cmd = Command::cargo_bin("md-models").unwrap();
        cmd.arg("convert")
            .arg("-i")
            .arg("tests/data/model.md")
            .arg("-t")
            .arg("json-schema-all")
            .arg("-o")
            .arg(dir.to_str().unwrap())
            .assert()
            .success();

        assert!(dir.is_dir(), "the output directory must be created");

        for name in ["Test.json", "Test2.json"] {
            let path = dir.join(name);
            assert!(path.exists(), "{name} was not written");

            let content = fs::read_to_string(&path).unwrap();
            assert!(!content.trim().is_empty(), "{name} is empty");

            let parsed: serde_json::Value = serde_json::from_str(&content)
                .unwrap_or_else(|e| panic!("{name} is not JSON: {e}"));
            assert!(
                parsed.get("properties").is_some(),
                "{name} has no properties"
            );
        }
    }
}
