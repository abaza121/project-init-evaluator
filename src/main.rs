use std::process::ExitCode;

use clap::Parser;
use project_initiation_evaluator::cli::{Cli, EvaluatorCommand};
use project_initiation_evaluator::{
    ComparisonInput, CorpusLimits, EvaluatorError, PackageInput, compare_packages, evaluate_package,
};

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
        EvaluatorCommand::Compare(args) => {
            let input = ComparisonInput {
                brief: args.brief,
                baseline_dir: args.baseline,
                candidate_dir: args.candidate,
                baseline_model: args.baseline_model,
                candidate_model: args.candidate_model,
                baseline_metadata: args.baseline_metadata,
                candidate_metadata: args.candidate_metadata,
                output_dir: args.output,
            };
            let result = compare_packages(&input, CorpusLimits::default())?;
            println!(
                "Baseline DRPFS: {:.1}; Candidate DRPFS: {:.1}",
                result.baseline.drpfs, result.candidate.drpfs
            );
            Ok(())
        }
    }
}
