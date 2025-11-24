use crate::fsm::{FiniteStateMachine, FunctionBlock};
use crate::analysis::{FsmStatistics, StateSignatureTable}; // ✅ NEW IMPORT
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

// Row structure for signature table
#[derive(Tabled)]
struct SignatureRow {
    #[tabled(rename = "State")]
    state: String,
    #[tabled(rename = "Signature Conditions")]
    conditions: String,
    #[tabled(rename = "Paths")]
    paths: String,
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

// Print FSM with signatures
pub fn print_with_signatures(
    fsm: &FiniteStateMachine,
    signatures: &HashMap<String, StateSignatureTable>
) {
    for fb in &fsm.function_blocks {
        print_function_block(fb);

        if let Some(sig_table) = signatures.get(&fb.name) {
            print_signature_table(sig_table);
        }
    }
}

// Print FSM with full analysis (stats + signatures)
pub fn print_with_full_analysis(
    fsm: &FiniteStateMachine,
    stats: &HashMap<String, FsmStatistics>,
    signatures: &HashMap<String, StateSignatureTable>,
) {
    for fb in &fsm.function_blocks {
        print_function_block(fb);

        // Print analysis
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

        // Print signatures
        if let Some(sig_table) = signatures.get(&fb.name) {
            print_signature_table(sig_table);
        }
    }
}

// Print signature table
fn print_signature_table(sig_table: &StateSignatureTable) {
    println!("\n{}", "State Signatures:".bold().cyan());
    println!("Case Variable: {}", sig_table.case_variable.yellow());

    let rows: Vec<SignatureRow> = sig_table.signatures
        .values()
        .map(|sig| SignatureRow {
            state: sig.state_id.clone(),
            conditions: sig.format_conditions(),
            paths: sig.paths_count.to_string(),
        })
        .collect();

    if !rows.is_empty() {
        let table = Table::new(rows)
            .with(Style::modern())
            .to_string();
        println!("{}", table);
    } else {
        println!("No signatures generated.");
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