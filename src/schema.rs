use std::collections::HashSet;

use serde_json::json;

use crate::markdown::attribute;
use crate::markdown::attribute::AttrOption;
use crate::markdown::object;
use crate::primitives::PrimitiveTypes;

static DEFINITIONS_KEY: &str = "definitions";

pub fn to_json_schema(name: &String, objects: &Vec<object::Object>) -> String {
    let obj = objects.iter().find(|o| o.name == *name).unwrap();
    let (mut schema, used_refs) = process_class(obj);

    for reference in used_refs {
        let sub_obj = objects.iter().find(|o| o.name == reference).unwrap();
        let (sub_properties, _) = process_class(sub_obj);

        let definitions = json!({
            "title": reference,
            "type": "object",
            "properties": sub_properties,
        });

        schema[DEFINITIONS_KEY][reference] = definitions;
    }

    serde_json::to_string_pretty(&schema).unwrap()
}

fn process_class(object: &object::Object) -> (serde_json::Value, HashSet<String>) {
    let mut all_refs = HashSet::new();
    let mut schema = json!({
        "title": object.name,
        "type": "object",
        "properties": {},
    });

    if object.term.is_some() {
        schema["term"] = json!(object.term.as_ref().unwrap());
    }

    for attribute in &object.attributes {
        let (primitives, references) = extract_primitives_and_refs(&attribute.dtypes);

        for primitive in primitives {
            process_primitive(&mut schema["properties"], attribute, &primitive);
        }

        for reference in references {
            all_refs.insert(reference.clone());
            process_reference(&mut schema["properties"], attribute, &reference);
        }
    }

    (schema, all_refs)
}

fn extract_primitives_and_refs(dtypes: &Vec<String>) -> (Vec<String>, Vec<String>) {
    let primitives = PrimitiveTypes::new();
    let references = primitives.filter_non_primitives(&dtypes);
    let primitives = primitives.filter_primitive(&dtypes);

    (primitives, references)
}

fn create_property(name: &String) -> serde_json::Value {
    return json!({
        "title": capitalize(&name),
    });
}

fn process_primitive(
    properties: &mut serde_json::Value,
    attribute: &attribute::Attribute,
    primitive: &String,
) {
    let name = &attribute.name;
    properties[name] = create_property(&name);

    set_primitive_dtype(properties, attribute, primitive);
    set_options(&mut properties[name], &attribute.options);
}

fn set_primitive_dtype(
    properties: &mut serde_json::Value,
    attribute: &attribute::Attribute,
    primitive: &String,
) {
    let is_array = attribute.is_array;
    let name = &attribute.name;
    let primitives = PrimitiveTypes::new();
    let json_dtype = primitives.dtype_to_json(primitive);

    if is_array {
        properties[name]["type"] = json!("array");
        properties[name]["items"] = json!({
            "type": json_dtype
        });

        return;
    }

    properties[name]["type"] = json!(json_dtype);
}

fn set_options(property: &mut serde_json::Value, options: &Vec<AttrOption>) {
    for option in options {
        property[option.key()] = json!(option.value());
    }
}

fn process_reference(
    properties: &mut serde_json::Value,
    attribute: &attribute::Attribute,
    reference: &String,
) {
    let name = &attribute.name;
    properties[name] = create_property(&name);

    set_ref_dtype(properties, attribute, reference);
    set_options(&mut properties[name], &attribute.options);
}

fn set_ref_dtype(
    properties: &mut serde_json::Value,
    attribute: &attribute::Attribute,
    reference: &String,
) {
    let name = &attribute.name;
    let def_path = format!("#/{}/{}", DEFINITIONS_KEY, reference);
    if attribute.is_array {
        properties[name]["type"] = json!("array");
        properties[name]["items"] = json!({
            "$ref": json!(def_path)
        });

        return;
    }

    properties[name]["$ref"] = json!(def_path);
}

fn capitalize(s: &str) -> String {
    s.chars()
        .enumerate()
        .map(|(i, c)| if i == 0 { c.to_ascii_uppercase() } else { c })
        .collect()
}
