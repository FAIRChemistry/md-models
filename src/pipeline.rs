use crate::{datamodel::DataModel, exporters::Templates};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, error::Error, fs, path::PathBuf, str::FromStr};

/// Represents a template with metadata and generation specifications.
#[derive(Debug, Serialize, Deserialize)]
struct GenTemplate {
    meta: Meta,
    generate: HashMap<String, GenSpecs>,
}

/// Represents metadata for the template.
#[derive(Debug, Serialize, Deserialize)]
struct Meta {
    name: Option<String>,
    description: Option<String>,
    paths: Vec<PathBuf>,
}

/// Represents generation specifications for a template.
#[derive(Debug, Serialize, Deserialize)]
struct GenSpecs {
    description: Option<String>,
    out: PathBuf,
    root: Option<String>,
}

/// Processes the pipeline by reading the template file, building the data model, and generating files based on the specifications.
///
/// # Arguments
///
/// * `path` - Path to the template file.
///
/// # Returns
///
/// A Result indicating success or failure.
pub fn process_pipeline(path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(path).unwrap();
    let gen_template: GenTemplate = toml::from_str(content.as_str()).unwrap();
    let model = build_models(gen_template.meta.paths.as_slice())?;

    for (name, specs) in gen_template.generate.into_iter() {
        let template = Templates::from_str(name.as_str())?;

        match template {
            Templates::JsonSchema => {
                serialize_to_json_schema(model.clone(), specs.root, &specs.out)?;
            }
            Templates::JsonSchemaAll => {
                serialize_all_json_schemes(model.clone(), &specs.out)?;
            }
            Templates::Shex => {
                let content = model.clone().convert_to(&template)?;
                save_to_file(&specs.out, content.as_str())?;
            }
            Templates::Shacl => {
                let content = model.clone().convert_to(&template)?;
                save_to_file(&specs.out, content.as_str())?;
            }
            Templates::Markdown => {
                let content = model.clone().convert_to(&template)?;
                save_to_file(&specs.out, content.as_str())?;
            }
            Templates::CompactMarkdown => {
                let content = model.clone().convert_to(&template)?;
                save_to_file(&specs.out, content.as_str())?;
            }
            Templates::PythonDataclass => {
                let content = model.clone().convert_to(&template)?;
                save_to_file(&specs.out, content.as_str())?;
            }
            Templates::PythonSdrdm => {
                let content = model.clone().convert_to(&template)?;
                save_to_file(&specs.out, content.as_str())?;
            }
            Templates::XmlSchema => {
                let content = model.clone().convert_to(&template)?;
                save_to_file(&specs.out, content.as_str())?;
            }
            Templates::MkDocs => {
                let content = model.clone().convert_to(&template)?;
                save_to_file(&specs.out, content.as_str())?;
            }
        }

        println!(
            "  Generated {} - {}",
            name.bold().green(),
            specs.out.to_str().unwrap().bold()
        )
    }

    Ok(())
}

/// Builds the data model by reading and merging multiple paths.
///
/// # Arguments
///
/// * `paths` - A slice of PathBuf representing the paths to read.
///
/// # Returns
///
/// A Result containing the DataModel or an error.
fn build_models(paths: &[PathBuf]) -> Result<DataModel, Box<dyn Error>> {
    let first_path = paths.first().unwrap();
    path_exists(first_path)?;

    let mut model = DataModel::from_markdown(first_path)?;

    if paths.len() == 1 {
        return Ok(model);
    }

    for path in paths.iter().skip(1) {
        path_exists(path)?;
        let new_model = DataModel::from_markdown(path)?;
        model.merge(&new_model);
    }

    Ok(model)
}

/// Checks if the given path exists.
///
/// # Arguments
///
/// * `path` - A reference to a PathBuf to check.
///
/// # Returns
///
/// A Result indicating success or failure.
fn path_exists(path: &PathBuf) -> Result<(), Box<dyn Error>> {
    if !path.exists() {
        return Err(format!("Path does not exist: {:?}", path).into());
    }
    Ok(())
}

/// Serializes the data model to a JSON schema file.
///
/// # Arguments
///
/// * `model` - The DataModel to serialize.
/// * `root` - The root object for the JSON schema.
/// * `out` - The output path for the JSON schema file.
///
/// # Returns
///
/// A Result indicating success or failure.
fn serialize_to_json_schema(
    model: DataModel,
    root: Option<String>,
    out: &PathBuf,
) -> Result<(), Box<dyn Error>> {
    match root {
        Some(root) => {
            let schema = model.json_schema(root);
            save_to_file(out, &schema)?;
            Ok(())
        }
        None => Err("Root object has to be specified".into()),
    }
}

/// Serializes all JSON schemas for the data model to the specified output directory.
///
/// # Arguments
///
/// * `model` - The DataModel to serialize.
/// * `out` - The output directory for the JSON schema files.
///
/// # Returns
///
/// A Result indicating success or failure.
fn serialize_all_json_schemes(model: DataModel, out: &PathBuf) -> Result<(), Box<dyn Error>> {
    if !out.exists() {
        fs::create_dir_all(out)?;
    }
    model.json_schema_all(out.to_str().unwrap().to_string());

    Ok(())
}

/// Saves the given content to the specified file.
///
/// # Arguments
///
/// * `out` - The output path for the file.
/// * `content` - The content to write to the file.
///
/// # Returns
///
/// A Result indicating success or failure.
fn save_to_file(out: &PathBuf, content: &str) -> Result<(), Box<dyn Error>> {
    let dir = out.parent().unwrap();
    if !dir.exists() {
        fs::create_dir_all(dir)?;
    }

    fs::write(out, content.trim())?;
    Ok(())
}
