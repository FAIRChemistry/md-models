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

use colored::Colorize;
use core::panic;
use lazy_static::lazy_static;
use log::error;
use std::collections::BTreeMap;
use std::error::Error;

use pulldown_cmark::{CowStr, Event, Parser, Tag};
use regex::Regex;

use crate::attribute;
use crate::datamodel::DataModel;
use crate::object::{self, Enumeration, Object};
use crate::validation::Validator;

use super::frontmatter::parse_frontmatter;

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

#[derive(Debug, PartialEq, Eq)]
enum ParserState {
    InDefinition,
    OutsideDefinition,
    InHeading,
}

/// Parses a Markdown file at the given path and returns a `DataModel`.
///
/// # Arguments
///
/// * `path` - A reference to the path of the Markdown file.
///
/// # Returns
///
/// A `Result` containing a `DataModel` on success or an error on failure.
pub fn parse_markdown(content: &str) -> Result<DataModel, Validator> {
    // Remove HTML and links
    let content = clean_content(content);

    // Parse the frontmatter
    let config = parse_frontmatter(&content);

    // Parse the markdown content
    let parser = Parser::new(&content);
    let mut iterator = parser.into_iter();

    let mut objects = Vec::new();
    let mut enums = Vec::new();

    let mut model = DataModel::new(None, config);

    // Extract objects from the markdown file
    let mut state = ParserState::OutsideDefinition;
    while let Some(event) = iterator.next() {
        process_object_event(&mut iterator, &mut objects, event, &mut model, &mut state);
    }

    // Reset the iterator
    let parser = Parser::new(&content);
    let mut iterator = parser.into_iter();

    while let Some(event) = iterator.next() {
        process_enum_event(&mut iterator, &mut enums, event);
    }

    // Filter empty objects and enums
    model.enums = enums.into_iter().filter(|e| e.has_values()).collect();
    model.objects = objects.into_iter().filter(|o| o.has_attributes()).collect();

    // Add internal types, if used
    add_internal_types(&mut model);

    // Apply inheritance (THIS NEEDS TO BE SPLIT INTO A SEPARATE FUNCTION)
    add_parent_types(&mut model).expect("Failed to add parent types");

    // Validate the model
    let mut validator = Validator::new();
    validator.validate(&model);

    if !validator.is_valid {
        return Err(validator);
    }

    Ok(model)
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

/// Processes a single Markdown event for object extraction.
///
/// # Arguments
///
/// * `iterator` - A mutable reference to the parser iterator.
/// * `objects` - A mutable reference to the vector of objects.
/// * `event` - The current Markdown event.
/// * `model` - A mutable reference to the data model.
fn process_object_event(
    iterator: &mut Parser,
    objects: &mut Vec<Object>,
    event: Event,
    model: &mut DataModel,
    state: &mut ParserState,
) {
    match event {
        Event::Start(Tag::Heading(1)) => {
            model.name = Some(extract_name(iterator));
        }
        Event::Start(Tag::Heading(2)) => {
            *state = ParserState::OutsideDefinition;
        }
        Event::Start(Tag::Heading(3)) => {
            *state = ParserState::InHeading;
            let object = process_object_heading(iterator);
            objects.push(object);
        }
        Event::End(Tag::Heading(3)) => {
            *state = ParserState::InDefinition;
        }
        Event::Text(CowStr::Borrowed("[")) => {
            if *state == ParserState::InHeading {
                // Extract parent from the next text event
                let last_object = objects.last_mut().unwrap();
                let parent = iterator.next();

                match parent {
                    Some(Event::Text(text)) if text.to_string() != "]" => {
                        last_object.parent = Some(text.to_string());
                    }
                    _ => {
                        error!(
                            "[{}] {}: Opening bracket but no parent name. Inheritance wont be applied",
                            last_object.name.bold(),
                            "SyntaxError".bold(),
                        );

                        panic!(
                            "Inheritance syntax error. Expected parent name after opening bracket."
                        );
                    }
                }
            }
        }
        Event::Start(Tag::List(None)) => {
            if *state == ParserState::OutsideDefinition {
                return;
            }

            let last_object = objects.last_mut().unwrap();
            if !last_object.has_attributes() {
                iterator.next();
                let (required, attr_name) = extract_attr_name_required(iterator);
                let attribute = attribute::Attribute::new(attr_name, required);
                objects.last_mut().unwrap().add_attribute(attribute);
            } else {
                let attr_strings = extract_attribute_options(iterator);
                for attr_string in attr_strings {
                    distribute_attribute_options(objects, attr_string);
                }
            }
        }
        Event::Start(Tag::Item) => {
            if *state == ParserState::OutsideDefinition {
                return;
            }

            let (required, attr_string) = extract_attr_name_required(iterator);
            let attribute = attribute::Attribute::new(attr_string, required);
            objects.last_mut().unwrap().add_attribute(attribute);
        }
        Event::Text(text) => {
            if *state == ParserState::InDefinition {
                let last_object = objects.last_mut().unwrap();
                last_object.docstring.push_str(text.as_ref());
            }
        }
        _ => {}
    }
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
fn process_object_heading(iterator: &mut Parser) -> object::Object {
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
fn extract_name(iterator: &mut Parser) -> String {
    if let Some(Event::Text(text)) = iterator.next() {
        return text.to_string();
    }

    // Try for two text events
    for _ in 0..2 {
        if let Some(Event::Text(text)) = iterator.next() {
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
fn extract_attr_name_required(iterator: &mut Parser) -> (bool, String) {
    if let Some(Event::Text(text)) = iterator.next() {
        return (false, text.to_string());
    }

    // Try for two text events
    for _ in 0..2 {
        if let Some(Event::Text(text)) = iterator.next() {
            return (true, text.to_string());
        }
    }

    panic!("Could not extract name. Plesae check the markdown file.");
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
fn extract_attribute_options(iterator: &mut Parser) -> Vec<String> {
    let mut options = Vec::new();
    while let Some(next) = iterator.next() {
        match next {
            Event::Start(Tag::Item) => {
                let name = extract_name(iterator);
                options.push(name);
            }
            Event::End(Tag::List(None)) => {
                break;
            }
            Event::Text(text) if text.to_string() == "[" => {
                let last_option = options.last_mut().unwrap();
                *last_option = format!("{}[]", last_option);
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
    let option = attribute::AttrOption::new(key, value);
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
pub fn process_enum_event(iterator: &mut Parser, enums: &mut Vec<Enumeration>, event: Event) {
    match event {
        Event::Start(Tag::Heading(3)) => {
            let enum_name = extract_name(iterator);
            let enum_obj = Enumeration {
                name: enum_name,
                mappings: BTreeMap::new(),
                docstring: "".to_string(),
            };
            enums.push(enum_obj);
        }
        Event::Start(Tag::CodeBlock(pulldown_cmark::CodeBlockKind::Fenced(_))) => {
            let event = iterator.next().unwrap();
            if let Event::Text(text) = event {
                let mappings = text.to_string();
                let enum_obj = enums.last_mut().unwrap();
                process_enum_mappings(enum_obj, mappings);
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
