use crate::fsm::FunctionBlock;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FsmStatistics {
    pub total_states: usize,
    pub total_transitions: usize,
    pub avg_transitions_per_state: f64,
    pub max_transitions_from_state: usize,
    pub unreachable_states: Vec<String>,
    pub dead_states: Vec<String>,
    pub cycles: Vec<Vec<String>>,
}

impl FsmStatistics {
    pub fn analyze(fsm: &FunctionBlock) -> Self {
        use super::{FsmValidator, CycleDetector};

        let total_states = fsm.state_count();
        let total_transitions = fsm.transition_count();

        let avg_transitions_per_state = if total_states > 0 {
            total_transitions as f64 / total_states as f64
        } else {
            0.0
        };

        let max_transitions_from_state = fsm.states
            .values()
            .map(|s| s.transitions_out.len())
            .max()
            .unwrap_or(0);

        Self {
            total_states,
            total_transitions,
            avg_transitions_per_state,
            max_transitions_from_state,
            unreachable_states: FsmValidator::find_unreachable_states(fsm),
            dead_states: FsmValidator::find_dead_states(fsm),
            cycles: CycleDetector::find_cycles(fsm),
        }
    }
}
