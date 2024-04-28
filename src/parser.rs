use pulldown_cmark::{Event, Parser, Tag};
use regex::Regex;

use crate::attribute;
use crate::object;

pub fn process_event(iterator: &mut Parser, objects: &mut Vec<object::Object>, event: Event) {
    match event {
        Event::Start(Tag::Heading(level)) if level == 3 => {
            let object = process_object_heading(iterator);
            objects.push(object);
        }
        Event::Start(Tag::List(None)) => {
            let attr_strings = extract_attribute(iterator);
            for attr_string in attr_strings {
                distribute_attribute_options(objects, attr_string);
            }
        }
        Event::Start(Tag::Item) => {
            let attr_string = extract_name(iterator);
            let attribute = attribute::Attribute::new(attr_string);
            objects.last_mut().unwrap().add_attribute(attribute);
        }
        _ => {
            println!("Event not handled: {:?}", event)
        }
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
    } else {
        return String::new();
    }
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

fn extract_attribute(iterator: &mut Parser) -> Vec<String> {
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
    last_attr.add_option(key, value);
}

fn distribute_attribute_options(
    objects: &mut Vec<object::Object>,
    attr_string: String,
) -> Option<()> {
    if attr_string.contains(":") {
        let (key, value) = process_option(&attr_string);
        add_option_to_last_attribute(objects, key, value);
        return None;
    }

    objects
        .last_mut()
        .unwrap()
        .create_new_attribute(attr_string);

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
