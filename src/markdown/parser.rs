use std::error::Error;
use std::fs;
use std::path::Path;

use pulldown_cmark::{Event, Parser, Tag};
use regex::Regex;

use crate::attribute;
use crate::datamodel::DataModel;
use crate::object;

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

    let mut model = DataModel::new(None, Some(config));

    while let Some(event) = iterator.next() {
        process_event(&mut iterator, &mut objects, event, &mut model);
    }

    model.objects = objects;

    return Ok(model);
}

pub fn process_event(
    iterator: &mut Parser,
    objects: &mut Vec<object::Object>,
    event: Event,
    model: &mut DataModel,
) {
    match event {
        Event::Start(Tag::Heading(level)) if level == 1 => {
            // Get the title of the data model
            model.name = Some(extract_name(iterator));
        }
        Event::Start(Tag::Heading(level)) if level == 3 => {
            let object = process_object_heading(iterator);
            objects.push(object);
        }
        Event::Start(Tag::List(None)) => {
            // When the last object has no attributes, we need to parse
            // the initial attribute in the list here
            let last_object = objects.last_mut().unwrap();
            if !last_object.has_attributes() {
                iterator.next();
                let (required, attr_name) = extract_attr_name_required(iterator);
                let attribute = attribute::Attribute::new(attr_name, required);
                objects.last_mut().unwrap().add_attribute(attribute);
                return;
            }

            // Every other match within this list will be an option for the last attribute
            let attr_strings = extract_attribute_options(iterator);
            for attr_string in attr_strings {
                distribute_attribute_options(objects, attr_string);
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

    return object::Object::new(name, object::ObjectType::Object, term);
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

fn extract_object_term(heading: &String) -> Option<String> {
    // Example: Test (schema:test)
    // Extract the term "schema:test" using regex

    let re = Regex::new(r"\(([^)]+)\)").unwrap();
    let matches = re.captures(heading);

    if matches.is_none() {
        return None;
    }

    let term = matches.unwrap().get(1).unwrap().as_str();

    Some(term.to_string())
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

fn add_option_to_last_attribute(objects: &mut Vec<object::Object>, key: String, value: String) {
    let last_attr = objects.last_mut().unwrap().get_last_attribute();
    let option = attribute::AttrOption::new(key, value);
    last_attr.add_option(option);
}

fn distribute_attribute_options(
    objects: &mut Vec<object::Object>,
    attr_string: String,
) -> Option<()> {
    if attr_string.contains(":") {
        // This is an option
        let (key, value) = process_option(&attr_string);
        add_option_to_last_attribute(objects, key, value);
        return None;
    }

    objects
        .last_mut()
        .unwrap()
        .create_new_attribute(attr_string, false);

    return None;
}

fn process_option(option: &String) -> (String, String) {
    // Split by colon, strip both results and return a tuple
    let parts: Vec<&str> = option.split(":").collect();

    assert!(
        parts.len() > 1,
        "Attribute {} does not have a valid option",
        option
    );

    let key = parts[0].trim();
    let value = parts[1..].join(":");

    (key.to_string(), value.trim().to_string())
}
