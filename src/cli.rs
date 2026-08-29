use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

/// Parses the evaluator's top-level command and selected operating mode.
#[derive(Debug, Parser)]
#[command(version, about)]
pub struct Cli {
    /// Evaluation mode to execute.
    #[command(subcommand)]
    pub command: EvaluatorCommand,
}

/// Selects single-package evaluation or isolated baseline comparison.
#[derive(Debug, Subcommand)]
pub enum EvaluatorCommand {
    /// Evaluate one generated package and write JSON and Markdown reports.
    Evaluate(EvaluateArgs),
    /// Evaluate baseline and candidate packages independently, then compare them.
    Compare(CompareArgs),
}

/// Describes paths and identity for a single-package evaluation.
#[derive(Debug, Args)]
pub struct EvaluateArgs {
    /// Original user brief used to generate the project foundation.
    #[arg(long)]
    pub brief: PathBuf,
    /// Generated project directory to inspect without modifying it.
    #[arg(long)]
    pub generated: PathBuf,
    /// Optional structured project model or database export in JSON format.
    #[arg(long)]
    pub project_model: Option<PathBuf>,
    /// Optional validation metadata in JSON format.
    #[arg(long)]
    pub metadata: Option<PathBuf>,
    /// Stable package identifier recorded in evaluator-owned evidence.
    #[arg(long, default_value = "package")]
    pub package_id: String,
    /// Directory that will receive the generated reports.
    #[arg(long)]
    pub output: PathBuf,
}

/// Describes isolated package paths for baseline-versus-candidate comparison.
#[derive(Debug, Args)]
pub struct CompareArgs {
    /// Original user brief shared by both generated packages.
    #[arg(long)]
    pub brief: PathBuf,
    /// Baseline generated project directory.
    #[arg(long)]
    pub baseline: PathBuf,
    /// Candidate generated project directory.
    #[arg(long)]
    pub candidate: PathBuf,
    /// Optional baseline structured project model in JSON format.
    #[arg(long)]
    pub baseline_model: Option<PathBuf>,
    /// Optional candidate structured project model in JSON format.
    #[arg(long)]
    pub candidate_model: Option<PathBuf>,
    /// Optional baseline validation metadata in JSON format.
    #[arg(long)]
    pub baseline_metadata: Option<PathBuf>,
    /// Optional candidate validation metadata in JSON format.
    #[arg(long)]
    pub candidate_metadata: Option<PathBuf>,
    /// Directory that will receive per-package and comparison reports.
    #[arg(long)]
    pub output: PathBuf,
}
