use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transition {
    pub id: String,
    pub from_state: String,
    pub to_state: String,
    pub condition: String,
    pub raw_expression: String,
}

impl Transition {
    pub fn new(from: String, to: String, condition: String) -> Self {
        let id = format!("{}_to_{}", from, to);
        Self {
            id,
            from_state: from,
            to_state: to,
            condition: condition.clone(),
            raw_expression: condition,
        }
    }
}