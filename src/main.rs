use std::process::ExitCode;

use clap::Parser;
use project_initiation_evaluator::cli::{Cli, EvaluatorCommand};
use project_initiation_evaluator::{CorpusLimits, EvaluatorError, PackageInput, evaluate_package};

/// Parses the CLI, executes the selected evaluator mode, and returns a process status.
fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

/// Converts CLI DTOs into library requests and dispatches the selected operation.
fn run(cli: Cli) -> Result<(), EvaluatorError> {
    match cli.command {
        EvaluatorCommand::Evaluate(args) => {
            let input = PackageInput {
                brief: args.brief,
                generated_dir: args.generated,
                project_model: args.project_model,
                metadata: args.metadata,
                package_id: args.package_id,
                output_dir: args.output,
            };
            let report = evaluate_package(&input, CorpusLimits::default())?;
            println!("DRPFS: {:.1} / 100", report.drpfs);
            Ok(())
        }
        EvaluatorCommand::Compare(_) => Err(EvaluatorError::ModeUnavailable { mode: "compare" }),
    }
}
