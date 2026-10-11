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
/// let header = to_json_ld(&model, Some("Document")).unwrap();
/// ```
pub fn to_json_ld(model: &DataModel, root: Option<&str>) -> Result<JsonLdHeader, Box<dyn Error>> {
    let context = create_context(model, root)?;
    let config = model.config.clone().unwrap_or_default();
    crate::exporters::warn_namespace_issues(&config, "json-ld");
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
/// let context = create_context(&model, Some("Document")).unwrap();
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
    let namespace = model_namespace(&config.repo);
    context
        .terms
        .insert(model_id.clone(), prefix_term(&namespace));

    // Fallback IRI for keys that are not part of the model
    context.vocab = Some(Some(namespace));

    // Add prefixes from DataModel config
    if let Some(prefixes) = &config.prefixes {
        for (prefix, uri) in prefixes {
            context.terms.insert(prefix.clone(), prefix_term(uri));
        }
    }

    let mut context_cache: HashMap<String, SimpleContext> = HashMap::new();
    let mut in_progress: HashSet<String> = HashSet::new();

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
        let obj_context = build_object_context(
            &graph,
            root_idx,
            &model_id,
            &mut context_cache,
            &mut in_progress,
        );
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
/// Recursive models are supported: `in_progress` holds the objects being built
/// further up the stack, and re-entering one returns an empty context. Omitting
/// the nested `@context` is valid, since scoped contexts are inherited.
///
/// # Arguments
/// - `graph`: The object type dependency graph.
/// - `node_idx`: The current object's graph node index.
/// - `model_id`: The base prefix/id for the model.
/// - `cache`: Memo of contexts already built.
/// - `in_progress`: Objects being built further up the stack, to break cycles.
fn build_object_context(
    graph: &petgraph::graph::DiGraph<Object, ()>,
    node_idx: petgraph::graph::NodeIndex,
    model_id: &str,
    cache: &mut HashMap<String, SimpleContext>,
    in_progress: &mut HashSet<String>,
) -> SimpleContext {
    let object = &graph[node_idx];

    if let Some(cached) = cache.get(&object.name) {
        return cached.clone();
    }

    // Cycle detected: this object is already being built further up the stack.
    if in_progress.contains(&object.name) {
        return SimpleContext::default();
    }

    in_progress.insert(object.name.clone());

    let mut context = SimpleContext::default();
    let object_names: HashSet<String> = graph
        .node_indices()
        .map(|idx| graph[idx].name.clone())
        .collect();

    for attr in &object.attributes {
        let has_nested = attr.dtypes.iter().any(|dt| object_names.contains(dt));
        let term_def = if has_nested || attr.is_array || attr.is_iri || is_date(attr) {
            build_detailed_term_def(
                graph,
                node_idx,
                attr,
                model_id,
                has_nested,
                cache,
                in_progress,
            )
        } else {
            let term_id = get_attr_term_id(attr, object, model_id);
            TermDef::Simple(term_id)
        };

        context.terms.insert(attr.name.clone(), term_def);
    }

    in_progress.remove(&object.name);
    cache.insert(object.name.clone(), context.clone());
    context
}

/// JSON-LD 1.1 only uses a simple term as a prefix if its IRI ends in a gen-delim.
pub(crate) fn ends_with_gen_delim(iri: &str) -> bool {
    iri.ends_with([':', '/', '?', '#', '[', ']', '@'])
}

/// The model namespace: `repo` itself if it ends in a gen-delim, else `repo#`,
/// so `prefix:Class/attr` never glues onto the repo (`.../repoClass/attr`).
pub(crate) fn model_namespace(repo: &str) -> String {
    if ends_with_gen_delim(repo) {
        repo.to_string()
    } else {
        format!("{repo}#")
    }
}

/// A prefix definition JSON-LD 1.1 accepts: a plain IRI, or `{"@id": iri, "@prefix": true}`
/// when the IRI does not end in a gen-delim (e.g. `https://example.org/item_`).
fn prefix_term(iri: &str) -> TermDef {
    if ends_with_gen_delim(iri) {
        TermDef::Simple(iri.to_string())
    } else {
        TermDef::Detailed(TermDetail {
            id: Some(iri.to_string()),
            prefix: Some(true),
            ..Default::default()
        })
    }
}

const XSD_DATE: &str = "http://www.w3.org/2001/XMLSchema#date";

fn is_date(attr: &crate::attribute::Attribute) -> bool {
    attr.dtypes == ["date"]
}

/// Build a [`TermDef`] in detailed form, for array or nested attributes.
fn build_detailed_term_def(
    graph: &petgraph::graph::DiGraph<Object, ()>,
    node_idx: petgraph::graph::NodeIndex,
    attr: &crate::attribute::Attribute,
    model_id: &str,
    has_nested: bool,
    cache: &mut HashMap<String, SimpleContext>,
    in_progress: &mut HashSet<String>,
) -> TermDef {
    // Determine type using the first dtype, falling back to parent_object if needed
    let object_type = attr
        .dtypes
        .first()
        .and_then(|dtype| find_sub_object(graph, dtype))
        .map(|_idx| "@id".to_string());

    // Build term detail, setting @container and nested @context as needed
    let mut detail = TermDetail {
        id: Some(get_attr_term_id(attr, &graph[node_idx], model_id)),
        type_: object_type
            .or_else(|| attr.is_iri.then(|| "@id".to_string()))
            .or_else(|| is_date(attr).then(|| XSD_DATE.to_string())),
        container: if attr.is_array {
            Some(OneOrMany::One("@list".to_string()))
        } else {
            None
        },
        ..Default::default()
    };

    if has_nested {
        let nested = build_nested_for_attr(graph, node_idx, attr, model_id, cache, in_progress);
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
    in_progress: &mut HashSet<String>,
) -> SimpleContext {
    let mut context = SimpleContext::default();

    for neighbor_idx in graph.neighbors_directed(parent_idx, Direction::Outgoing) {
        let neighbor_obj = &graph[neighbor_idx];

        if attr.dtypes.contains(&neighbor_obj.name) {
            // No `@type` here: JSON-LD 1.1 does not allow it inside a scoped context;
            // node types belong on the nodes themselves.
            let nested_context =
                build_object_context(graph, neighbor_idx, model_id, cache, in_progress);

            for (key, value) in nested_context.terms {
                context.terms.insert(key, value);
            }
        }
    }

    context
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::markdown::parser::parse_markdown;
    use pretty_assertions::assert_eq;
    use serde_json::{json, Value};

    fn context() -> Value {
        let content = std::fs::read_to_string("tests/data/model_jsonld.md").unwrap();
        let model = parse_markdown(&content, None).expect("Could not parse markdown");
        let header = to_json_ld(&model, Some("Document")).expect("Could not build JSON-LD");
        serde_json::to_value(header).unwrap()["@context"].clone()
    }

    #[test]
    fn test_date_typed_only_for_pure_date_fields() {
        let content = "---\nrepo: \"https://example.org/\"\nprefix: ex\n---\n\n### Doc\n\n- both\n  - Type: date, string\n- only\n  - Type: date\n- many\n  - Type: date[]\n";
        let model = parse_markdown(content, None).expect("Could not parse markdown");
        let header = to_json_ld(&model, Some("Doc")).expect("Could not build JSON-LD");
        let ctx = serde_json::to_value(header).unwrap()["@context"].clone();
        assert_eq!(ctx["only"]["@type"], json!(XSD_DATE));
        assert_eq!(ctx["many"]["@type"], json!(XSD_DATE));
        assert_eq!(ctx["both"], json!("ex:Doc/both"));
    }

    #[test]
    fn test_ends_with_gen_delim() {
        assert!(ends_with_gen_delim("https://schema.org/"));
        assert!(ends_with_gen_delim("http://www.w3.org/2000/01/rdf-schema#"));
        assert!(!ends_with_gen_delim("https://example.org/category_"));
    }

    #[test]
    fn test_model_namespace() {
        assert_eq!(
            model_namespace("https://example.org/"),
            "https://example.org/"
        );
        assert_eq!(
            model_namespace("https://example.org/x#"),
            "https://example.org/x#"
        );
        assert_eq!(
            model_namespace("https://example.org/repo"),
            "https://example.org/repo#"
        );
    }

    #[test]
    fn test_slashless_repo_gets_hash_namespace() {
        let content = "---\nrepo: \"https://example.org/repo\"\nprefix: ex\n---\n\n### Doc\n\n- id\n  - Type: string\n";
        let model = parse_markdown(content, None).expect("Could not parse markdown");
        let header = to_json_ld(&model, Some("Doc")).expect("Could not build JSON-LD");
        let ctx = serde_json::to_value(header).unwrap()["@context"].clone();
        assert_eq!(ctx["@vocab"], json!("https://example.org/repo#"));
        assert_eq!(ctx["ex"], json!("https://example.org/repo#"));
    }

    #[test]
    fn test_prefixes_without_gen_delim_opt_in() {
        let ctx = context();
        assert_eq!(
            ctx["cat"],
            json!({"@id": "https://example.org/category_", "@prefix": true})
        );
        assert_eq!(ctx["rdfs"], json!("http://www.w3.org/2000/01/rdf-schema#"));
        assert_eq!(ctx["ex"], json!("https://example.org/jsonld/"));
    }

    #[test]
    fn test_every_attribute_has_an_iri() {
        let ctx = context();
        assert_eq!(ctx["@vocab"], json!("https://example.org/jsonld/"));
        assert_eq!(ctx["version"], json!("ex:Document/version"));
        assert_eq!(ctx["items"]["@id"], json!("ex:Document/items"));
        assert_eq!(
            ctx["items"]["@context"]["references"]["@id"],
            json!("schema:citation")
        );
    }

    #[test]
    fn test_identifiers_are_literals_and_iris_are_coerced() {
        let ctx = context();
        assert_eq!(ctx["id"], json!("schema:identifier"));
        assert_eq!(
            ctx["items"]["@context"]["owner_id"],
            json!("ex:Item/owner_id")
        );
        assert_eq!(
            ctx["homepage"],
            json!({"@id": "schema:url", "@type": "@id"})
        );
        assert_eq!(
            ctx["items"]["@context"]["references"]["@type"],
            json!("@id")
        );
    }

    #[test]
    fn test_scoped_contexts_have_no_type_entry() {
        assert_eq!(context()["items"]["@context"].get("@type"), None);
    }

    #[test]
    fn test_arrays_are_lists_and_dates_are_typed() {
        let ctx = context();
        assert_eq!(ctx["items"]["@container"], json!("@list"));
        let item = &ctx["items"]["@context"];
        assert_eq!(item["references"]["@container"], json!("@list"));
        assert_eq!(item["values"]["@container"], json!("@list"));
        assert_eq!(
            ctx["created"],
            json!({"@id": "ex:Document/created", "@type": "http://www.w3.org/2001/XMLSchema#date"})
        );
    }

    use std::path::PathBuf;

    /// Builds the JSON-LD header for a fixture and returns it as JSON.
    fn build_context(path: &str, root: Option<&str>) -> serde_json::Value {
        let model = DataModel::from_markdown(&PathBuf::from(path)).expect("Could not parse model");
        serde_json::to_value(
            model
                .json_ld_header(root)
                .expect("Could not build JSON-LD header"),
        )
        .expect("Could not serialise header")
    }

    /// Regression anchor for the acyclic case, which must not change.
    //
    // Recursion
    //
    // `cache` is populated only after the recursive descent, so it could never
    // terminate a cycle; a self-referential object overflowed the stack. The
    // cycle is now broken by omitting the nested `@context`, which JSON-LD
    // resolves through the enclosing context.
    //

    #[test]
    fn test_self_reference_terminates() {
        let context = build_context("tests/data/model_recursive.md", Some("Experiment"));
        let terms = &context["@context"];

        assert_eq!(terms["parent"]["@type"], "@id");
        assert!(
            terms["parent"].get("@context").is_none(),
            "the recursive branch must be cut: {terms:#?}"
        );
    }

    /// Cutting the cycle must not drop the object's other terms.
    #[test]
    fn test_self_reference_keeps_sibling_terms() {
        let context = build_context("tests/data/model_recursive.md", Some("Experiment"));
        let terms = &context["@context"];

        assert_eq!(terms["name"], "schema:name");
        assert!(terms.get("conditions").is_some());
    }

    /// A recursive array keeps its `@container`, which is set before the
    /// cycle check.
    #[test]
    fn test_self_referencing_array_keeps_container() {
        let context = build_context("tests/data/model_recursive.md", Some("Experiment"));
        let children = &context["@context"]["children"];

        assert_eq!(children["@container"], "@list");
        assert_eq!(children["@type"], "@id");
        assert!(children.get("@context").is_none());
    }

    /// Experiment -> Condition -> Experiment must terminate as well.
    #[test]
    fn test_mutual_recursion_terminates() {
        let context = build_context("tests/data/model_recursive.md", Some("Condition"));
        let terms = &context["@context"];

        assert_eq!(context["@type"], "tst:Condition");
        assert!(terms.get("experiment").is_some());
    }

    /// The root may sit anywhere in the cycle.
    #[test]
    fn test_recursion_from_every_root_terminates() {
        for root in ["Experiment", "Condition"] {
            let context = build_context("tests/data/model_recursive.md", Some(root));
            assert!(
                context.get("@context").is_some(),
                "root {root} produced no context"
            );
        }
    }

    /// A diamond is not a cycle: an object reachable by two paths must keep its
    /// nested context on both. This is what an over-eager "already visited"
    /// guard would break.
    #[test]
    fn test_diamond_is_not_treated_as_a_cycle() {
        let context = build_context("tests/data/model_diamond.md", Some("A"));
        let terms = &context["@context"];

        for branch in ["b", "c"] {
            let nested = &terms[branch]["@context"]["d"]["@context"];
            assert!(
                nested.get("name").is_some(),
                "D's context must survive under branch {branch}: {terms:#?}"
            );
        }
    }
}
