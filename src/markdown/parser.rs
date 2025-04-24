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

use colored::Colorize;
use core::panic;
use lazy_static::lazy_static;
use log::error;
use std::collections::{BTreeMap, HashMap};
use std::error::Error;
use std::path::Path;

use pulldown_cmark::{CowStr, Event, HeadingLevel, OffsetIter, Options, Parser, Tag, TagEnd};
use regex::Regex;

use crate::attribute;
use crate::datamodel::DataModel;
use crate::object::{self, Enumeration, Object};
use crate::option::RawOption;
use crate::validation::Validator;

use super::frontmatter::{parse_frontmatter, FrontMatter, ImportType};
use super::position::{Position, PositionRange};

lazy_static! {
    static ref MD_MODEL_TYPES: BTreeMap<&'static str, &'static str> = {
        let mut m = BTreeMap::new();
        m.insert(
            "Equation",
            include_str!("../../types/equation/equation-internal.json"),
        );
        m.insert(
            "UnitDefinition",
            include_str!("../../types/unit-definition/unit-definition-internal.json"),
        );
        m
    };
}

// Heading levels for re-use
const H1: Tag = Tag::Heading {
    level: HeadingLevel::H1,
    id: None,
    classes: Vec::new(),
    attrs: Vec::new(),
};
const H2: Tag = Tag::Heading {
    level: HeadingLevel::H2,
    id: None,
    classes: Vec::new(),
    attrs: Vec::new(),
};
const H3: Tag = Tag::Heading {
    level: HeadingLevel::H3,
    id: None,
    classes: Vec::new(),
    attrs: Vec::new(),
};

const H3_END: TagEnd = TagEnd::Heading(HeadingLevel::H3);

#[derive(Debug, PartialEq, Eq)]
enum ParserState {
    InDefinition,
    OutsideDefinition,
    InHeading,
}

/// Parses a Markdown file and returns a DataModel.
///
/// # Arguments
/// * `content` - The markdown content to parse
/// * `path` - Optional path to the markdown file
///
/// # Returns
/// * `Result<DataModel, Validator>` - The parsed data model or validation errors
#[allow(clippy::result_large_err)]
pub fn parse_markdown(content: &str, path: Option<&Path>) -> Result<DataModel, Validator> {
    let content = clean_content(content);
    let config = parse_frontmatter(&content).unwrap_or_default();
    let line_offsets = create_line_offsets(&content);

    let mut model = DataModel::new(None, Some(config.clone()));
    let (objects, enums) = parse_model_components(&content, &line_offsets, &mut model);

    process_model_components(&mut model, objects, enums, &config);
    merge_imports(&mut model, config.imports, path);

    validate_model(&model)?;
    Ok(model)
}

/// Creates a vector of line offset positions from content.
///
/// # Arguments
/// * `content` - The content to analyze
///
/// # Returns
/// * Vector of line offset positions
fn create_line_offsets(content: &str) -> Vec<usize> {
    content
        .char_indices()
        .filter(|(_, c)| *c == '\n')
        .map(|(i, _)| i)
        .collect()
}

/// Parses objects and enums from the markdown content.
///
/// # Arguments
/// * `content` - The markdown content to parse
/// * `line_offsets` - Vector of line offset positions
/// * `model` - Mutable reference to the data model
///
/// # Returns
/// * Tuple containing vectors of objects and enums
fn parse_model_components(
    content: &str,
    line_offsets: &[usize],
    model: &mut DataModel,
) -> (Vec<Object>, Vec<Enumeration>) {
    let mut objects = Vec::new();
    let mut enums = Vec::new();

    // Parse objects
    let mut options = Options::empty();
    options.insert(Options::ENABLE_HEADING_ATTRIBUTES);
    let mut iterator = Parser::new_ext(content, options).into_offset_iter();
    let mut state = ParserState::OutsideDefinition;

    while let Some(event) = iterator.next() {
        process_object_event(
            content,
            &mut iterator,
            &mut objects,
            event,
            model,
            &mut state,
            line_offsets,
        );
    }

    // Parse enums
    let mut iterator = Parser::new(content).into_offset_iter();
    while let Some((event, range)) = iterator.next() {
        process_enum_event(
            content,
            &mut iterator,
            &mut enums,
            (event, range),
            line_offsets,
        );
    }

    (objects, enums)
}

