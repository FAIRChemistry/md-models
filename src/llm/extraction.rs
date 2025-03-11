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

use std::env;

use log::{error, info};
use openai_api_rs::v1::{api::OpenAIClient, chat_completion};
use serde_json::{json, Value};

use crate::{datamodel::DataModel, json::export::to_json_schema};

use super::patch::JSONPatch;

const UPDATE_SYSPROMPT: &str = include_str!("update_sysprompt.md");
const UPDATE_PROMPT: &str = include_str!("update_prompt.md");
const REFINE_PROMPT: &str = include_str!("refine_prompt.md");

/// Queries the OpenAI API with a given prompt and pre-prompt, using a specified data model and root.
///
/// # Arguments
///
/// * `prompt` - The main prompt to send to the OpenAI API.
/// * `pre_prompt` - An additional pre-prompt to provide context or setup for the main prompt.
/// * `data_model` - The data model used to generate the JSON schema for the response format.
/// * `root` - The root name for the JSON schema.
/// * `model` - The OpenAI model to use for the chat completion.
/// * `multiple` - Whether to extract multiple objects.
///
/// # Returns
///
/// A `Result` containing a `serde_json::Value` with the parsed JSON response from the OpenAI API, or an error if the operation fails.
pub async fn query_openai(
    prompt: &str,
    pre_prompt: &str,
    data_model: &DataModel,
    root: &str,
    model: &str,
    multiple: bool,
    api_key: Option<String>,
) -> Result<Value, Box<dyn std::error::Error>> {
    // Prepare the response format
    let schema = serde_json::to_value(to_json_schema(data_model, root, true)?)?;
    let response_format = prepare_response_format(&schema, root, multiple)?;
    let mut client = prepare_client(api_key)?;

    // Refine the prompt
    let refined_prompt = refine_query(prompt, model).await?;

    let messages = vec![
        create_chat_message(pre_prompt, chat_completion::MessageRole::system),
        create_chat_message(&refined_prompt, chat_completion::MessageRole::user),
    ];

    let req = chat_completion::ChatCompletionRequest::new(model.to_string(), messages)
        .response_format(response_format)
        .temperature(0.0);

    let result = client.chat_completion(req).await?;
    let content = result
        .choices
        .first()
        .and_then(|choice| choice.message.content.as_ref())
        .ok_or_else(|| format!("No content in response from {}", model))?;

    Ok(serde_json::from_str(content)?)
}

pub async fn patch_openai(
    prompt: &str,
    dataset: &Value,
    pre_prompt: Option<&str>,
    data_model: &DataModel,
    root: &str,
    model: &str,
    api_key: Option<String>,
) -> Result<Value, Box<dyn std::error::Error>> {
    // Copy the dataset
    let mut dataset = dataset.clone();

    // Prepare the response format
    let mut client = prepare_client(api_key)?;
    let response_format = prepare_response_format(&JSONPatch::schema(), root, false)?;

    // Refine the prompt and reduce the schema
    let mut messages = vec![create_chat_message(
        UPDATE_SYSPROMPT,
        chat_completion::MessageRole::system,
    )];

    if let Some(pre_prompt) = pre_prompt {
        messages.push(create_chat_message(
            pre_prompt,
            chat_completion::MessageRole::system,
        ));
    }

    let schema = serde_json::to_value(to_json_schema(data_model, root, true)?)?;

    // Refine the prompt
    let refined_prompt = refine_query(prompt, model).await?;
    let prompt = UPDATE_PROMPT
        .replace("{dataset}", &dataset.to_string())
        .replace("{prompt}", &refined_prompt)
        .replace("{schema}", &schema.to_string());

    messages.push(create_chat_message(
        &prompt,
        chat_completion::MessageRole::user,
    ));

    let req = chat_completion::ChatCompletionRequest::new(model.to_string(), messages)
        .response_format(response_format)
        .temperature(0.0);

    info!("Patching dataset");

    let result = client.chat_completion(req).await?;
    let content = result
        .choices
        .first()
        .and_then(|choice| choice.message.content.as_ref())
        .ok_or_else(|| format!("No content in response from {}", model))?;

    // Parse the content as a JSON patch
    let patch = serde_json::from_str::<JSONPatch>(content)?;

    // Apply the patch to the dataset
    let val_errors = patch.apply(&mut dataset, data_model, Some(root.to_string()))?;

    if !val_errors.is_empty() {
        log::error!("Validation errors: {}", val_errors.len());
        for error in val_errors {
            error!("{}:\n\t└── {}", error.instance_path, error.message);
        }
        return Err(format!("Response from {} is invalid", model).into());
    } else {
        log::info!("Patched dataset successfully");
        Ok(dataset)
    }
}

pub async fn refine_query(query: &str, model: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut client = prepare_client(None)?;

    let prompt = REFINE_PROMPT.replace("{query}", query);

    info!("Refining query");

    let messages = vec![create_chat_message(
        &prompt,
        chat_completion::MessageRole::user,
    )];

    let req =
        chat_completion::ChatCompletionRequest::new(model.to_string(), messages).temperature(0.0);

    let result = client.chat_completion(req).await?;
    let content = result
        .choices
        .first()
        .and_then(|choice| choice.message.content.as_ref())
        .ok_or_else(|| format!("No content in response from {}", model))?;

    info!("Refined query");

    Ok(content.to_string())
}

/// Prepares a response format for the OpenAI API.
///
/// # Arguments
///
/// * `model` - The data model to use for the response format.
/// * `root` - The root name for the JSON schema.
/// * `multiple` - Whether to prepare a response format for multiple objects.
///
/// # Returns
///
/// A `Result` containing a `serde_json::Value` with the prepared response format, or an error if the operation fails.
fn prepare_response_format(
    schema: &Value,
    root: &str,
    multiple: bool,
) -> Result<Value, Box<dyn std::error::Error>> {
    if multiple {
        Ok(json!(
            { "type": "json_schema",
              "json_schema": {
                "name": root,
                "schema": {
                    "type": "object",
                    "properties": {
                        "items": {
                            "type": "array",
                            "items": schema
                        }
                    }
                }
              }
            }
        ))
    } else {
        Ok(json!({ "type": "json_schema", "json_schema": { "name": root, "schema": schema } }))
    }
}

/// Prepares a client for the OpenAI API.
///
/// # Arguments
///
/// * `api_key` - The API key to use for the OpenAI API.
///
/// # Returns
///
/// A `Result` containing an `OpenAIClient`, or an error if the operation fails.
pub fn prepare_client(api_key: Option<String>) -> Result<OpenAIClient, Box<dyn std::error::Error>> {
    let api_key = match api_key {
        Some(api_key) => api_key,
        None => env::var("OPENAI_API_KEY")?,
    };

    OpenAIClient::builder().with_api_key(api_key).build()
}

/// Creates a chat message.
///
/// # Arguments
///
/// * `content` - The content of the message.
///
/// # Returns
///
/// A `chat_completion::ChatCompletionMessage` with the message.    
fn create_chat_message(
    content: &str,
    role: chat_completion::MessageRole,
) -> chat_completion::ChatCompletionMessage {
    chat_completion::ChatCompletionMessage {
        role,
        content: chat_completion::Content::Text(content.to_string()),
        name: None,
        tool_calls: None,
        tool_call_id: None,
    }
}
