use crate::fsm::FunctionBlock;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::algo::kosaraju_scc;
use std::collections::HashMap;

pub struct CycleDetector;

impl CycleDetector {
    pub fn find_cycles(fsm: &FunctionBlock) -> Vec<Vec<String>> {
        let mut graph = DiGraph::new();
        let mut node_map: HashMap<String, NodeIndex> = HashMap::new();
        let mut index_map: HashMap<NodeIndex, String> = HashMap::new();

        // Add nodes
        for state_id in fsm.states.keys() {
            let idx = graph.add_node(state_id.clone());
            node_map.insert(state_id.clone(), idx);
            index_map.insert(idx, state_id.clone());
        }

        // Add edges
        for transition in &fsm.transitions {
            if let (Some(&from_idx), Some(&to_idx)) =
                (node_map.get(&transition.from_state), node_map.get(&transition.to_state)) {
                graph.add_edge(from_idx, to_idx, ());
            }
        }

        // Find strongly connected components
        let sccs = kosaraju_scc(&graph);

        // Filter out single-node SCCs without self-loops
        let mut cycles = Vec::new();
        for scc in sccs {
            if scc.len() > 1 {
                let cycle: Vec<String> = scc.iter()
                    .filter_map(|idx| index_map.get(idx).cloned())
                    .collect();
                cycles.push(cycle);
            } else if scc.len() == 1 {
                // Check for self-loop
                let node = scc[0];
                if graph.find_edge(node, node).is_some() {
                    if let Some(state_id) = index_map.get(&node) {
                        cycles.push(vec![state_id.clone()]);
                    }
                }
            }
        }

        cycles
    }

    pub fn is_acyclic(fsm: &FunctionBlock) -> bool {
        Self::find_cycles(fsm).is_empty()
    }
}