/// Processes and filters model components, applying inheritance and internal types.
///
/// # Arguments
/// * `model` - Mutable reference to the data model
/// * `objects` - Vector of parsed objects
/// * `enums` - Vector of parsed enums
/// * `config` - Reference to the configuration
fn process_model_components(
    model: &mut DataModel,
    objects: Vec<Object>,
    enums: Vec<Enumeration>,
    config: &FrontMatter,
) {
    let allow_empty = &config.allow_empty;

    // Filter and set components
    model.enums = enums.into_iter().filter(|e| e.has_values()).collect();
    model.objects = objects
        .into_iter()
        .filter(|o| {
            if *allow_empty {
                !&model.enums.iter().any(|e| e.name == o.name)
            } else {
                o.has_attributes()
            }
        })
        .collect();

    set_enum_attributes(model);
    add_internal_types(model);
    add_parent_types(model).expect("Failed to add parent types");
}

/// Merges imported models into the main model.
///
/// # Arguments
/// * `model` - Mutable reference to the data model
/// * `imports` - The imports configuration
/// * `path` - Optional path to the markdown file
fn merge_imports(model: &mut DataModel, imports: HashMap<String, ImportType>, path: Option<&Path>) {
    for (_prefix, import) in imports {
        let model_to_merge = import.fetch(path).unwrap();
        model.merge(&model_to_merge);
    }
}

/// Validates the model and returns any validation errors.
///
/// # Arguments
/// * `model` - Reference to the data model to validate
///
/// # Returns
/// * `Result<(), Validator>` - Ok if valid, Err with validator if invalid
#[allow(clippy::result_large_err)]
fn validate_model(model: &DataModel) -> Result<(), Validator> {
    let mut validator = Validator::new();
    validator.validate(model);

    if !validator.is_valid {
        return Err(validator);
    }
    Ok(())
}

fn clean_content(content: &str) -> String {
    // Remove all html tags
    let re = Regex::new(r"<[^>]*>").unwrap();
    let content = re.replace_all(content, "").to_string();

    // Remove all Markdown links
    let re = Regex::new(r"\[([^]]+)]\([^)]+\)").unwrap();
    let content = re.replace_all(content.as_str(), "$1").to_string();

    content
}

// Helper function to convert byte offset to line and column numbers
fn get_position(content: &str, line_offsets: &[usize], start: usize, end: usize) -> Position {
    let line = match line_offsets.binary_search(&start) {
        Ok(line) => line + 1,
        Err(line) => line + 1,
    };

    // Get the line content
    let line_start = if line > 1 { line_offsets[line - 2] } else { 0 };
    let line_end = if line <= line_offsets.len() {
        line_offsets[line - 1]
    } else {
        content.len()
    };
    let line_content = &content[line_start..line_end];

    // Count leading whitespace
    let leading_space = line_content
        .chars()
        .take_while(|c| c.is_whitespace())
        .count();

    // Calculate column numbers, adding leading whitespace to start
    let start_col = if line > 1 {
        start - line_offsets[line - 2] + leading_space - 1
    } else {
        start + 1 + leading_space
    };

    let end_col = if line <= line_offsets.len() {
        line_offsets[line - 1] - (if line > 1 { line_offsets[line - 2] } else { 0 })
    } else {
        end - (if line > 1 { line_offsets[line - 2] } else { 0 })
    };

    Position {
        line,
        column: PositionRange {
            start: start_col,
            end: end_col,
        },
        offset: PositionRange { start, end },
    }
}

