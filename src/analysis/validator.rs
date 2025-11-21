use crate::fsm::FunctionBlock;
use std::collections::{HashSet, VecDeque};

pub struct FsmValidator;

impl FsmValidator {
    pub fn find_unreachable_states(fsm: &FunctionBlock) -> Vec<String> {
        if fsm.states.is_empty() {
            return Vec::new();
        }

        let mut reachable = HashSet::new();
        let mut queue = VecDeque::new();

        // Find initial states (states with no incoming transitions)
        let initial_states: Vec<_> = fsm.states
            .values()
            .filter(|s| s.transitions_in.is_empty())
            .map(|s| s.id.clone())
            .collect();

        // If no initial states found, use the first state or state "100"
        if initial_states.is_empty() {
            if fsm.states.contains_key("100") {
                queue.push_back("100".to_string());
            } else if let Some(first) = fsm.states.keys().next() {
                queue.push_back(first.clone());
            }
        } else {
            for state in initial_states {
                queue.push_back(state);
            }
        }

        // BFS to find all reachable states
        while let Some(state_id) = queue.pop_front() {
            if !reachable.insert(state_id.clone()) {
                continue;
            }

            // Find all transitions from this state
            for transition in &fsm.transitions {
                if transition.from_state == state_id {
                    queue.push_back(transition.to_state.clone());
                }
            }
        }

        // Find unreachable states
        fsm.states
            .keys()
            .filter(|id| !reachable.contains(*id))
            .cloned()
            .collect()
    }

    pub fn find_dead_states(fsm: &FunctionBlock) -> Vec<String> {
        fsm.states
            .values()
            .filter(|s| s.transitions_out.is_empty())
            .map(|s| s.id.clone())
            .collect()
    }

    pub fn validate_references(fsm: &FunctionBlock) -> anyhow::Result<()> {
        for transition in &fsm.transitions {
            if !fsm.states.contains_key(&transition.from_state) {
                return Err(anyhow::anyhow!(
                    "Invalid state reference in transition: from_state '{}'",
                    transition.from_state
                ));
            }
            if !fsm.states.contains_key(&transition.to_state) {
                return Err(anyhow::anyhow!(
                    "Invalid state reference in transition: to_state '{}'",
                    transition.to_state
                ));
            }
        }
        Ok(())
    }
}