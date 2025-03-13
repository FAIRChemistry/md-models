use openai_api_rs::v1::{
    api::OpenAIClient,
    embedding::{EmbeddingData, EmbeddingRequest, EncodingFormat},
    error::APIError,
};
use petgraph::graph::DiGraph;

use crate::{
    prelude::DataModel,
    tree::{model_tree, Node},
};

/// Model identifier for OpenAI's small text embedding model
pub const TEXT_EMBEDDING_3_SMALL: &str = "text-embedding-3-small";
/// Model identifier for OpenAI's large text embedding model
pub const TEXT_EMBEDDING_3_LARGE: &str = "text-embedding-3-large";

/// Embeds a data model by converting it to a tree and generating embeddings for each node's description.
///
/// # Arguments
///
/// * `client` - OpenAI API client used to generate embeddings
/// * `data_model` - The data model to embed
///
/// # Returns
///
/// A tuple containing:
/// * A HashMap mapping node names to their descriptions
/// * A vector of embedding data for each description
/// * A directed graph representing the model tree with nodes that can store f64 values
///
/// # Errors
///
/// Returns an APIError if the embedding request fails
pub async fn embed_model(
    client: &mut OpenAIClient,
    data_model: &DataModel,
) -> Result<
    (
        Vec<String>,
        Vec<String>,
        Vec<EmbeddingData>,
        DiGraph<Node<f64>, ()>,
    ),
    APIError,
> {
    // Convert the data model into a tree
    let tree = model_tree::<f64>(&data_model.objects[0], data_model);

    // Names
    let (names, descriptions): (Vec<String>, Vec<String>) = tree
        .raw_nodes()
        .iter()
        .map(|n| {
            (
                n.weight.name().to_string(),
                format!("{}: {}", n.weight.name(), n.weight.description()),
            )
        })
        .unzip();

    // Embed the descriptions
    let embeddings = embed_texts(&descriptions, client, TEXT_EMBEDDING_3_LARGE).await?;

    Ok((names, descriptions, embeddings, tree))
}

/// Generates embeddings for a collection of texts using OpenAI's embedding API.
///
/// # Arguments
///
/// * `texts` - HashMap mapping identifiers to text content to embed
/// * `client` - OpenAI API client used to generate embeddings
/// * `model` - Identifier of the embedding model to use
///
/// # Returns
///
/// A vector of embedding data for each input text
///
/// # Errors
///
/// Returns an APIError if the embedding request fails
pub async fn embed_texts(
    texts: &[String],
    client: &mut OpenAIClient,
    model: &str,
) -> Result<Vec<EmbeddingData>, APIError> {
    let request = EmbeddingRequest {
        model: model.to_string(),
        input: texts.to_vec(),
        dimensions: Some(800),
        encoding_format: Some(EncodingFormat::Float),
        user: None,
    };

    let response = client.embedding(request).await?;

    Ok(response.data)
}
