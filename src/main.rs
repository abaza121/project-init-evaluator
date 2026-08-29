use clap::Parser;
use project_initiation_evaluator::cli::Cli;

/// Parses the documented command surface while the evaluation slices are initialized.
fn main() {
    let _cli = Cli::parse();
    eprintln!("evaluation engine is not initialized yet");
    std::process::exit(2);
}
