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
use std::hash::{DefaultHasher, Hash, Hasher};

use crate::{attribute::Attribute, datamodel::DataModel, object::Object};
use petgraph::visit::EdgeRef;
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

/// A node in the data model tree that can represent either an Object or an Attribute.
///
/// Each node can optionally store associated data of type T.
///
/// # Type Parameters
///
/// * `T` - The type of data that can be stored in the node
#[derive(Debug, Clone)]
pub enum Node<T> {
    /// An object node containing the object definition and optional data
    Object {
        /// The object definition
        object: Object,
        /// Optional associated data
        data: Option<T>,
    },
    /// An attribute node containing the attribute definition and optional data
    Attribute {
        /// The attribute definition
        attribute: Attribute,
        /// Optional associated data
        data: Option<T>,
    },
}

impl<T> Hash for Node<T> {
    // Hash the name and description of the node
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name().hash(state);
        self.description().hash(state);
    }
}

impl<T> Node<T> {
    /// Returns the name of the node, whether it's an Object or Attribute.
    ///
    /// # Returns
    ///
    /// A string slice containing the name of the object or attribute
    pub fn name(&self) -> &str {
        match self {
            Node::Object { object, .. } => &object.name,
            Node::Attribute { attribute, .. } => &attribute.name,
        }
    }

    /// Returns the description (docstring) of the node.
    ///
    /// # Returns
    ///
    /// A string slice containing the docstring of the object or attribute
    pub fn description(&self) -> &str {
        match self {
            Node::Object { object, .. } => &object.docstring,
            Node::Attribute { attribute, .. } => &attribute.docstring,
        }
    }

    /// Returns the data stored in the node.
    ///
    /// # Returns
    ///
    /// An Option containing a reference to the stored data, or None if no data is stored
    pub fn data(&self) -> &Option<T> {
        match self {
            Node::Object { data, .. } => data,
            Node::Attribute { data, .. } => data,
        }
    }

    /// Sets the data stored in the node.
    ///
    /// # Arguments
    ///
    /// * `data` - The data to store in the node
    pub fn set_data(&mut self, data: T) {
        match self {
            Node::Object { data: d, .. } => *d = Some(data),
            Node::Attribute { data: d, .. } => *d = Some(data),
        }
    }
}

/// Builds a tree representation of a data model starting from a root object.
///
/// This function creates a directed graph where nodes represent objects and attributes,
/// and edges represent relationships between them. Objects that appear in multiple branches
/// are duplicated to ensure a true tree structure without cross-connections.
///
/// # Arguments
///
/// * `root_object` - The starting object to build the tree from
/// * `model` - The data model containing all objects
///
/// # Returns
///
/// A directed graph (DiGraph) where nodes are either Object or Attribute variants
pub fn model_tree<T>(root_object: &Object, model: &DataModel) -> DiGraph<Node<T>, ()> {
    let mut graph = DiGraph::<Node<T>, ()>::new();
    let mut queue = Vec::new();

    // Initialize the tree with the root object
    initialize_tree(&mut graph, &mut queue, root_object);

    // Build the tree by processing the queue
    process_queue(&mut graph, &mut queue, model);

    graph
}

/// Hashes a directed graph.
///
/// This function hashes a directed graph by first sorting the nodes and edges by their name and target/source indices, respectively.
/// It then hashes each node and edge in turn, using their respective hash functions.
///
/// # Arguments
///
/// * `graph` - The directed graph to hash
///
/// # Returns
///
/// A u64 hash value
#[allow(dead_code)]
pub(crate) fn hash_graph<T>(graph: &DiGraph<Node<T>, ()>) -> u64 {
    let mut hasher = DefaultHasher::new();

    let mut nodes = graph.node_weights().collect::<Vec<_>>();
    nodes.sort_by_key(|n| n.name());

    for node in nodes {
        node.hash(&mut hasher);
    }

    let mut edges = graph.edge_references().collect::<Vec<_>>();
    edges.sort_by_key(|e| (e.source().index(), e.target().index()));

    for edge in edges {
        {
            edge.target().hash(&mut hasher);
        }
        {
            edge.source().hash(&mut hasher);
        }
    }

    hasher.finish()
}

