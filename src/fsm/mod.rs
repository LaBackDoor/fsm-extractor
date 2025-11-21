pub mod state;
pub mod transition;
pub mod function_block;
pub mod extractor;

pub use state::State;
pub use transition::Transition;
pub use function_block::FunctionBlock;
pub use extractor::FsmExtractor;

use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FiniteStateMachine {
    pub function_blocks: Vec<FunctionBlock>,
    pub metadata: Metadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub source_file: PathBuf,
    pub extraction_date: DateTime<Utc>,
    pub total_states: usize,
    pub total_transitions: usize,
}