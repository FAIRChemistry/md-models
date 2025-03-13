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

use derive_builder::Builder;
use log::{error, info};
use openai_api_rs::v1::{api::OpenAIClient, chat_completion};
use serde_json::{json, Value};

use crate::{datamodel::DataModel, json::export::to_json_schema};

use super::patch::JSONPatch;

const UPDATE_SYSPROMPT: &str = include_str!("update_sysprompt.md");
const UPDATE_PROMPT: &str = include_str!("update_prompt.md");
const REFINE_PROMPT: &str = include_str!("refine_prompt.md");
const DEFAULT_PRE_PROMPT: &str = "You are a helpful assistant that extracts data from text input.";

#[derive(Debug, Clone, Builder)]
pub struct QueryArgs {
    pub prompt: String,
    pub pre_prompt: Option<String>,
    pub data_model: DataModel,
    pub root: String,
    pub model: String,
    pub multiple: bool,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub dataset: Option<Value>,
}

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
    query: impl Into<QueryArgs>,
) -> Result<Value, Box<dyn std::error::Error>> {
    let query = query.into();

    // Prepare the response format
    let schema = serde_json::to_value(to_json_schema(&query.data_model, &query.root, true)?)?;
    let response_format = prepare_response_format(&schema, &query.root, query.multiple)?;
    let mut client = prepare_client(query.api_key, &query.base_url)?;

    let messages = vec![
        create_chat_message(
            &query.pre_prompt.unwrap_or(DEFAULT_PRE_PROMPT.to_string()),
            chat_completion::MessageRole::system,
        ),
        create_chat_message(&query.prompt, chat_completion::MessageRole::user),
    ];

    let req = chat_completion::ChatCompletionRequest::new(query.model.to_string(), messages)
        .response_format(response_format)
        .temperature(0.0);

    let result = client.chat_completion(req).await?;
    let content = result
        .choices
        .first()
        .and_then(|choice| choice.message.content.as_ref())
        .ok_or_else(|| format!("No content in response from {}", query.model))?;

    Ok(serde_json::from_str(content)?)
}

pub async fn patch_openai(
    query: impl Into<QueryArgs>,
) -> Result<Value, Box<dyn std::error::Error>> {
    let query = query.into();

    let mut dataset = query
        .dataset
        .ok_or("Dataset is required for patch operation")?
        .clone();

    // Prepare the response format
    let mut client = prepare_client(query.api_key, &query.base_url)?;
    let response_format = prepare_response_format(&JSONPatch::schema(), &query.root, false)?;

    // Refine the prompt and reduce the schema
    let mut messages = vec![create_chat_message(
        UPDATE_SYSPROMPT,
        chat_completion::MessageRole::system,
    )];

    if let Some(pre_prompt) = query.pre_prompt {
        messages.push(create_chat_message(
            &pre_prompt,
            chat_completion::MessageRole::system,
        ));
    }

    let schema = serde_json::to_value(to_json_schema(&query.data_model, &query.root, true)?)?;

    // Refine the prompt
    let refined_prompt = refine_query(&query.prompt, &query.model, &query.base_url).await?;
    let prompt = UPDATE_PROMPT
        .replace("{dataset}", &dataset.to_string())
        .replace("{prompt}", &refined_prompt)
        .replace("{schema}", &schema.to_string());

    messages.push(create_chat_message(
        &prompt,
        chat_completion::MessageRole::user,
    ));

    let req = chat_completion::ChatCompletionRequest::new(query.model.to_string(), messages)
        .response_format(response_format)
        .temperature(0.0);

    let result = client.chat_completion(req).await?;
    let content = result
        .choices
        .first()
        .and_then(|choice| choice.message.content.as_ref())
        .ok_or_else(|| format!("No content in response from {}", query.model))?;

    // Parse the content as a JSON patch
    let patch = serde_json::from_str::<JSONPatch>(content)?;

    // Apply the patch to the dataset
    let val_errors = patch.apply(
        &mut dataset,
        &query.data_model,
        Some(query.root.to_string()),
    )?;

    if !val_errors.is_empty() {
        log::error!("Validation errors: {}", val_errors.len());
        for error in val_errors {
            error!("{}:\n\t└── {}", error.instance_path, error.message);
        }
        Err(format!("Response from {} is invalid", query.model).into())
    } else {
        log::info!("Patched dataset successfully");
        Ok(dataset)
    }
}

pub async fn refine_query(
    query: &str,
    model: &str,
    base_url: &Option<String>,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut client = prepare_client(None, base_url)?;

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
pub fn prepare_client(
    api_key: Option<String>,
    base_url: &Option<String>,
) -> Result<OpenAIClient, Box<dyn std::error::Error>> {
    let api_key = match api_key {
        Some(api_key) => api_key,
        None => env::var("OPENAI_API_KEY")?,
    };

    if let Some(base_url) = base_url {
        OpenAIClient::builder()
            .with_api_key(api_key)
            .with_endpoint(base_url)
            .build()
    } else {
        OpenAIClient::builder().with_api_key(api_key).build()
    }
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
