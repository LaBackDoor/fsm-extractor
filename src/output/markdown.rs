use crate::fsm::FiniteStateMachine;
use crate::analysis::FsmStatistics;
use anyhow::Result;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub fn export_markdown(fsm: &FiniteStateMachine, output_path: Option<&Path>) -> Result<()> {
    let mut md = String::new();

    md.push_str("# FSM Extraction Report\n\n");
    md.push_str(&format!("**Source File:** {}\n", fsm.metadata.source_file.display()));
    md.push_str(&format!("**Extraction Date:** {}\n", fsm.metadata.extraction_date));
    md.push_str(&format!("**Total States:** {}\n", fsm.metadata.total_states));
    md.push_str(&format!("**Total Transitions:** {}\n\n", fsm.metadata.total_transitions));

    for fb in &fsm.function_blocks {
        md.push_str(&format!("## Function Block: {}\n\n", fb.name));
        md.push_str(&format!("**Case Variable:** `{}`\n\n", fb.case_variable));
        md.push_str(&format!("**States:** {} | **Transitions:** {}\n\n",
                             fb.state_count(),
                             fb.transition_count()
        ));

        if !fb.transitions.is_empty() {
            md.push_str("| Current State | Next State | Transition Condition |\n");
            md.push_str("|---------------|------------|---------------------|\n");

            for transition in &fb.transitions {
                md.push_str(&format!(
                    "| {} | {} | {} |\n",
                    transition.from_state,
                    transition.to_state,
                    transition.condition.replace('|', "\\|")
                ));
            }

            md.push_str("\n");
        }
    }

    if let Some(path) = output_path {
        let mut file = File::create(path)?;
        file.write_all(md.as_bytes())?;
    } else {
        println!("{}", md);
    }

    Ok(())
}

pub fn export_with_analysis(
    fsm: &FiniteStateMachine,
    stats: &HashMap<String, FsmStatistics>,
    output_path: Option<&Path>
) -> Result<()> {
    let mut md = String::new();

    md.push_str("# FSM Extraction and Analysis Report\n\n");
    md.push_str(&format!("**Source File:** {}\n", fsm.metadata.source_file.display()));
    md.push_str(&format!("**Extraction Date:** {}\n", fsm.metadata.extraction_date));
    md.push_str(&format!("**Total States:** {}\n", fsm.metadata.total_states));
    md.push_str(&format!("**Total Transitions:** {}\n\n", fsm.metadata.total_transitions));

    for fb in &fsm.function_blocks {
        md.push_str(&format!("## Function Block: {}\n\n", fb.name));
        md.push_str(&format!("**Case Variable:** `{}`\n\n", fb.case_variable));

        // Add analysis if available
        if let Some(stat) = stats.get(&fb.name) {
            md.push_str("### Analysis Results\n\n");
            md.push_str(&format!("- **Total States:** {}\n", stat.total_states));
            md.push_str(&format!("- **Total Transitions:** {}\n", stat.total_transitions));
            md.push_str(&format!("- **Avg Transitions/State:** {:.2}\n", stat.avg_transitions_per_state));
            md.push_str(&format!("- **Max Transitions from State:** {}\n", stat.max_transitions_from_state));

            if !stat.unreachable_states.is_empty() {
                md.push_str(&format!("- **Unreachable States:** {:?}\n", stat.unreachable_states));
            }
            if !stat.dead_states.is_empty() {
                md.push_str(&format!("- **Dead-end States:** {:?}\n", stat.dead_states));
            }
            if !stat.cycles.is_empty() {
                md.push_str(&format!("- **Cycles Found:** {}\n", stat.cycles.len()));
            }

            md.push_str("\n");
        }

        md.push_str("### State Transitions\n\n");

        if !fb.transitions.is_empty() {
            md.push_str("| Current State | Next State | Transition Condition |\n");
            md.push_str("|---------------|------------|---------------------|\n");

            for transition in &fb.transitions {
                md.push_str(&format!(
                    "| {} | {} | {} |\n",
                    transition.from_state,
                    transition.to_state,
                    transition.condition.replace('|', "\\|")
                ));
            }

            md.push_str("\n");
        }
    }

    if let Some(path) = output_path {
        let mut file = File::create(path)?;
        file.write_all(md.as_bytes())?;
    } else {
        println!("{}", md);
    }

    Ok(())
}