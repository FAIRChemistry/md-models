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

use crate::datamodel::DataModel;
use crate::exporters::Templates;
use crate::json::export::to_json_schema;
use crate::validation::Validator;
use wasm_bindgen::prelude::*;

// Add console.log support for debugging
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

/// Parses the given markdown content into a `DataModel` and returns it as a `JsValue`.
///
/// # Arguments
///
/// * `markdown_content` - A string slice that holds the markdown content to be parsed.
///
/// # Returns
///
/// A `DataModel` or an error `JsError`.
#[wasm_bindgen]
pub fn parse_model(markdown_content: &str) -> Result<DataModel, JsError> {
    DataModel::from_markdown_string(markdown_content)
        .map_err(|e| JsError::new(&format!("Error parsing markdown content: {}", e)))
}

/// Converts the given markdown content into a specified template format.
///
/// # Arguments
///
/// * `markdown_content` - A string slice that holds the markdown content to be converted.
/// * `template` - The template format to convert the markdown content into.
///
/// # Returns
///
/// A `Result` which is:
/// - `Ok(String)` if the conversion is successful.
/// - `Err(JsValue)` if there is an error during parsing or conversion.
#[wasm_bindgen]
pub fn convert_to(markdown_content: &str, template: Templates) -> Result<String, JsValue> {
    let mut model = DataModel::from_markdown_string(markdown_content)
        .map_err(|e| JsValue::from_str(&format!("Error parsing markdown content: {}", e)))?;

    model
        .convert_to(&template, None)
        .map_err(|e| JsValue::from_str(&format!("Error converting markdown content: {}", e)))
}

/// Returns the JSON schema for the given markdown content.
///
/// # Arguments
///
/// * `markdown_content` - A string slice that holds the markdown content to be converted.
/// * `root` - The root object to use for the schema.
/// * `openai` - Whether to remove options from the schema properties. OpenAI does not support options.
///
/// # Returns
///
/// A `Result` which is:
/// - `Ok(JsValue)` if the conversion is successful.
/// - `Err(JsValue)` if there is an error during parsing or conversion.
#[wasm_bindgen]
pub fn json_schema(
    markdown_content: &str,
    root: Option<String>,
    openai: bool,
) -> Result<String, JsValue> {
    let model = DataModel::from_markdown_string(markdown_content)
        .map_err(|e| JsValue::from_str(&format!("Error parsing markdown content: {}", e)))?;

    let root = match root {
        Some(root) => root,
        None => model
            .objects
            .first()
            .ok_or(JsValue::from_str("No objects found in model"))?
            .name
            .clone(),
    };

    let json_schema = to_json_schema(&model, &root, openai)
        .map_err(|e| JsValue::from_str(&format!("Error serializing schema: {}", e)))?;

    // Directly return the JSON schema object instead of converting it to a JsValue
    Ok(serde_json::to_string(&json_schema).unwrap())
}

/// Validates the given markdown content and returns the validation result as a `Validator`.
///
/// # Arguments
///
/// * `markdown_content` - A string slice that holds the markdown content to be validated.
///
/// # Returns
///
/// Either an empty `Validator` or an error `Validator`.
#[wasm_bindgen]
pub fn validate(markdown_content: &str) -> Validator {
    let model = DataModel::from_markdown_string(markdown_content);
    match model {
        Ok(_) => Validator::new(),
        Err(res) => res,
    }
}