/// Maps over a tree structure and collects nodes that match a given predicate into a HashMap.
///
/// This function traverses all nodes in the directed graph and applies a predicate function to each node.
/// For nodes where the predicate returns true, it stores them in a HashMap keyed by the node's name.
///
/// # Arguments
///
/// * `graph` - A mutable reference to the directed graph containing the tree structure
/// * `predicate` - A closure that takes a mutable reference to a Node<T> and returns a boolean.
///                 This predicate determines which nodes should be included in the resulting map.
///
/// # Type Parameters
///
/// * `T` - The type parameter for the Node enum, must implement Clone
///
/// # Details
///
/// The function works by:
/// 1. Creating an empty HashMap to store matching nodes
/// 2. Collecting all node indices from the graph into a vector to avoid borrowing issues
/// 3. Iterating through each node index
/// 4. For each node:
///    - Getting a mutable reference to the node
///    - Applying the predicate
///    - If the predicate returns true, cloning the node and storing it in the map
///      with the node's name as the key
///
/// # Note
///
/// - Mutable access to the graph is required to allow the predicate to potentially modify nodes
pub fn tree_map<T>(graph: &mut DiGraph<Node<T>, ()>, predicate: impl Fn(&mut Node<T>)) {
    let indices: Vec<_> = graph.node_indices().collect();

    for node_idx in indices {
        if let Some(node) = graph.node_weight_mut(node_idx) {
            predicate(node);
        }
    }
}

/// Initializes the tree with the root object.
///
/// Creates the initial node for the root object and adds it to the processing queue.
///
/// # Arguments
///
/// * `graph` - The graph being built
/// * `queue` - Queue for breadth-first processing of objects
/// * `root_object` - The root object to start building the tree from
fn initialize_tree<T>(
    graph: &mut DiGraph<Node<T>, ()>,
    queue: &mut Vec<(Object, NodeIndex, Option<NodeIndex>)>,
    root_object: &Object,
) {
    let root_node = graph.add_node(Node::Object {
        object: root_object.clone(),
        data: None,
    });
    queue.push((root_object.clone(), root_node, None));
}

/// Processes the queue to build the complete tree.
///
/// Iteratively processes each object in the queue, connecting it to its parent attribute
/// if one exists, and processing all of its attributes to continue building the tree.
///
/// # Arguments
///
/// * `graph` - The graph being built
/// * `queue` - Queue containing objects to process
/// * `model` - The data model containing all objects
fn process_queue<T>(
    graph: &mut DiGraph<Node<T>, ()>,
    queue: &mut Vec<(Object, NodeIndex, Option<NodeIndex>)>,
    model: &DataModel,
) {
    while let Some((object, object_node, parent_attr_node)) = queue.pop() {
        // Connect to parent attribute if it exists
        connect_to_parent(graph, object_node, parent_attr_node);

        // Process all attributes of the current object
        process_object_attributes(graph, queue, &object, object_node, model);
    }
}

/// Connects an object node to its parent attribute node if it exists.
///
/// # Arguments
///
/// * `graph` - The graph being built
/// * `object_node` - The node index of the object to connect
/// * `parent_attr_node` - Optional node index of the parent attribute
fn connect_to_parent<T>(
    graph: &mut DiGraph<Node<T>, ()>,
    object_node: NodeIndex,
    parent_attr_node: Option<NodeIndex>,
) {
    if let Some(parent_node) = parent_attr_node {
        graph.add_edge(parent_node, object_node, ());
    }
}

/// Processes all attributes of an object.
///
/// Creates nodes for each attribute and processes their dependencies.
///
/// # Arguments
///
/// * `graph` - The graph being built
/// * `queue` - Queue for processing object dependencies
/// * `object` - The object whose attributes are being processed
/// * `object_node` - Node index of the object being processed
/// * `model` - The data model containing all objects
fn process_object_attributes<T>(
    graph: &mut DiGraph<Node<T>, ()>,
    queue: &mut Vec<(Object, NodeIndex, Option<NodeIndex>)>,
    object: &Object,
    object_node: NodeIndex,
    model: &DataModel,
) {
    for attribute in &object.attributes {
        let attr_node = add_attribute_node(graph, attribute, object_node);
        process_attribute_dependencies(graph, queue, attribute, attr_node, model);
    }
}

/// Adds an attribute node to the graph and connects it to its parent object.
///
/// # Arguments
///
/// * `graph` - The graph being built
/// * `attribute` - The attribute to add as a node
/// * `parent_object_node` - Node index of the parent object
///
/// # Returns
///
/// The node index of the newly added attribute node
fn add_attribute_node<T>(
    graph: &mut DiGraph<Node<T>, ()>,
    attribute: &Attribute,
    parent_object_node: NodeIndex,
) -> NodeIndex {
    let attr_node = graph.add_node(Node::Attribute {
        attribute: attribute.clone(),
        data: None,
    });
    graph.add_edge(parent_object_node, attr_node, ());
    attr_node
}

/// Processes dependencies of an attribute.
///
/// For each data type referenced by the attribute that corresponds to an object,
/// adds that object to the graph and queue for processing.
///
/// # Arguments
///
/// * `graph` - The graph being built
/// * `queue` - Queue for processing object dependencies
/// * `attribute` - The attribute whose dependencies are being processed
/// * `attr_node` - Node index of the attribute being processed
/// * `model` - The data model containing all objects
fn process_attribute_dependencies<T>(
    graph: &mut DiGraph<Node<T>, ()>,
    queue: &mut Vec<(Object, NodeIndex, Option<NodeIndex>)>,
    attribute: &Attribute,
    attr_node: NodeIndex,
    model: &DataModel,
) {
    for dtype in &attribute.dtypes {
        if let Some(referenced_obj) = extract_object(dtype, model) {
            add_referenced_object(graph, queue, referenced_obj, attr_node);
        }
    }
}