/// Processes a single Markdown event for object extraction.
///
/// # Arguments
///
/// * `content` - The full content of the markdown file
/// * `iterator` - A mutable reference to the parser iterator
/// * `objects` - A mutable reference to the vector of objects
/// * `event` - The current Markdown event and its range
/// * `model` - A mutable reference to the data model
/// * `state` - A mutable reference to the parser state
/// * `line_offsets` - A reference to the line offsets of the file
fn process_object_event(
    content: &str,
    iterator: &mut pulldown_cmark::OffsetIter,
    objects: &mut Vec<Object>,
    event: (Event, std::ops::Range<usize>),
    model: &mut DataModel,
    state: &mut ParserState,
    line_offsets: &[usize],
) {
    let (event, range) = event;

    match event {
        Event::Start(tag) if tag == H1 => {
            handle_h1_event(iterator, model);
        }
        Event::Start(tag) if tag == H2 => {
            *state = ParserState::OutsideDefinition;
        }
        Event::Start(tag) if tag == H3 => {
            handle_h3_start(content, iterator, objects, state, line_offsets, range);
        }
        Event::End(tag) if tag == H3_END => {
            *state = ParserState::InDefinition;
        }
        Event::Text(CowStr::Borrowed(text)) if text.starts_with(":") => {
            handle_type_annotation(objects, text);
        }
        Event::Text(CowStr::Borrowed("[")) => {
            if *state == ParserState::InHeading {
                handle_inheritance(objects, iterator);
            }
        }
        Event::Start(Tag::List(None)) => {
            if *state == ParserState::OutsideDefinition {
                return;
            }

            handle_list_start(content, iterator, objects, line_offsets, range);
        }
        Event::Start(Tag::Item) => {
            if *state == ParserState::OutsideDefinition {
                return;
            }

            handle_list_item(content, iterator, objects, line_offsets, range);
        }
        Event::Text(text) if text.to_string() == "]" => {
            handle_array_marker(objects);
        }
        Event::Text(text) => {
            if *state == ParserState::InDefinition {
                handle_docstring(objects, text);
            }
        }
        _ => {}
    }
}

/// Handles H1 heading events by setting the model name.
///
/// # Arguments
///
/// * `iterator` - A mutable reference to the markdown parser iterator
/// * `model` - A mutable reference to the data model to update
fn handle_h1_event(iterator: &mut pulldown_cmark::OffsetIter, model: &mut DataModel) {
    model.name = Some(extract_name(iterator));
}

/// Handles H3 heading start events by creating and adding a new object.
///
/// # Arguments
///
/// * `content` - The full content of the markdown file
/// * `iterator` - A mutable reference to the parser iterator
/// * `objects` - A mutable reference to the vector of objects
/// * `state` - A mutable reference to the parser state
/// * `line_offsets` - A reference to the line offsets of the file
/// * `range` - The byte range of the current event
fn handle_h3_start(
    content: &str,
    iterator: &mut pulldown_cmark::OffsetIter,
    objects: &mut Vec<Object>,
    state: &mut ParserState,
    line_offsets: &[usize],
    range: std::ops::Range<usize>,
) {
    *state = ParserState::InHeading;
    let mut object = process_object_heading(iterator);
    object.set_position(get_position(content, line_offsets, range.start, range.end));
    objects.push(object);
}

/// Handles type annotations in the format ": type".
///
/// # Arguments
///
/// * `objects` - A mutable slice of objects
/// * `text` - The text containing the type annotation
fn handle_type_annotation(objects: &mut [Object], text: &str) {
    let attribute = objects.last_mut().unwrap().get_last_attribute();
    attribute
        .add_option(RawOption::new(
            "type".to_string(),
            text.to_string().trim_start_matches(':').trim().to_string(),
        ))
        .unwrap();
}

/// Handles inheritance declarations in object headings.
///
/// # Arguments
///
/// * `objects` - A mutable slice of objects
/// * `iterator` - A mutable reference to the parser iterator
fn handle_inheritance(objects: &mut [Object], iterator: &mut pulldown_cmark::OffsetIter) {
    let last_object = objects.last_mut().unwrap();
    let parent = iterator.next();

    match parent {
        Some((Event::Text(text), _)) if text.to_string() != "]" => {
            last_object.parent = Some(text.to_string());
        }
        _ => {
            error!(
                "[{}] {}: Opening bracket but no parent name. Inheritance wont be applied",
                last_object.name.bold(),
                "SyntaxError".bold(),
            );
            panic!("Inheritance syntax error. Expected parent name after opening bracket.");
        }
    }
}

