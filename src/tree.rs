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
/// another object as its data type.
///
/// # Arguments
///
/// * `model` - The data model containing objects and their relationships
///
/// # Returns
///
/// A directed graph (DiGraph) where:
/// - Nodes are labeled with object names (String)
/// - Edges represent dependencies between objects (empty unit type)
pub fn dependency_graph(model: &DataModel) -> DiGraph<String, ()> {
    let mut graph = DiGraph::new();
    let adjacency_map = extract_adjacency_map(&model.objects);

    // Create nodes for each object
    let nodes: HashMap<String, NodeIndex> = model
        .objects
        .iter()
        .map(|o| (o.name.clone(), graph.add_node(o.name.clone())))
        .collect();

    // Add edges based on adjacency map
    for (object_name, dependencies) in adjacency_map {
        for dependency in dependencies {
            graph.add_edge(nodes[&dependency], nodes[&object_name], ());
        }
    }

    graph
}

/// Returns a topologically sorted list of node names from the dependency graph.
///
/// This function performs a depth-first traversal of the graph to generate a topological
/// ordering of the nodes. The ordering ensures that for any directed edge u->v in the graph,
/// node u comes before node v in the ordering.
///
/// # Arguments
///
/// * `graph` - The directed graph to traverse, with String node labels
///
/// # Returns
///
/// A Vec<String> containing the node labels in topological order
pub fn get_topological_order(graph: &DiGraph<String, ()>) -> Vec<String> {
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
/// marking visited nodes and building up the topologically sorted stack.
///
/// # Arguments
///
/// * `graph` - The directed graph being traversed
/// * `node` - The current node being visited
/// * `visited` - Set of already visited node labels
/// * `stack` - Vector storing the topologically sorted node labels
fn visit(
    graph: &DiGraph<String, ()>,
    node: NodeIndex,
    visited: &mut HashSet<String>,
    stack: &mut Vec<String>,
) {
    if visited.contains(&graph[node]) {
        return;
    }
    visited.insert(graph[node].clone());

    // Visit all dependencies first
    for neighbor in graph.neighbors_directed(node, Direction::Outgoing) {
        visit(graph, neighbor, visited, stack);
    }

    // Add to stack only after visiting all dependencies
    stack.push(graph[node].clone());
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
