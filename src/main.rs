use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod error;
mod xml_parser;
mod fsm;
mod analysis;
mod output;

use crate::fsm::FsmExtractor;
use crate::output::{OutputFormat, OutputWriter};
use crate::analysis::{FsmAnalyzer, AnalysisOptions};

#[derive(Parser)]
#[command(name = "fsm-extractor")]
#[command(about = "Extract and analyze FSMs from PLC XML output")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Extract FSM from XML
    Extract {
        /// Input XML file
        #[arg(value_name = "FILE")]
        input: PathBuf,
        
        /// Output format
        #[arg(short, long, value_enum, default_value = "text")]
        format: OutputFormat,
        
        /// Output file (stdout if not specified)
        #[arg(short, long)]
        output: Option<PathBuf>,
        
        /// Filter function blocks (comma-separated)
        #[arg(short = 'f', long, value_delimiter = ',')]
        function_block: Option<Vec<String>>,
        
        /// Include analysis in output
        #[arg(short = 'a', long)]
        analyze: bool,
    },
    
    /// Analyze FSM structure
    Analyze {
        /// Input XML file
        input: PathBuf,
        
        /// Check for cycles
        #[arg(long)]
        check_cycles: bool,
        
        /// Check for unreachable states
        #[arg(long)]
        check_unreachable: bool,
        
        /// Check for dead-end states
        #[arg(long)]
        check_dead_states: bool,
        
        /// Show all checks
        #[arg(long)]
        all: bool,
    },
    
    /// Generate visualization
    Visualize {
        /// Input XML file
        input: PathBuf,
        
        /// Output image file
        #[arg(short, long)]
        output: PathBuf,
        
        /// Filter function blocks (comma-separated)
        #[arg(short = 'f', long, value_delimiter = ',')]
        function_block: Option<Vec<String>>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Extract { input, format, output, function_block, analyze } => {
            let extractor = FsmExtractor::new(&input)?;
            let fsm = if let Some(filters) = function_block {
                extractor.extract_filtered(&filters)?
            } else {
                extractor.extract()?
            };
            
            let writer = OutputWriter::new(format);
            
            if analyze {
                let analyzer = FsmAnalyzer::new();
                let stats = analyzer.analyze_all(&fsm);
                writer.write_with_analysis(&fsm, &stats, output.as_deref())?;
            } else {
                writer.write(&fsm, output.as_deref())?;
            }
        },
        Commands::Analyze { input, check_cycles, check_unreachable, check_dead_states, all } => {
            let extractor = FsmExtractor::new(&input)?;
            let fsm = extractor.extract()?;
            
            let options = AnalysisOptions {
                check_cycles: check_cycles || all,
                check_unreachable: check_unreachable || all,
                check_dead_states: check_dead_states || all,
            };
            
            let analyzer = FsmAnalyzer::new();
            analyzer.analyze_and_report(&fsm, &options)?;
        },
        Commands::Visualize { input, output, function_block } => {
            let extractor = FsmExtractor::new(&input)?;
            let fsm = if let Some(filters) = function_block {
                extractor.extract_filtered(&filters)?
            } else {
                extractor.extract()?
            };
            
            let writer = OutputWriter::new(OutputFormat::Dot);
            writer.write(&fsm, Some(&output))?;
            
            println!("Visualization saved to: {}", output.display());
            println!("Generate image with: dot -Tpng {} -o {}.png", output.display(), output.display());
        }
    }
    
    Ok(())
}