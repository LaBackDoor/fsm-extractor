use crate::error::FsmError;
use crate::xml_parser::{XmlParser, FunctionBlockData};
use crate::fsm::{FiniteStateMachine, FunctionBlock, State, Transition, Metadata};
use anyhow::Result;
use chrono::Utc;
use std::path::Path;

pub struct FsmExtractor {
    parser: XmlParser,
    source_path: std::path::PathBuf,
}

impl FsmExtractor {
    pub fn new(xml_path: &Path) -> Result<Self> {
        let parser = XmlParser::new(xml_path)?;
        Ok(Self {
            parser,
            source_path: xml_path.to_path_buf(),
        })
    }

    pub fn extract(&self) -> Result<FiniteStateMachine> {
        let function_block_names = self.parser.find_function_blocks();

        if function_block_names.is_empty() {
            return Err(FsmError::NoFunctionBlocks.into());
        }

        let mut function_blocks = Vec::new();
        let mut total_states = 0;
        let mut total_transitions = 0;
        

        // KEEP AND MODIFY THIS LOOP TO PROCESS ALL BLOCKS
        for name in &function_block_names {

            if let Ok(fb_data) = self.parser.extract_function_block(name) {
                if let Ok(fb) = self.build_function_block(fb_data) {
                    if fb.state_count() > 0 || fb.transition_count() > 0 {
                        total_states += fb.state_count();
                        total_transitions += fb.transition_count();
                        function_blocks.push(fb);
                    }
                }
            }
        }

        let metadata = Metadata {
            source_file: self.source_path.clone(),
            extraction_date: Utc::now(),
            total_states,
            total_transitions,
        };

        Ok(FiniteStateMachine {
            function_blocks,
            metadata,
        })
    }

    pub fn extract_filtered(&self, filters: &[String]) -> Result<FiniteStateMachine> {
        let function_block_names = self.parser.find_function_blocks();

        let mut function_blocks = Vec::new();
        let mut total_states = 0;
        let mut total_transitions = 0;

        for name in &function_block_names {
            if !filters.contains(name) {
                continue;
            }

            if let Ok(fb_data) = self.parser.extract_function_block(name) {
                if let Ok(fb) = self.build_function_block(fb_data) {
                    total_states += fb.state_count();
                    total_transitions += fb.transition_count();
                    function_blocks.push(fb);
                }
            }
        }

        let metadata = Metadata {
            source_file: self.source_path.clone(),
            extraction_date: Utc::now(),
            total_states,
            total_transitions,
        };

        Ok(FiniteStateMachine {
            function_blocks,
            metadata,
        })
    }

    fn build_function_block(&self, fb_data: FunctionBlockData) -> Result<FunctionBlock> {
        let mut function_block = FunctionBlock::new(
            fb_data.name.clone(),
            fb_data.case_variable.clone(),
        );

        // First pass: create all states
        for element in &fb_data.case_elements {
            let state = State::new(element.state_id.clone());
            function_block.add_state(state);
        }

        // Second pass: extract transitions
        for element in &fb_data.case_elements {
            let current_state = element.state_id.clone();

            // Handle case with no if statements
            if element.if_statements.is_empty() {
                // This state has no transitions
                continue;
            }

            for if_stmt in &element.if_statements {
                // Look for assignments to the case variable
                for assignment in &if_stmt.assignments {
                    if assignment.variable == fb_data.case_variable {
                        let next_state = assignment.value.clone();

                        // Create the transition
                        let condition = if if_stmt.condition.is_empty() {
                            "No Check".to_string()
                        } else {
                            if_stmt.condition.clone()
                        };

                        let transition = Transition::new(
                            current_state.clone(),
                            next_state.clone(),
                            condition,
                        );

                        // Ensure the target state exists
                        if !function_block.states.contains_key(&next_state) {
                            function_block.add_state(State::new(next_state));
                        }

                        function_block.add_transition(transition);
                    }
                }
            }
        }

        Ok(function_block)
    }
}