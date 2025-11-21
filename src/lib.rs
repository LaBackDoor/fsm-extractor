pub mod error;
pub mod xml_parser;
pub mod fsm;
pub mod analysis;
pub mod output;

pub use fsm::{FsmExtractor, FiniteStateMachine, FunctionBlock, State, Transition};
pub use analysis::{FsmAnalyzer, FsmStatistics};
pub use output::{OutputFormat, OutputWriter};