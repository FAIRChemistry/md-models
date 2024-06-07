use crate::{datamodel::DataModel, exporters::Templates};
use colored::Colorize;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    error::Error,
    fs,
    path::{Path, PathBuf},
    str::FromStr,
};

/// Represents a template with metadata and generation specifications.
#[derive(Debug, Serialize, Deserialize)]
struct GenTemplate {
    meta: Meta,
    generate: HashMap<String, GenSpecs>,
}

impl GenTemplate {
    pub fn prepend_root(&mut self, path: &Path) {
        for (_, specs) in self.generate.iter_mut() {
            specs.prepend_root(path);
        }
    }
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
    #[serde(rename = "per-spec")]
    per_spec: Option<bool>,
}

impl GenSpecs {
    pub fn prepend_root(&mut self, path: &Path) {
        if path.is_file() {
            panic!("Root to prepend is not a directory.");
        }

        self.out = path.join(&self.out);
    }
}

/// Sate that determines whether objects are merged or not.
#[derive(Debug)]
enum MergeState {
    Merge,
    NoMerge,
}

impl From<bool> for MergeState {
    fn from(value: bool) -> Self {
        if value {
            MergeState::NoMerge
        } else {
            MergeState::Merge
        }
    }
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
    let content = std::fs::read_to_string(path)?;
    let mut gen_template: GenTemplate = toml::from_str(content.as_str()).unwrap();

    if let Some(parent) = path.parent() {
        gen_template.prepend_root(parent);
    }

    let paths = gen_template.meta.paths.as_slice();