/// Handles the start of a list, processing either attributes or attribute options.
///
/// # Arguments
///
/// * `content` - The full content of the markdown file
/// * `iterator` - A mutable reference to the parser iterator
/// * `objects` - A mutable reference to the vector of objects
/// * `line_offsets` - A reference to the line offsets of the file
/// * `range` - The byte range of the current event
fn handle_list_start(
    content: &str,
    iterator: &mut pulldown_cmark::OffsetIter,
    objects: &mut [Object],
    line_offsets: &[usize],
    range: std::ops::Range<usize>,
) {
    let last_object = objects.last_mut().unwrap();
    if !last_object.has_attributes() {
        iterator.next();
        let (required, attr_name, dtypes) = extract_attr_name_required(iterator);
        let mut attribute = attribute::Attribute::new(attr_name, required);

        if let Some((key, dtypes)) = dtypes {
            attribute.add_option(RawOption::new(key, dtypes)).unwrap();
        }

        attribute.set_position(get_position(content, line_offsets, range.start, range.end));
        objects.last_mut().unwrap().add_attribute(attribute);
    } else {
        let attr_strings = extract_attribute_options(iterator);
        for attr_string in attr_strings {
            distribute_attribute_options(objects, attr_string);
        }
    }
}

/// Handles list items by creating new attributes.
///
/// # Arguments
///
/// * `content` - The full content of the markdown file
/// * `iterator` - A mutable reference to the parser iterator
/// * `objects` - A mutable reference to the vector of objects
/// * `line_offsets` - A reference to the line offsets of the file
/// * `range` - The byte range of the current event
fn handle_list_item(
    content: &str,
    iterator: &mut pulldown_cmark::OffsetIter,
    objects: &mut [Object],
    line_offsets: &[usize],
    range: std::ops::Range<usize>,
) {
    let (required, attr_string, dtypes) = extract_attr_name_required(iterator);
    let mut attribute = attribute::Attribute::new(attr_string, required);

    if let Some((key, dtypes)) = dtypes {
        attribute.add_option(RawOption::new(key, dtypes)).unwrap();
    }

    attribute.set_position(get_position(content, line_offsets, range.start, range.end));
    objects.last_mut().unwrap().add_attribute(attribute);
}

/// Handles array markers by setting the is_array flag on the last attribute.
///
/// # Arguments
///
/// * `objects` - A mutable slice of objects
fn handle_array_marker(objects: &mut [Object]) {
    let last_object = objects.last_mut().unwrap();
    if last_object.has_attributes() {
        let last_attribute = last_object.get_last_attribute();
        last_attribute.is_array = true;
    }
}

/// Handles docstring text by appending it to the last object's docstring.
///
/// # Arguments
///
/// * `objects` - A mutable slice of objects
/// * `text` - The text to append to the docstring
fn handle_docstring(objects: &mut [Object], text: CowStr) {
    let last_object = objects.last_mut().unwrap();
    last_object.docstring.push_str(text.as_ref());
}

/// Processes the heading of an object.
///
/// # Arguments
///
/// * `iterator` - A mutable reference to the parser iterator.
///
/// # Returns
///
/// An `Object` created from the heading.
fn process_object_heading(iterator: &mut OffsetIter) -> object::Object {
    let heading = extract_name(iterator);
    let term = extract_object_term(&heading);
    let name = heading.split_whitespace().next().unwrap().to_string();

    object::Object::new(name, term)
}

/// Extracts the name from the next text event in the iterator.
///
/// # Arguments
///
/// * `iterator` - A mutable reference to the parser iterator.
///
/// # Returns
///
/// A string containing the extracted name.
fn extract_name(iterator: &mut OffsetIter) -> String {
    if let Some((Event::Text(text), _)) = iterator.next() {
        return text.to_string();
    }

    // Try for two text events
    for _ in 0..2 {
        if let Some((Event::Text(text), _)) = iterator.next() {
            return text.to_string();
        }
    }

    panic!("Could not extract name: Got {:?}", iterator.next().unwrap());
}

