use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    pub id: String,
    pub name: Option<String>,
    pub transitions_out: Vec<String>,  // IDs of outgoing transitions
    pub transitions_in: Vec<String>,   // IDs of incoming transitions
}

impl State {
    pub fn new(id: String) -> Self {
        Self {
            id,
            name: None,
            transitions_out: Vec::new(),
            transitions_in: Vec::new(),
        }
    }
}
