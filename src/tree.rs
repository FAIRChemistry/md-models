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

use std::collections::{HashMap, HashSet};

use crate::{datamodel::DataModel, object::Object};
use petgraph::{
    graph::{DiGraph, NodeIndex},
    Direction,
};

/// Creates a directed graph representing dependencies between objects in a data model.
///
/// This function builds a directed graph where nodes represent objects and edges represent
/// dependencies between them. A dependency exists when one object's attributes reference
/// another object as its data type. This function includes all objects in the model.
///
/// # Arguments
///
/// * `model` - The data model containing objects and their relationships
///
/// # Returns
///
/// A directed graph (DiGraph) where:
/// - Nodes contain the full Object structs
/// - Edges represent dependencies between objects (empty unit type)
/// - All objects in the model are included in the graph
pub fn dependency_graph(model: &DataModel) -> DiGraph<Object, ()> {
    let mut graph = DiGraph::new();
    let adjacency_map = extract_adjacency_map(&model.objects);

    // Create nodes for each object
    let nodes: HashMap<String, NodeIndex> = model
        .objects
        .iter()
        .map(|o| (o.name.clone(), graph.add_node(o.clone())))
        .collect();

    // Add edges based on adjacency map
    for (object_name, dependencies) in adjacency_map {
        for dependency in dependencies {
            graph.add_edge(nodes[&dependency], nodes[&object_name], ());
        }
    }

    graph
}

/// Creates a directed graph representing dependencies between objects in a data model.
///
/// This function builds a directed graph where nodes represent objects and edges represent
/// dependencies between them. A dependency exists when one object's attributes reference
/// another object as its data type. The graph is built recursively starting from the root
/// object and includes only objects reachable from the root.
///
/// # Arguments
///
/// * `model` - The data model containing objects and their relationships
/// * `root` - The name of the root object to start building the graph from
///
/// # Returns
///
/// A directed graph (DiGraph) where:
/// - Nodes contain the full Object structs
/// - Edges represent dependencies between objects (empty unit type)
/// - Only objects reachable from the root are included in the graph
///
/// # Errors
///
/// Returns an error if the root object is not found in the model or if there are
/// issues traversing the object dependencies.
pub fn object_graph(model: &DataModel, root: &str) -> Result<DiGraph<Object, ()>, String> {
    let mut graph = DiGraph::new();
    let adjacency_map = extract_adjacency_map(&model.objects);

    // Create nodes for each object
    let nodes: HashMap<String, NodeIndex> = model
        .objects
        .iter()
        .map(|o| (o.name.clone(), graph.add_node(o.clone())))
        .collect();

    // Verify root exists
    if !nodes.contains_key(root) {
        return Err(format!("Object '{}' not found in model", root));
    }

    // Track visited nodes to avoid infinite recursion
    let mut visited = HashSet::new();

    // Build the graph recursively starting from root
    object_graph_helper(&nodes, &adjacency_map, &mut graph, root, &mut visited)?;

    Ok(graph)
}

/// Recursive helper function to build the object dependency graph.
///
/// This function traverses the object dependency tree starting from the given root,
/// adding edges to the graph for each dependency relationship. It uses a visited set
/// to prevent infinite recursion in case of circular dependencies.
///
/// # Arguments
///
/// * `nodes` - Mapping of object names to their node indices in the graph
/// * `adjacency_map` - Map of object names to their list of dependencies
/// * `graph` - The graph being built (modified in place)
/// * `root` - The current object being processed
/// * `visited` - Set of object names that have already been processed
fn object_graph_helper(
    nodes: &HashMap<String, NodeIndex>,
    adjacency_map: &HashMap<String, Vec<String>>,
    graph: &mut DiGraph<Object, ()>,
    root: &str,
    visited: &mut HashSet<String>,
) -> Result<(), String> {
    // If we've already processed this node, return early
    if visited.contains(root) {
        return Ok(());
    }

    // Mark this node as visited
    visited.insert(root.to_string());

    // Get the dependencies for this object
    let dependencies = adjacency_map.get(root).map(|v| v.as_slice()).unwrap_or(&[]);

    // Process each dependency
    for dependency in dependencies {
        // Verify the dependency exists
        if !nodes.contains_key(dependency) {
            return Err(format!(
                "Dependency '{}' referenced by '{}' not found in model",
                dependency, root
            ));
        }

        // Recursively process the dependency first
        object_graph_helper(nodes, adjacency_map, graph, dependency, visited)?;

        // Add edge from root to dependency
        graph.add_edge(nodes[root], nodes[dependency], ());
    }

    Ok(())
}

/// Returns a topologically sorted list of node names from the dependency graph.
///
/// This function performs a depth-first traversal of the graph to generate a topological
/// ordering of the nodes. The ordering ensures that for any directed edge u->v in the graph,
/// node u comes before node v in the ordering.
///
/// # Arguments
///
/// * `graph` - The directed graph to traverse, with Object node values
///
/// # Returns
///
/// A Vec<String> containing the object names in topological order
pub fn get_topological_order(graph: &DiGraph<Object, ()>) -> Vec<String> {
    let mut visited = HashSet::new();
    let mut stack = Vec::new();

    for node in graph.node_indices() {
        visit(graph, node, &mut visited, &mut stack);
    }

    stack
}

/// Helper function for depth-first post-order traversal during topological sorting.
///
/// This function performs a recursive depth-first traversal starting from the given node,
/// marking visited nodes and building up the topologically sorted stack. It tracks visited
/// nodes by their names to ensure each object is processed only once.
///
/// # Arguments
///
/// * `graph` - The directed graph being traversed
/// * `node` - The current node being visited
/// * `visited` - Set of already visited object names
/// * `stack` - Vector storing the topologically sorted object names
fn visit(
    graph: &DiGraph<Object, ()>,
    node: NodeIndex,
    visited: &mut HashSet<String>,
    stack: &mut Vec<String>,
) {
    let object_name = &graph[node].name;

    if visited.contains(object_name) {
        return;
    }
    visited.insert(object_name.clone());

    // Visit all dependencies first
    for neighbor in graph.neighbors_directed(node, Direction::Outgoing) {
        visit(graph, neighbor, visited, stack);
    }

    // Add to stack only after visiting all dependencies
    stack.push(object_name.clone());
}

/// Extracts an adjacency map from a list of objects, representing their type dependencies.
///
/// This function analyzes the data types of attributes in each object and creates a mapping
/// of attribute names to their dependent object types. It only includes attributes that have
/// data types referencing other objects in the model.
///
/// # Arguments
///
/// * `objects` - A slice of Object structs to analyze
///
/// # Returns
///
/// A HashMap where:
/// - Keys are attribute names (String)
/// - Values are vectors of object names (Vec<String>) that the attribute depends on
fn extract_adjacency_map(objects: &[Object]) -> HashMap<String, Vec<String>> {
    // Create a HashSet of object names for faster lookups
    let object_names: HashSet<_> = objects.iter().map(|o| &o.name).collect();

    // Pre-allocate with estimated capacity
    let mut dtype_map = HashMap::with_capacity(objects.len());

    for object in objects {
        let mut deps = Vec::new();
        for attribute in &object.attributes {
            deps.extend(
                attribute
                    .dtypes
                    .iter()
                    .filter(|d| object_names.contains(d))
                    .cloned(),
            );
        }

        dtype_map.insert(object.name.clone(), deps);
    }
    dtype_map
}
