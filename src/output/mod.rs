pub mod text;
pub mod json;
pub mod dot;
pub mod markdown;

use crate::fsm::FiniteStateMachine;
use crate::analysis::FsmStatistics;
use anyhow::Result;
use clap::ValueEnum;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum OutputFormat {
    Text,
    Json,
    Dot,
    Markdown,
}

pub struct OutputWriter {
    format: OutputFormat,
}

impl OutputWriter {
    pub fn new(format: OutputFormat) -> Self {
        Self { format }
    }

    pub fn write(&self, fsm: &FiniteStateMachine, output_path: Option<&Path>) -> Result<()> {
        match self.format {
            OutputFormat::Text => text::print_text_table(fsm),
            OutputFormat::Json => json::export_json(fsm, output_path)?,
            OutputFormat::Dot => dot::export_graphviz(fsm, output_path)?,
            OutputFormat::Markdown => markdown::export_markdown(fsm, output_path)?,
        }
        Ok(())
    }

    pub fn write_with_analysis(
        &self,
        fsm: &FiniteStateMachine,
        stats: &HashMap<String, FsmStatistics>,
        output_path: Option<&Path>
    ) -> Result<()> {
        match self.format {
            OutputFormat::Text => text::print_with_analysis(fsm, stats),
            OutputFormat::Json => json::export_with_analysis(fsm, stats, output_path)?,
            OutputFormat::Dot => dot::export_graphviz(fsm, output_path)?,
            OutputFormat::Markdown => markdown::export_with_analysis(fsm, stats, output_path)?,
        }
        Ok(())
    }
}