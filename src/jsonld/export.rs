//! This module provides utilities for generating JSON-LD contexts from a `DataModel`
//! definition. It constructs nested JSON-LD contexts by traversing the data model's
//! object graph and encoding type, term, and container information suited
//! for consumption by JSON-LD tools.

use crate::{
    jsonld::schema::{
        JsonLdContext, JsonLdHeader, OneOrMany, SimpleContext, TermDef, TermDetail, TypeOrVec,
    },
    object::Object,
    prelude::DataModel,
    tree,
};
use petgraph::Direction;
use std::{
    collections::{HashMap, HashSet},
    error::Error,
};

/// Generate a full JSON-LD document header for the given data model.
///
/// This function constructs a [`JsonLdHeader`] for a [`DataModel`], using its prefix,
/// object name, and any configured context prefixes. It builds the `@context`, `@id`,
/// and `@type` fields required for compliant JSON-LD output. An explicit root object
/// can be chosen by its name; otherwise, the first object in the model is used.
///
/// # Arguments
/// * `model` - Reference to the source [`DataModel`].
/// * `root` - Optional name of the root object; if `None`, the model's first object is used.
///
/// # Returns
/// * `Ok(JsonLdHeader)` containing complete JSON-LD metadata for the root object.
/// * `Err(Box<dyn Error>)` if the root cannot be found or context creation fails.
///
/// # Example
/// ```rust,ignore
/// let header = to_json_ld(&model, Some("EnzymeMLDocument")).unwrap();
/// ```
pub fn to_json_ld(model: &DataModel, root: Option<&str>) -> Result<JsonLdHeader, Box<dyn Error>> {
    let context = create_context(model, root)?;
    let config = model.config.clone().unwrap_or_default();
    let object = match root {
        Some(name) => model
            .objects
            .iter()
            .find(|o| o.name == name)
            .ok_or_else(|| format!("Object {name} not found"))?,
        None => model.objects.first().ok_or("No objects found in model")?,
    };

    let object_type = match object.term.clone() {
        Some(term) => TypeOrVec::Single(term),
        None => TypeOrVec::Single(format!("{}:{}", config.prefix, object.name)),
    };

    Ok(JsonLdHeader {
        context: Some(JsonLdContext::Object(context)),
        type_: Some(object_type),
        ..Default::default()
    })
}

/// Generate a [`SimpleContext`] for use as the JSON-LD `@context` for the given data model.
///
/// # Arguments
///
/// * `model` - The data model from which to derive the context.
/// * `root` - Optional. The name of the root object. If not provided, the function uses the first object.
///
/// # Returns
///
/// * `Ok(SimpleContext)` containing the JSON-LD context terms.
/// * `Err(String)` if there is an error building the context.
///
/// # Example
///
/// ```rust,ignore
/// let context = create_context(&model, Some("EnzymeMLDocument")).unwrap();
/// ```
fn create_context(model: &DataModel, root: Option<&str>) -> Result<SimpleContext, String> {
    let mut context = SimpleContext::default();
    let config = model.config.clone().unwrap_or_default();
    let model_id = if !config.prefix.is_empty() {
        config.prefix
    } else if config.id.is_some() {
        config.id.unwrap()
    } else {
        "model".to_string()
    };

    // Add the model prefix
    context
        .terms
        .insert(model_id.clone(), TermDef::Simple(config.repo.clone()));

    // Add prefixes from DataModel config
    if let Some(prefixes) = &config.prefixes {
        for (prefix, uri) in prefixes {
            context
                .terms
                .insert(prefix.clone(), TermDef::Simple(uri.clone()));
        }
    }

    let mut context_cache: HashMap<String, SimpleContext> = HashMap::new();

    // Determine which root to use: explicit or first object
    let root_name = match root {
        Some(name) => name.to_string(),
        None => model
            .objects
            .first()
            .ok_or("No objects found in model")?
            .name
            .clone(),
    };

    // Build the graph based on the determined root
    let graph = tree::object_graph(model, &root_name)?;

    // Process only the root object's attributes at the top level
    // Nested contexts will be built automatically via build_nested_for_attr
    if let Some(root_idx) = graph
        .node_indices()
        .find(|&idx| graph[idx].name == root_name)
    {
        let obj_context = build_object_context(&graph, root_idx, &model_id, &mut context_cache);
        for (term_name, term_def) in obj_context.terms {
            context.terms.insert(term_name, term_def);
        }
    }
    Ok(context)
}

