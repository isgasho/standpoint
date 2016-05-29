extern crate graph_match;

use std::collections::HashMap;
use graph_match::graph::Graph;

pub fn find_verbs(graph: &Graph) -> Option<Vec<usize>> {
    let mut verb_indices: Vec<usize> = Vec::new();
    for node_index in 0..graph.nodes.len() {
        let attrs = try_opt_clone!(graph.nodes[node_index].attributes);
        let pos = try_opt_clone!(attrs.get("pos"));
        if pos.contains("VB") {
            verb_indices.push(node_index);
        }
    }
    return Some(verb_indices);
}

#[test]
fn find_verbs_in_graph() {
    let mut graph = Graph {
        edges: vec![],
        nodes: vec![],
    };
    let mut verb_attrs: HashMap<String, String> = HashMap::new();
    verb_attrs.insert(String::from("pos"), String::from("VBD"));
    let mut other_attrs: HashMap<String, String> = HashMap::new();
    other_attrs.insert(String::from("pos"), String::from("NN"));
    graph.add_node(String::from("node0"), Some(verb_attrs.clone()));
    graph.add_node(String::from("node1"), Some(other_attrs.clone()));
    graph.add_node(String::from("node2"), Some(verb_attrs.clone()));

    assert_eq!(vec![0, 2], find_verbs(&graph).unwrap());
}

pub fn build_queries(verb_indices: &Vec<usize>,
                     graph: &Graph,
                     verbs: &HashMap<String, Vec<String>>,
                     frames: &HashMap<String, (usize, Graph)>)
                     -> Option<Vec<(usize, String)>> {
    let mut queries: Vec<(usize, String)> = Vec::new();
    for verb_index in verb_indices {
        let attrs = try_opt_clone!(graph.nodes[*verb_index].attributes);
        let lemma = try_opt_clone!(attrs.get("lemma"));
        let patterns = try_opt_clone!(verbs.get(lemma));
        for pattern in patterns {
            match frames.get(pattern) {
                Some(_) => queries.push((*verb_index, pattern.clone())),
                None => println!("Frame missing for: {}", pattern),
            }
        }
    }
    return Some(queries);
}

#[test]
fn build_queries_for_verbs_in_graph() {
    use graph_parser;
    let verb_indices = vec![0];
    let mut graph = Graph {
        edges: vec![],
        nodes: vec![],
    };

    let mut verb_attrs: HashMap<String, String> = HashMap::new();
    verb_attrs.insert(String::from("pos"), String::from("VBD"));
    verb_attrs.insert(String::from("lemma"), String::from("run"));

    graph.add_node(String::from("node0"), Some(verb_attrs.clone()));

    let mut verbs: HashMap<String, Vec<String>> = HashMap::new();
    verbs.insert(String::from("run"), vec![String::from("NP V NP")]);
    let mut frames: HashMap<String, (usize, Graph)> = HashMap::new();

    let query_string = "type:node identifier:subj\n\
                        type:node identifier:verb pos:VB\n\
                        type:node identifier:obj\n\
                        type:edge identifier:subj source:1 target:0 label:nsubj\n\
                        type:edge identifier:obj source:1 target:2 label:dobj";
    let frame = (1, graph_parser::parse(&query_string.to_string()).unwrap());
    frames.insert(String::from("NP V NP"), frame);
    let result = build_queries(&verb_indices, &graph, &verbs, &frames).unwrap();
    assert_eq!(vec![(0, String::from("NP V NP"))], result);
}

pub fn matched_components_to_pattern(matched_components: &graph_match::matching::MatchedComponents,
                                     graph: &Graph)
                                     -> String {
    let mut pattern: Vec<(usize, String)> = Vec::new();

    for component in &matched_components.list {
        let mut label = String::new();
        match graph.nodes[component.node].attributes {
            Some(ref attrs) => {
                match attrs.get("lemma") {
                    Some(lemma) => label.push_str(lemma.as_str()),
                    None => label.push_str("BLANK"),
                }
            }
            None => label.push_str("BLANK"),
        }
        label.push_str(".");
        match component.from_edge {
            Some(edge) => label.push_str(&graph.edges[edge].identifier.as_str()),
            None => label.push_str("root"),
        }
        pattern.push((component.node, label));
    }

    pattern.sort_by(|a, b| a.0.cmp(&b.0));
    return pattern.iter().map(|e| e.1.clone()).collect::<Vec<String>>().join(" ");
}

#[test]
fn matched_component_pattern() {
    use std::collections::HashMap;
    let mut graph = graph_match::graph::Graph {
        nodes: vec![],
        edges: vec![],
    };
    let mut attributes: HashMap<String, String> = HashMap::new();
    attributes.insert(String::from("lemma"), String::from("cat"));
    graph.add_node(String::from("node1"), Some(attributes.clone()));
    graph.add_edge(0, 0, String::from("edge1"), None);
    let matched_components = graph_match::matching::MatchedComponents {
        list: vec![
            graph_match::matching::Component { from_edge: Some(0), node: 0},
        ],
    };
    assert_eq!(String::from("cat.edge1"),
               matched_components_to_pattern(&matched_components, &graph));
}

pub fn subgraph_nodes_to_extract_string(node_indexes: &Vec<usize>, graph: &Graph) -> String {
    let mut extract_string = String::new();
    let mut sorted_node_indexes = node_indexes.clone();
    sorted_node_indexes.sort_by(|a, b| a.cmp(b));
    for index in sorted_node_indexes {
        match graph.nodes[index].attributes {
            Some(ref attrs) => {
                match attrs.get("before") {
                    Some(value) => {
                        match value.as_str() {
                            "space" => extract_string.push_str(" "),
                            "empty" => extract_string.push_str(""),
                            "unknown" => extract_string.push_str("?"),
                            _ => extract_string.push_str("???"),
                        }
                    }
                    _ => {}
                }
                match attrs.get("word") {
                    Some(value) => extract_string.push_str(value.as_str()),
                    _ => {}
                }
                match attrs.get("after") {
                    Some(value) => {
                        match value.as_str() {
                            "space" => extract_string.push_str(" "),
                            "empty" => extract_string.push_str(""),
                            "unknown" => extract_string.push_str("?"),
                            _ => extract_string.push_str("???"),
                        }
                    }
                    _ => {}
                }
            }
            None => {}
        };
    }
    return extract_string.replace("  ", " ");
}

#[test]
fn subgraph_string() {
    use std::collections::HashMap;
    let mut graph = graph_match::graph::Graph {
        nodes: vec![],
        edges: vec![],
    };
    let mut attributes: HashMap<String, String> = HashMap::new();
    attributes.insert(String::from("word"), String::from("cat"));
    attributes.insert(String::from("before"), String::from("space"));
    attributes.insert(String::from("after"), String::from("empty"));
    graph.add_node(String::from("node1"), Some(attributes.clone()));
    assert_eq!(String::from(" cat"),
               subgraph_nodes_to_extract_string(&vec![0], &graph));
}