    for (name, specs) in gen_template.generate.into_iter() {
        let template = Templates::from_str(name.as_str())?;
        let merge_state = MergeState::from(specs.per_spec.unwrap_or(false));

        match template {
            Templates::JsonSchema => {
                let model = build_models(paths)?;
                serialize_to_json_schema(model, specs.root, &specs.out, &merge_state)?;
            }
            Templates::JsonSchemaAll => {
                serialize_all_json_schemes(&specs.out, paths, &merge_state)?;
            }
            Templates::Shex => {
                serialize_by_template(&specs.out, paths, &merge_state, &template, None)?;
            }
            Templates::Shacl => {
                serialize_by_template(&specs.out, paths, &merge_state, &template, None)?;
            }
            Templates::Markdown => {
                serialize_by_template(&specs.out, paths, &merge_state, &template, None)?;
            }
            Templates::CompactMarkdown => {
                serialize_by_template(&specs.out, paths, &merge_state, &template, None)?;
            }
            Templates::PythonDataclass => {
                serialize_by_template(&specs.out, paths, &merge_state, &template, None)?;
            }
            Templates::PythonSdrdm => {
                serialize_by_template(&specs.out, paths, &merge_state, &template, None)?;
            }
            Templates::XmlSchema => {
                serialize_by_template(&specs.out, paths, &merge_state, &template, None)?;
            }
            Templates::MkDocs => {
                // If the template is not set to merge, then disable the navigation.
                let mut config = HashMap::new();
                if let MergeState::Merge = merge_state {
                    config.insert("nav".to_string(), "false".to_string());
                }

                serialize_by_template(&specs.out, paths, &merge_state, &template, Some(&config))?;
            }
        }
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
    merge_state: &MergeState,
) -> Result<(), Box<dyn Error>> {
    if let MergeState::NoMerge = merge_state {
        return Err(
            "Per spec is not supported for single JSON schema generation at the moment.".into(),
        );
    }

    match root {
        Some(root) => {
            let schema = model.json_schema(Some(root));
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
fn serialize_all_json_schemes(
    out: &PathBuf,
    specs: &[PathBuf],
    merge_state: &MergeState,
) -> Result<(), Box<dyn Error>> {
    if out.is_file() {
        return Err("Output path is a file".into());
    }
    if !out.exists() {
        fs::create_dir_all(out)?;
    }

    match merge_state {
        MergeState::Merge => {
            let model = build_models(specs)?;
            model.json_schema_all(out.to_str().unwrap().to_string());
            Ok(())
        }
        MergeState::NoMerge => {
            for spec in specs {
                let model = DataModel::from_markdown(spec)?;
                let path = out.join(get_file_name(spec));
                model.json_schema_all(path.to_str().unwrap().to_string());
            }
            Ok(())
        }
    }
}

/// Serializes the data model by the specified template.
///
/// # Arguments
///
/// * `out` - The output path for the serialized data model.
/// * `specs` - A slice of PathBuf representing the paths to read.
/// * `merge_state` - The merge state.
/// * `template` - The template to use for serialization.
///
/// # Returns
///
/// A Result indicating success or failure.
fn serialize_by_template(
    out: &PathBuf,
    specs: &[PathBuf],
    merge_state: &MergeState,
    template: &Templates,
    config: Option<&HashMap<String, String>>,
) -> Result<(), Box<dyn Error>> {
    match merge_state {
        MergeState::Merge => {
            print_render_msg(out, template);

            let mut model = build_models(specs)?;
            let content = model.convert_to(template, None)?;

            return save_to_file(out, content.as_str());
        }
        MergeState::NoMerge => {
            if !has_wildcard_fname(out) {
                return Err("
                    Output file name must contain a wildcard.
                    For example, a valid wildcard is 'path/to/*.json'"
                    .into());
            }

            for spec in specs {
                if !spec.exists() {
                    return Err(format!("Path does not exist: {:?}", spec).into());
                }

                let path = replace_wildcard_fname(out, get_file_name(spec).as_str());
                print_render_msg(&path, template);

                let mut model = DataModel::from_markdown(spec)?;
                let content = model.convert_to(template, config)?;

                save_to_file(&path, content.as_str())?;
            }
        }
    }

    Ok(())
}

/// Checks if the given path has a wildcard file name.
///
/// # Arguments
///
/// * `path` - The path to check.
///
/// # Returns
///
/// A boolean indicating if the path has a wildcard file name.
fn has_wildcard_fname(path: &Path) -> bool {
    let pattern = r"^.+/\*\.[a-zA-Z0-9]+$";
    let re = Regex::new(pattern).unwrap();
    re.is_match(path.to_str().unwrap())
}

/// Replaces the wildcard file name with the given name.
///
/// # Arguments
///
/// * `path` - The path to replace the wildcard file name.
/// * `name` - The name to replace the wildcard file name with.
///
/// # Returns
///
/// A PathBuf with the wildcard file name replaced.
fn replace_wildcard_fname(path: &Path, name: &str) -> PathBuf {
    let path = PathBuf::from(path);
    let file_name = path.file_name().unwrap().to_str().unwrap();
    let new_name = file_name.replace('*', name);
    let parent = path.parent().unwrap();

    parent.join(new_name)
}

/// Gets the file name without the extension.
///
/// # Arguments
///
/// * `path` - The path to get the file name from.
///
/// # Returns
///
/// A string containing the file name without the extension.
fn get_file_name(path: &Path) -> String {
    // Get the filename without the extension
    let file_name = path.file_name().unwrap().to_str().unwrap();
    let file_name = file_name.split('.').collect::<Vec<&str>>()[0];
    file_name.to_string()
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

fn print_render_msg(out: &Path, template: &Templates) {
    println!(
        " [{}] Writing to '{}'",
        template.to_string().green().bold(),
        out.to_str().unwrap().to_string().bold(),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_has_wildcard_fname() {
        let path = PathBuf::from("path/to/*.json");
        let result = has_wildcard_fname(&path);
        assert!(result);
    }

    #[test]
    fn test_has_wildcard_fname_no_wildcard() {
        let path = PathBuf::from("path/to/file.json");
        let result = has_wildcard_fname(&path);
        assert!(!result);
    }
}
