use crate::fsm::FiniteStateMachine;
use anyhow::Result;
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub fn export_graphviz(fsm: &FiniteStateMachine, output_path: Option<&Path>) -> Result<()> {
    let mut dot = String::new();

    for (idx, fb) in fsm.function_blocks.iter().enumerate() {
        if idx > 0 {
            dot.push_str("\n\n");
        }

        dot.push_str(&format!("digraph \"{}\" {{\n", fb.name));
        dot.push_str("    rankdir=LR;\n");
        dot.push_str("    node [shape=circle, style=filled, fillcolor=lightblue];\n");
        dot.push_str("    edge [fontsize=10];\n\n");

        // Add nodes
        for state in fb.states.keys() {
            dot.push_str(&format!("    \"{}\" [label=\"{}\"];\n", state, state));
        }

        dot.push_str("\n");

        // Add edges
        for transition in &fb.transitions {
            let label = transition.condition
                .replace('\"', "\\\"")
                .replace('\n', "\\n");

            dot.push_str(&format!(
                "    \"{}\" -> \"{}\" [label=\"{}\"];\n",
                transition.from_state,
                transition.to_state,
                label
            ));
        }

        dot.push_str("}");
    }

    if let Some(path) = output_path {
        let mut file = File::create(path)?;
        file.write_all(dot.as_bytes())?;
    } else {
        println!("{}", dot);
    }

    Ok(())
}