/// Extracts the attribute name and its required status from the iterator.
///
/// # Arguments
///
/// * `iterator` - A mutable reference to the parser iterator.
///
/// # Returns
///
/// A tuple containing a boolean indicating if the attribute is required and the attribute name.
fn extract_attr_name_required(
    iterator: &mut OffsetIter,
) -> (bool, String, Option<(String, String)>) {
    let mut next = iterator.next();

    // If there are newlines between the attributes, the likely a paragraph
    // is being started. We need to consume the paragraph and the following
    // text event.
    if let Some((Event::Start(Tag::Paragraph), _)) = next {
        next = iterator.next();
    }

    match next {
        Some((Event::Text(text), _)) => {
            if let Some((key, dtypes)) = shorthand_type(&text) {
                return (false, key, Some(dtypes));
            } else {
                return (false, text.to_string(), None);
            }
        }
        Some((Event::Start(Tag::Strong), _)) => {
            let next = iterator.next();
            let mut name = String::new();
            if let Some((Event::Text(text), _)) = next {
                name = text.to_string();
            }

            // Consume the Strong end tag
            iterator.next();

            return (true, name, None);
        }
        _ => {}
    }

    panic!("Could not extract attribute name. Please check the markdown file.");
}

/// Extracts a type definition from shorthand notation in the form "key: type".
///
/// # Arguments
///
/// * `text` - A string slice containing the potential shorthand type definition
///
/// # Returns
///
/// An `Option` containing a tuple of `(key, type)` strings if the text matches the shorthand format,
/// or `None` if it does not contain a colon separator.
fn shorthand_type(text: &str) -> Option<(String, (String, String))> {
    if let Some((key, dtypes)) = text.split_once(":") {
        Some((
            key.trim().to_string(),
            ("type".to_string(), dtypes.trim().to_string()),
        ))
    } else {
        None
    }
}

/// Extracts the term from an object heading.
///
/// # Arguments
///
/// * `heading` - A string slice containing the heading.
///
/// # Returns
///
/// An optional string containing the extracted term.
fn extract_object_term(heading: &str) -> Option<String> {
    let re = Regex::new(r"\(([^)]+)\)").unwrap();

    re.captures(heading)
        .map(|cap| cap.get(1).map_or("", |m| m.as_str()).to_string())
}

/// Extracts attribute options from the iterator.
///
/// # Arguments
///
/// * `iterator` - A mutable reference to the parser iterator.
///
/// # Returns
///
/// A vector of strings containing the extracted attribute options.
fn extract_attribute_options(iterator: &mut OffsetIter) -> Vec<String> {
    let mut options = Vec::new();
    while let Some((next, _)) = iterator.next() {
        match next {
            Event::Start(Tag::Item) => {
                let name = extract_name(iterator);
                options.push(name);
            }
            Event::End(TagEnd::List(false)) => {
                break;
            }
            Event::Text(text) if text.to_string() == "[" => {
                let last_option = options.last_mut().unwrap();
                let lower = last_option.to_lowercase();
                if lower.contains("pattern:") || lower.contains("regex:") {
                    *last_option = format!("{}[", last_option);
                } else {
                    *last_option = format!("{}[]", last_option.trim());
                }
            }
            Event::Text(text) if text.to_string() == "]" => {
                let last_option = options.last_mut().unwrap();
                let lower = last_option.to_lowercase();
                if lower.contains("pattern:") || lower.contains("regex:") {
                    *last_option = format!("{}]", last_option);
                }
            }
            Event::Text(text) if text.to_string() != "]" => {
                let last_option = options.last_mut().unwrap();
                let lower = last_option.to_lowercase();
                if lower.contains("description:") {
                    *last_option = format!("{} {}", last_option.trim(), text);
                } else if lower.contains("pattern:") || lower.contains("regex:") {
                    *last_option = format!("{}{}", last_option.trim(), text);
                }
            }
            _ => {}
        }
    }
    options
}

/// Adds an option to the last attribute of the last object in the list.
///
/// # Arguments
///
/// * `objects` - A mutable reference to the list of objects.
/// * `key` - The key of the attribute option.
/// * `value` - The value of the attribute option.
fn add_option_to_last_attribute(
    objects: &mut [object::Object],
    key: String,
    value: String,
) -> Result<(), Box<dyn Error>> {
    let last_attr = objects.last_mut().unwrap().get_last_attribute();
    let option = RawOption::new(key, value);
    last_attr.add_option(option)?;

    Ok(())
}