/// Build a [`SimpleContext`] for a given object within the graph.
///
/// Caches contexts per-object to allow sharing/nesting without recomputation.
///
/// # Arguments
/// - `graph`: The object type dependency graph.
/// - `node_idx`: The current object's graph node index.
/// - `model_id`: The base prefix/id for the model.
/// - `cache`: Cache of previously-built contexts.
fn build_object_context(
    graph: &petgraph::graph::DiGraph<Object, ()>,
    node_idx: petgraph::graph::NodeIndex,
    model_id: &str,
    cache: &mut HashMap<String, SimpleContext>,
) -> SimpleContext {
    let object = &graph[node_idx];

    if let Some(cached) = cache.get(&object.name) {
        return cached.clone();
    }

    let mut context = SimpleContext::default();
    let object_names: HashSet<String> = graph
        .node_indices()
        .map(|idx| graph[idx].name.clone())
        .collect();

    for attr in &object.attributes {
        let has_nested = attr.dtypes.iter().any(|dt| object_names.contains(dt));
        let term_def = if has_nested || attr.is_array {
            build_detailed_term_def(graph, node_idx, attr, model_id, has_nested, cache)
        } else {
            let term_id = get_attr_term_id(attr, object, model_id);
            TermDef::Simple(term_id)
        };

        context.terms.insert(attr.name.clone(), term_def);
    }

    cache.insert(object.name.clone(), context.clone());
    context
}

/// Build a [`TermDef`] in detailed form, for array or nested attributes.
fn build_detailed_term_def(
    graph: &petgraph::graph::DiGraph<Object, ()>,
    node_idx: petgraph::graph::NodeIndex,
    attr: &crate::attribute::Attribute,
    model_id: &str,
    has_nested: bool,
    cache: &mut HashMap<String, SimpleContext>,
) -> TermDef {
    // Determine type using the first dtype, falling back to parent_object if needed
    let object_type = attr
        .dtypes
        .first()
        .and_then(|dtype| find_sub_object(graph, dtype))
        .map(|_idx| "@id".to_string());

    // Build term detail, setting @container and nested @context as needed
    let mut detail = TermDetail {
        type_: object_type,
        container: if attr.is_array {
            Some(OneOrMany::One("@set".to_string()))
        } else {
            None
        },
        ..Default::default()
    };

    if has_nested {
        let nested = build_nested_for_attr(graph, node_idx, attr, model_id, cache);
        if !nested.terms.is_empty() {
            detail.context = Some(Box::new(JsonLdContext::Object(nested)));
        }
    }

    TermDef::Detailed(detail)
}

/// Find the node index for a child (sub-)object in the graph by name (data type).
///
/// Returns index if the dtype matches an object in the graph, or `None` if not found.
fn find_sub_object(
    graph: &petgraph::graph::DiGraph<Object, ()>,
    dtype: &str,
) -> Option<petgraph::graph::NodeIndex> {
    graph.node_indices().find(|&idx| graph[idx].name == dtype)
}

/// Get the JSON-LD term id/IRI for a concrete attribute, using an override if present.
///
/// If no explicit term is present, returns a default constructed IRI
/// of the form `"{model_id}:{object}/{attr}"`.
fn get_attr_term_id(attr: &crate::attribute::Attribute, object: &Object, model_id: &str) -> String {
    match &attr.term {
        Some(term) => term.clone(),
        None => format!("{}:{}/{}", model_id, object.name, attr.name),
    }
}

/// Retrieve a term for an attribute from its explicit term, or default to constructed IRI.
///
/// Accepts the explicit annotated term or synthesizes a term if missing.
fn get_object_term_or_default(term: &Option<String>, model_id: &str, object_name: &str) -> String {
    match term {
        Some(term) => term.clone(),
        None => format!("{}:{}", model_id, object_name),
    }
}

/// Recursively build the nested context for an attribute's referenced object(s).
///
/// Returns a [`SimpleContext`] capturing nested keys/objects for JSON-LD.
///
/// # Arguments
/// - `graph`: The object type dependency graph.
/// - `parent_idx`: The parent object's node index in the graph.
/// - `attr`: The attribute whose nested context is built.
/// - `model_id`: The base model prefix/IRI.
/// - `cache`: Reference to context cache to permit recursion.
fn build_nested_for_attr(
    graph: &petgraph::graph::DiGraph<Object, ()>,
    parent_idx: petgraph::graph::NodeIndex,
    attr: &crate::attribute::Attribute,
    model_id: &str,
    cache: &mut HashMap<String, SimpleContext>,
) -> SimpleContext {
    let mut context = SimpleContext::default();

    for neighbor_idx in graph.neighbors_directed(parent_idx, Direction::Outgoing) {
        let neighbor_obj = &graph[neighbor_idx];

        if attr.dtypes.contains(&neighbor_obj.name) {
            let nested_context = build_object_context(graph, neighbor_idx, model_id, cache);

            // We need to add the type as `@id` to tell JSON-LD that this attribute references a  IRI
            let term = get_object_term_or_default(&neighbor_obj.term, model_id, &neighbor_obj.name);
            context.type_ = Some(TypeOrVec::Single(term));

            for (key, value) in nested_context.terms {
                context.terms.insert(key, value);
            }
        }
    }

    context
}
