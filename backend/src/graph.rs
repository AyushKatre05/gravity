use std::collections::HashMap;
use petgraph::graph::{DiGraph, NodeIndex};
use serde_json;

use crate::models::{GraphData, GraphEdge, GraphNode, ParsedFile};

pub struct DependencyGraph {
    pub graph: DiGraph<String, ()>,
    pub index_map: HashMap<String, NodeIndex>,
}

impl DependencyGraph {
    pub fn from_parsed(files: &[ParsedFile]) -> Self {
        let mut graph: DiGraph<String, ()> = DiGraph::new();
        let mut index_map: HashMap<String, NodeIndex> = HashMap::new();

        for pf in files {
            let idx = graph.add_node(pf.path.clone());
            index_map.insert(pf.path.clone(), idx);
        }

        for pf in files {
            let src_idx = match index_map.get(&pf.path) {
                Some(&i) => i,
                None => continue,
            };

            for import in &pf.imports {
                let target_id = normalise_import(import);

                let dst_idx = if let Some(&i) = index_map.get(&target_id) {
                    i
                } else {
                    let i = graph.add_node(target_id.clone());
                    index_map.insert(target_id.clone(), i);
                    i
                };

                // Avoid duplicate edges
                if !graph.contains_edge(src_idx, dst_idx) {
                    graph.add_edge(src_idx, dst_idx, ());
                }
            }
        }

        DependencyGraph { graph, index_map }
    }

    pub fn to_graph_data(&self) -> GraphData {
        let nodes: Vec<GraphNode> = self
            .graph
            .node_indices()
            .map(|idx| {
                let id = &self.graph[idx];
                let label = id.split("::").last().unwrap_or(id).to_string();
                // Heuristic: if the id contains "/" it's a file node, otherwise external
                let kind = if id.contains('/') { "file" } else { "module" };
                GraphNode {
                    id: id.clone(),
                    label,
                    kind: kind.to_string(),
                }
            })
            .collect();

        let edges: Vec<GraphEdge> = self
            .graph
            .edge_indices()
            .filter_map(|e| {
                let (a, b) = self.graph.edge_endpoints(e)?;
                Some(GraphEdge {
                    from: self.graph[a].clone(),
                    to: self.graph[b].clone(),
                    label: None,
                })
            })
            .collect();

        GraphData { nodes, edges }
    }
}


fn normalise_import(raw: &str) -> String {
    let trimmed = raw
        .trim()
        .trim_start_matches("use ")
        .trim_end_matches(';')
        .trim();

    let top = trimmed.split("::").next().unwrap_or(trimmed);

    match top {
        "std" | "core" | "alloc" => trimmed.to_owned(),
        _ => trimmed.to_owned(),
    }
}