/// Distributes attribute options among the objects.
///
/// # Arguments
///
/// * `objects` - A mutable reference to the list of objects.
/// * `attr_string` - A string containing the attribute or option.
///
/// # Returns
///
/// An optional unit type.
fn distribute_attribute_options(objects: &mut [object::Object], attr_string: String) -> Option<()> {
    if attr_string.contains(':') {
        let (key, value) = process_option(&attr_string);
        add_option_to_last_attribute(objects, key, value).expect("Failed to add option");
        return None;
    }

    objects
        .last_mut()
        .unwrap()
        .create_new_attribute(attr_string, false);

    None
}

/// Processes an attribute option string.
///
/// # Arguments
///
/// * `option` - A string containing the attribute option.
///
/// # Returns
///
/// A tuple containing the key and value of the attribute option.
fn process_option(option: &String) -> (String, String) {
    let parts: Vec<&str> = option.split(':').collect();

    assert!(
        parts.len() > 1,
        "Attribute {} does not have a valid option",
        option
    );

    let key = parts[0].trim();
    let value = parts[1..].join(":");

    (key.to_string(), value.trim().to_string())
}

/// Processes a single Markdown event for enumeration extraction.
///
/// # Arguments
///
/// * `iterator` - A mutable reference to the parser iterator.
/// * `enums` - A mutable reference to the vector of enumerations.
/// * `event` - The current Markdown event.
/// * `range` - The range of the event.
/// * `line_offsets` - The line offsets of the file.
pub fn process_enum_event(
    content: &str,
    iterator: &mut OffsetIter,
    enums: &mut Vec<Enumeration>,
    event: (Event, std::ops::Range<usize>),
    line_offsets: &[usize],
) {
    let (event, range) = event;

    match event {
        Event::Start(tag) if tag == H3 => {
            let enum_name = extract_name(iterator);
            let mut enum_obj = Enumeration {
                name: enum_name,
                mappings: BTreeMap::new(),
                docstring: "".to_string(),
                position: None,
            };
            enum_obj.set_position(get_position(content, line_offsets, range.start, range.end));
            enums.push(enum_obj);
        }
        Event::Start(Tag::CodeBlock(pulldown_cmark::CodeBlockKind::Fenced(_))) => {
            let event = iterator.next().unwrap();
            if let (Event::Text(text), _) = event {
                let mappings = text.to_string();

                if enums.last_mut().is_some() {
                    let enum_obj = enums.last_mut().unwrap();
                    process_enum_mappings(enum_obj, mappings);
                }
            }
        }
        _ => {}
    }
}

/// Processes enumeration mappings from a code block.
///
/// # Arguments
///
/// * `enum_obj` - A mutable reference to the enumeration object.
/// * `mappings` - A string containing the mappings.
fn process_enum_mappings(enum_obj: &mut Enumeration, mappings: String) {
    let lines = mappings.split('\n');
    for line in lines {
        let parts: Vec<&str> = line.split('=').collect();
        if parts.len() != 2 {
            // Skip empty lines or lines that do not contain a mapping
            continue;
        }

        // Extract key and value, insert into enum object
        let key = parts[0].trim().replace('"', "");
        let value = parts[1].trim().replace('"', "");
        enum_obj.mappings.insert(key.to_string(), value.to_string());
    }
}

