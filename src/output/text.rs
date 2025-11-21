use crate::fsm::{FiniteStateMachine, FunctionBlock};
use crate::analysis::FsmStatistics;
use colored::*;
use tabled::{Table, Tabled, settings::Style};
use std::collections::HashMap;

#[derive(Tabled)]
struct TransitionRow {
    #[tabled(rename = "Current State")]
    current_state: String,
    #[tabled(rename = "Next State")]
    next_state: String,
    #[tabled(rename = "Transition Condition")]
    condition: String,
}

pub fn print_text_table(fsm: &FiniteStateMachine) {
    for fb in &fsm.function_blocks {
        print_function_block(fb);
    }
}

pub fn print_with_analysis(fsm: &FiniteStateMachine, stats: &HashMap<String, FsmStatistics>) {
    for fb in &fsm.function_blocks {
        print_function_block(fb);

        if let Some(stat) = stats.get(&fb.name) {
            println!("\n{}", "Analysis Results:".bold());
            println!("  Unreachable states: {}",
                     if stat.unreachable_states.is_empty() { "None".green() }
                     else { format!("{:?}", stat.unreachable_states).red() });
            println!("  Dead-end states: {}",
                     if stat.dead_states.is_empty() { "None".green() }
                     else { format!("{:?}", stat.dead_states).red() });
            println!("  Cycles: {}",
                     if stat.cycles.is_empty() { "None".green() }
                     else { format!("{} found", stat.cycles.len()).yellow() });
        }
    }
}

fn print_function_block(fb: &FunctionBlock) {
    println!("\n{}", format!("Function Block: {}", fb.name).bold().cyan());
    println!("Case Variable: {}", fb.case_variable.yellow());
    println!("\nStates: {} | Transitions: {}\n",
             fb.state_count().to_string().green(),
             fb.transition_count().to_string().green()
    );

    let rows: Vec<TransitionRow> = fb.transitions
        .iter()
        .map(|t| TransitionRow {
            current_state: t.from_state.clone(),
            next_state: t.to_state.clone(),
            condition: t.condition.clone(),
        })
        .collect();

    if !rows.is_empty() {
        let table = Table::new(rows)
            .with(Style::modern())
            .to_string();
        println!("{}", table);
    } else {
        println!("No transitions found.");
    }
}