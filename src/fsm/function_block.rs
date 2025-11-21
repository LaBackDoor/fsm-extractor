use crate::fsm::{State, Transition};
use indexmap::IndexMap;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionBlock {
    pub name: String,
    pub case_variable: String,
    pub states: IndexMap<String, State>,
    pub transitions: Vec<Transition>,
}

impl FunctionBlock {
    pub fn new(name: String, case_variable: String) -> Self {
        Self {
            name,
            case_variable,
            states: IndexMap::new(),
            transitions: Vec::new(),
        }
    }

    pub fn add_state(&mut self, state: State) {
        self.states.insert(state.id.clone(), state);
    }

    pub fn add_transition(&mut self, transition: Transition) {
        // Update state references
        if let Some(from_state) = self.states.get_mut(&transition.from_state) {
            from_state.transitions_out.push(transition.id.clone());
        }
        if let Some(to_state) = self.states.get_mut(&transition.to_state) {
            to_state.transitions_in.push(transition.id.clone());
        }

        self.transitions.push(transition);
    }

    pub fn get_state(&self, id: &str) -> Option<&State> {
        self.states.get(id)
    }

    pub fn state_count(&self) -> usize {
        self.states.len()
    }

    pub fn transition_count(&self) -> usize {
        self.transitions.len()
    }
}