/// Adds parent types to the objects in the model.
///
/// # Arguments
///
/// * `model` - A mutable reference to the data model.
///
///
/// # Panics
///
/// Panics if an object has a parent that does not exist.
///
/// # Errors
///
/// An error is logged if an object has a parent that does not exist.
///
fn add_parent_types(model: &mut DataModel) -> Result<(), Box<dyn Error>> {
    // Filter and clone the objects without a parent
    let parents: Vec<Object> = model
        .objects
        .iter()
        .filter(|o| o.parent.is_none())
        .cloned()
        .collect();

    let mut to_merge: Vec<DataModel> = vec![];
    let mut added_internals: Vec<String> = vec![];

    // REFACTOR THIS
    // Iterate over the objects and add the parent attributes
    for object in model.objects.iter_mut() {
        if let Some(parent_name) = &object.parent {
            if let Some(parent) = parents.iter().find(|o| o.name == *parent_name) {
                object.attributes.extend(parent.attributes.clone());
            } else if let Some(internal_type) = MD_MODEL_TYPES.get(parent_name.as_str()) {
                let mut internal_type = serde_json::from_str::<DataModel>(internal_type)
                    .expect("Failed to parse internal data type");

                // Pop the first object parent and add the attributes
                let target_obj = internal_type.objects[0].clone();
                internal_type.objects.remove(0);

                object.attributes.extend(target_obj.attributes.clone());

                if !added_internals.contains(parent_name) {
                    to_merge.push(internal_type);
                    added_internals.push(parent_name.clone());
                }
            } else {
                error!(
                    "[{}] {}: Parent {} does not exist.",
                    object.name.red().bold(),
                    "InheritanceError".bold(),
                    parent_name.red().bold(),
                );

                return Err("Object has a parent that does not exist".into());
            }
        }
    }

    for internal in to_merge {
        model.merge(&internal);
    }

    Ok(())
}

fn add_internal_types(model: &mut DataModel) {
    // Get all datatypes within the model
    let mut all_types = vec![];
    for object in &model.objects {
        for attr in &object.attributes {
            all_types.extend(attr.dtypes.clone());
        }
    }

    let object_names = model
        .objects
        .iter()
        .map(|obj| obj.name.clone())
        .collect::<Vec<String>>();

    for (name, content) in MD_MODEL_TYPES.iter() {
        if object_names.contains(&name.to_string()) {
            continue;
        }

        if all_types.contains(&name.to_string()) {
            model.merge(
                &serde_json::from_str::<DataModel>(content)
                    .expect("Failed to parse internal data type"),
            )
        }
    }
}

/// Sets the `is_enum` flag for attributes that are enumerations.
///
/// This function iterates through all objects and their attributes in the data model.
/// If an attribute's data types match any of the enumeration names, the `is_enum` flag
/// is set to `true`. If an attribute has data types that do not match any enumeration,
/// an error is returned.
///
/// # Arguments
///
/// * `model` - A mutable reference to the data model.
fn set_enum_attributes(model: &mut DataModel) {
    let enums = model
        .enums
        .iter()
        .map(|e| e.name.clone())
        .collect::<Vec<String>>();

    for object in model.objects.iter_mut() {
        for attr in object.attributes.iter_mut() {
            let enum_dtypes: Vec<String> = attr
                .dtypes
                .iter()
                .filter(|dtype| enums.contains(dtype))
                .cloned()
                .collect();
            if !enum_dtypes.is_empty() && enum_dtypes.len() == attr.dtypes.len() {
                attr.is_enum = true;
            }
        }
    }
}

/// Represents the different keys that can be used for attribute options.
pub(crate) enum OptionKey {
    /// Represents the data type of the attribute.
    Type,
    /// Represents the term associated with the attribute.
    Term,
    /// Represents the description of the attribute.
    Description,
    /// Represents the XML type information for the attribute.
    Xml,
    /// Represents the default value for the attribute.
    Default,
    /// Indicates if the attribute can have multiple values.
    Multiple,
    /// Represents any other option not covered by the predefined keys.
    Other,
}

impl OptionKey {
    /// Converts a string to an `OptionKey`.
    ///
    /// # Arguments
    ///
    /// * `key` - The string representation of the key.
    ///
    /// # Returns
    ///
    /// An `OptionKey` corresponding to the given string.
    pub fn from_str(key: &str) -> Self {
        match key.to_lowercase().as_str() {
            "type" => OptionKey::Type,
            "term" => OptionKey::Term,
            "description" => OptionKey::Description,
            "xml" => OptionKey::Xml,
            "default" => OptionKey::Default,
            "multiple" => OptionKey::Multiple,
            _ => OptionKey::Other,
        }
    }
}
