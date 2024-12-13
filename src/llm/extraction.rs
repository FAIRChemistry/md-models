use std::env;

use openai_api_rs::v1::{api::OpenAIClient, chat_completion};
use serde_json::{json, Value};

use crate::{datamodel::DataModel, json::export::to_json_schema};

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
    let response_format = prepare_response_format(data_model, root, multiple)?;
    let client = prepare_client(api_key)?;
    let messages = vec![create_chat_message(pre_prompt), create_chat_message(prompt)];
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

fn prepare_response_format(
    model: &DataModel,
    root: &str,
    multiple: bool,
) -> Result<Value, Box<dyn std::error::Error>> {
    let schema = to_json_schema(model, root, true)?;

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

fn prepare_client(api_key: Option<String>) -> Result<OpenAIClient, Box<dyn std::error::Error>> {
    let api_key = match api_key {
        Some(api_key) => api_key,
        None => env::var("OPENAI_API_KEY")?,
    };

    OpenAIClient::builder().with_api_key(api_key).build()
}

fn create_chat_message(content: &str) -> chat_completion::ChatCompletionMessage {
    chat_completion::ChatCompletionMessage {
        role: chat_completion::MessageRole::user,
        content: chat_completion::Content::Text(content.to_string()),
        name: None,
        tool_calls: None,
        tool_call_id: None,
    }
}
