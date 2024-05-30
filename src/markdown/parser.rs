use std::collections::BTreeMap;
use std::error::Error;
use std::fs;
use std::path::Path;

use pulldown_cmark::{Event, Parser, Tag};
use regex::Regex;

use crate::attribute;
use crate::datamodel::DataModel;
use crate::object::{self, Enumeration};

use super::frontmatter::parse_frontmatter;

pub fn parse_markdown(path: &Path) -> Result<DataModel, Box<dyn Error>> {
    if !path.exists() {
        return Err("File does not exist".into());
    }

    let content = fs::read_to_string(path).expect("Could not read file");
    let config = parse_frontmatter(content.as_str());
    let parser = Parser::new(&content);
    let mut iterator = parser.into_iter();

    let mut objects = Vec::new();
    let mut enums = Vec::new();

    let mut model = DataModel::new(None, config);

    // Extract objects from the markdown file
    while let Some(event) = iterator.next() {
        process_object_event(&mut iterator, &mut objects, event, &mut model);
    }

    // Reset the iterator
    let parser = Parser::new(&content);
    let mut iterator = parser.into_iter();

    while let Some(event) = iterator.next() {
        process_enum_event(&mut iterator, &mut enums, event)
    }

    model.enums = enums.into_iter().filter(|e| e.has_values()).collect();
    model.objects = objects.into_iter().filter(|o| o.has_attributes()).collect();

    Ok(model)
}

// Object processing //
// ----------------- //
fn process_object_event(
    iterator: &mut Parser,
    objects: &mut Vec<object::Object>,
    event: Event,
    model: &mut DataModel,
) {
    match event {
        // Heading processing
        Event::Start(Tag::Heading(1)) => {
            model.name = Some(extract_name(iterator));
        }
        Event::Start(Tag::Heading(3)) => {
            let object = process_object_heading(iterator);
            objects.push(object);
        }
        // Parsing the attributes of an object
        Event::Start(Tag::List(None)) => {
            // When the last object has no attributes, we need to parse
            // the initial attribute in the list here
            let last_object = objects.last_mut().unwrap();
            if !last_object.has_attributes() {
                iterator.next();
                let (required, attr_name) = extract_attr_name_required(iterator);
                let attribute = attribute::Attribute::new(attr_name, required);
                objects.last_mut().unwrap().add_attribute(attribute);
            } else {
                // Every other match within this list will be an option for the last attribute
                let attr_strings = extract_attribute_options(iterator);
                for attr_string in attr_strings {
                    distribute_attribute_options(objects, attr_string);
                }
            }
        }
        Event::Start(Tag::Item) => {
            let (required, attr_string) = extract_attr_name_required(iterator);
            let attribute = attribute::Attribute::new(attr_string, required);
            objects.last_mut().unwrap().add_attribute(attribute);
        }
        _ => {}
    }
}

fn process_object_heading(iterator: &mut Parser) -> object::Object {
    let heading = extract_name(iterator);
    let term = extract_object_term(&heading);
    let name = heading.split_whitespace().next().unwrap().to_string();

    object::Object::new(name, term)
}

fn extract_name(iterator: &mut Parser) -> String {
    if let Some(Event::Text(text)) = iterator.next() {
        return text.to_string();
    }

    panic!("Could not extract name: Got {:?}", iterator.next().unwrap());
}

fn extract_attr_name_required(iterator: &mut Parser) -> (bool, String) {
    // If it is a non required field, the name will be in the next event
    if let Some(Event::Text(text)) = iterator.next() {
        return (false, text.to_string());
    }

    // If it is a required field, the name will be in the next event
    let text = iterator.next().unwrap();
    if let Event::Text(text) = text {
        return (true, text.to_string());
    }

    panic!("Could not extract name: Got {:?}", text);
}

fn extract_object_term(heading: &str) -> Option<String> {
    // Example: Test (schema:test)
    // Extract the term "schema:test" using regex

    let re = Regex::new(r"\(([^)]+)\)").unwrap();

    re.captures(heading)
        .map(|cap| cap.get(1).map_or("", |m| m.as_str()).to_string())
}

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

fn add_option_to_last_attribute(objects: &mut [object::Object], key: String, value: String) {
    let last_attr = objects.last_mut().unwrap().get_last_attribute();
    let option = attribute::AttrOption::new(key, value);
    last_attr.add_option(option);
}

fn distribute_attribute_options(objects: &mut [object::Object], attr_string: String) -> Option<()> {
    // If the attribute string contains a colon, it is an option
    if attr_string.contains(':') {
        let (key, value) = process_option(&attr_string);
        add_option_to_last_attribute(objects, key, value);
        return None;
    }

    // If the attribute string does not contain a colon, it is a new attribute
    objects
        .last_mut()
        .unwrap()
        .create_new_attribute(attr_string, false);

    None
}

fn process_option(option: &String) -> (String, String) {
    // Split by colon, strip both results and return a tuple
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

// Enumeration processing //
// ---------------------- //
pub fn process_enum_event(iterator: &mut Parser, enums: &mut Vec<Enumeration>, event: Event) {
    match event {
        Event::Start(Tag::Heading(3)) => {
            let enum_name = extract_name(iterator);
            let enum_obj = Enumeration {
                name: enum_name,
                mappings: BTreeMap::new(),
            };
            enums.push(enum_obj);
        }
        Event::Start(Tag::CodeBlock(pulldown_cmark::CodeBlockKind::Fenced(_))) => {
            // If there is a code block, we need to extract the mappings
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
