use crate::fsm::FiniteStateMachine;
use crate::analysis::FsmStatistics;
use anyhow::Result;
use serde_json;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub fn export_json(fsm: &FiniteStateMachine, output_path: Option<&Path>) -> Result<()> {
    let json = serde_json::to_string_pretty(fsm)?;

    if let Some(path) = output_path {
        let mut file = File::create(path)?;
        file.write_all(json.as_bytes())?;
    } else {
        println!("{}", json);
    }

    Ok(())
}

pub fn export_with_analysis(
    fsm: &FiniteStateMachine,
    stats: &HashMap<String, FsmStatistics>,
    output_path: Option<&Path>
) -> Result<()> {
    #[derive(serde::Serialize)]
    struct FsmWithAnalysis<'a> {
        fsm: &'a FiniteStateMachine,
        analysis: &'a HashMap<String, FsmStatistics>,
    }

    let data = FsmWithAnalysis { fsm, analysis: stats };
    let json = serde_json::to_string_pretty(&data)?;

    if let Some(path) = output_path {
        let mut file = File::create(path)?;
        file.write_all(json.as_bytes())?;
    } else {
        println!("{}", json);
    }

    Ok(())
}