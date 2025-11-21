pub mod validator;
pub mod cycles;
pub mod stats;

use crate::fsm::FiniteStateMachine;
use colored::*;
use std::collections::HashMap;

pub use cycles::CycleDetector;
pub use stats::FsmStatistics;
pub use validator::FsmValidator;

pub struct FsmAnalyzer;

impl FsmAnalyzer {
    pub fn new() -> Self {
        Self
    }

    pub fn analyze_all(&self, fsm: &FiniteStateMachine) -> HashMap<String, FsmStatistics> {
        let mut results = HashMap::new();

        for fb in &fsm.function_blocks {
            let stats = FsmStatistics::analyze(fb);
            results.insert(fb.name.clone(), stats);
        }

        results
    }

    pub fn analyze_and_report(&self, fsm: &FiniteStateMachine, options: &AnalysisOptions) -> anyhow::Result<()> {
        for fb in &fsm.function_blocks {
            println!("\n{}", format!("Analyzing Function Block: {}", fb.name).bold().blue());
            println!("{}", "=".repeat(50));

            if options.check_unreachable {
                let unreachable = FsmValidator::find_unreachable_states(fb);
                if !unreachable.is_empty() {
                    println!("{} Unreachable states found:", "⚠".yellow());
                    for state in &unreachable {
                        println!("  - State {}", state.red());
                    }
                } else {
                    println!("{} No unreachable states", "✓".green());
                }
            }

            if options.check_dead_states {
                let dead = FsmValidator::find_dead_states(fb);
                if !dead.is_empty() {
                    println!("{} Dead-end states found:", "⚠".yellow());
                    for state in &dead {
                        println!("  - State {}", state.red());
                    }
                } else {
                    println!("{} No dead-end states", "✓".green());
                }
            }

            if options.check_cycles {
                let cycles = CycleDetector::find_cycles(fb);
                if !cycles.is_empty() {
                    println!("{} Cycles detected:", "ℹ".blue());
                    for cycle in &cycles {
                        println!("  - {}", cycle.join(" → "));
                    }
                } else {
                    println!("{} No cycles detected", "✓".green());
                }
            }

            // Always show statistics
            let stats = FsmStatistics::analyze(fb);
            println!("\n{}", "Statistics:".bold());
            println!("  Total states: {}", stats.total_states);
            println!("  Total transitions: {}", stats.total_transitions);
            println!("  Avg transitions per state: {:.2}", stats.avg_transitions_per_state);
            println!("  Max transitions from state: {}", stats.max_transitions_from_state);
        }

        Ok(())
    }
}

pub struct AnalysisOptions {
    pub check_cycles: bool,
    pub check_unreachable: bool,
    pub check_dead_states: bool,
}