/// Adds a referenced object to the graph and queue.
///
/// Creates a new node for the referenced object and connects it to the attribute node.
/// The object is also added to the queue for further processing.
///
/// # Arguments
///
/// * `graph` - The graph being built
/// * `queue` - Queue for processing object dependencies
/// * `referenced_obj` - The object being referenced
/// * `attr_node` - Node index of the attribute referencing the object
fn add_referenced_object<T>(
    graph: &mut DiGraph<Node<T>, ()>,
    queue: &mut Vec<(Object, NodeIndex, Option<NodeIndex>)>,
    referenced_obj: &Object,
    attr_node: NodeIndex,
) {
    // Always create a new node for the referenced object to ensure tree structure
    let new_obj_node = graph.add_node(Node::Object {
        object: referenced_obj.clone(),
        data: None,
    });

    // Connect attribute to the new object node
    graph.add_edge(attr_node, new_obj_node, ());

    // Add to queue for further processing
    queue.push((referenced_obj.clone(), new_obj_node, None));
}

/// Extracts an object from the data model by name.
///
/// # Arguments
///
/// * `name` - Name of the object to find
/// * `model` - The data model to search in
///
/// # Returns
///
/// An Option containing a reference to the found object, or None if not found
fn extract_object<'a>(name: &str, model: &'a DataModel) -> Option<&'a Object> {
    model.objects.iter().find(|o| o.name == name)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use petgraph::visit::EdgeRef;

    use super::*;
    use crate::datamodel::DataModel;

    fn load_model() -> DataModel {
        let path = Path::new("tests/data/model.md");
        let model = DataModel::from_markdown(path).expect("Failed to load model");
        model
    }

    #[test]
    fn test_dependency_graph() {
        let model = load_model();
        let graph = dependency_graph(&model);
        let order = get_topological_order(&graph);
        println!("{:?}", order);

        assert_eq!(order.len(), model.objects.len());
        assert_eq!(order[0], "Test");
        assert_eq!(order[1], "Test2");
    }

    #[test]
    fn test_model_tree() {
        let model = load_model();
        let tree = model_tree::<()>(&model.objects[0], &model);

        assert_eq!(tree.node_count(), 8, "Expected 8 nodes");
        assert_eq!(tree.edge_count(), 7, "Expected 7 edges");

        // Expected tree structure:
        // Test
        //  ├── name
        //  ├── number
        //  ├── test2
        //  │   ├── names
        //  │   └── number
        //  └── ontology

        let expected_edges = vec![
            ("Test", "name"),     // Test -> name
            ("Test", "number"),   // Test -> number
            ("Test", "test2"),    // Test -> test2
            ("test2", "Test2"),   // Test -> ontology
            ("Test2", "names"),   // test2 -> names
            ("Test2", "number"),  // test2 -> number
            ("Test", "ontology"), // Test -> ontology
        ];

        for edge in tree.edge_references() {
            // Get node names from indices
            let source = tree.node_weight(edge.source()).unwrap();
            let target = tree.node_weight(edge.target()).unwrap();

            assert!(
                expected_edges.contains(&(source.name(), target.name())),
                "Expected edge {:?} -> {:?} to exist",
                source.name(),
                target.name()
            );
        }
    }

    #[test]
    fn test_tree_map() {
        let model = load_model();
        let mut tree = model_tree::<i32>(&model.objects[0], &model);

        tree_map(&mut tree, |node| {
            if node.name() == "Test" {
                node.set_data(1);
            }
        });

        for node in tree.raw_nodes() {
            if node.weight.name() == "Test" {
                assert_eq!(node.weight.data(), &Some(1));
            } else {
                assert_eq!(node.weight.data(), &None);
            }
        }
    }

    #[test]
    fn test_hash_graph_same() {
        let model = load_model();
        let tree = model_tree::<()>(&model.objects[0], &model);
        let hash = hash_graph(&tree);
        println!("Hash: {}", hash);

        let tree2 = model_tree::<()>(&model.objects[0], &model);
        let hash2 = hash_graph(&tree2);
        println!("Hash2: {}", hash2);

        assert_eq!(hash, hash2);
    }

    #[test]
    fn test_hash_graph_different() {
        let model = load_model();
        let tree = model_tree::<()>(&model.objects[0], &model);
        let hash = hash_graph(&tree);
        println!("Hash: {}", hash);

        let tree2 = model_tree::<()>(&model.objects[1], &model);
        let hash2 = hash_graph(&tree2);
        println!("Hash2: {}", hash2);

        assert_ne!(hash, hash2);
    }
}
