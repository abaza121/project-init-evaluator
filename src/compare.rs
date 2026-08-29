use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::corpus::CorpusLimits;
use crate::error::EvaluatorError;
use crate::evaluate::{build_validation_report, write_validation_reports};
use crate::input::PackageInput;
use crate::report::{DimensionScore, ValidationReport};

static NEXT_COMPARISON_WRITE_ID: AtomicU64 = AtomicU64::new(0);

/// Identifies the shared brief and isolated package inputs for one comparison.
#[derive(Clone, Debug)]
pub struct ComparisonInput {
    /// Original user brief shared by baseline and candidate evaluations.
    pub brief: PathBuf,
    /// Baseline generated project directory.
    pub baseline_dir: PathBuf,
    /// Candidate generated project directory.
    pub candidate_dir: PathBuf,
    /// Optional baseline structured project model.
    pub baseline_model: Option<PathBuf>,
    /// Optional candidate structured project model.
    pub candidate_model: Option<PathBuf>,
    /// Optional baseline validation and execution metadata.
    pub baseline_metadata: Option<PathBuf>,
    /// Optional candidate validation and execution metadata.
    pub candidate_metadata: Option<PathBuf>,
    /// Root directory for partitioned package reports and the comparison report.
    pub output_dir: PathBuf,
}

/// Contains both independent reports and their rendered comparison.
#[derive(Clone, Debug)]
pub struct ComparisonResult {
    /// Independently calculated baseline report.
    pub baseline: ValidationReport,
    /// Independently calculated candidate report.
    pub candidate: ValidationReport,
    /// Markdown comparison derived only after both reports were complete.
    pub markdown: String,
}

/// Evaluates both packages independently, writes partitioned reports, and compares results.
pub fn compare_packages(
    input: &ComparisonInput,
    limits: CorpusLimits,
) -> Result<ComparisonResult, EvaluatorError> {
    let baseline_input = package_input(input, true);
    let candidate_input = package_input(input, false);
    let baseline = build_validation_report(&baseline_input, limits)?;
    let candidate = build_validation_report(&candidate_input, limits)?;
    let markdown = render_comparison_markdown(&baseline, &candidate);

    write_validation_reports(&baseline_input.output_dir, &baseline)?;
    write_validation_reports(&candidate_input.output_dir, &candidate)?;
    write_comparison_report(&input.output_dir, &markdown)?;
    Ok(ComparisonResult {
        baseline,
        candidate,
        markdown,
    })
}

/// Converts one side of a shared comparison request into an isolated package input.
fn package_input(input: &ComparisonInput, baseline: bool) -> PackageInput {
    let (generated_dir, project_model, metadata, package_id, output_name) = if baseline {
        (
            input.baseline_dir.clone(),
            input.baseline_model.clone(),
            input.baseline_metadata.clone(),
            "baseline",
            "baseline",
        )
    } else {
        (
            input.candidate_dir.clone(),
            input.candidate_model.clone(),
            input.candidate_metadata.clone(),
            "candidate",
            "candidate",
        )
    };
    PackageInput {
        brief: input.brief.clone(),
        generated_dir,
        project_model,
        metadata,
        package_id: package_id.to_owned(),
        output_dir: input.output_dir.join(output_name),
    }
}

/// Renders the required independent metric comparison and resource-difference disclosure.
pub fn render_comparison_markdown(
    baseline: &ValidationReport,
    candidate: &ValidationReport,
) -> String {
    let mut output = String::new();
    output.push_str("# Comparison Report\n\n");
    output.push_str(
        "Both packages were evaluated independently before this comparison was calculated.\n\n",
    );
    output.push_str("| Metric | Baseline | Candidate | Change |\n|---|---:|---:|---:|\n");
    append_number_row(&mut output, "DRPFS", baseline.drpfs, candidate.drpfs, "");
    append_dimension_row(
        &mut output,
        "Brief Fidelity",
        &baseline.dimensions.brief_fidelity,
        &candidate.dimensions.brief_fidelity,
    );
    append_dimension_row(
        &mut output,
        "Assumption Discipline",
        &baseline.dimensions.assumption_discipline,
        &candidate.dimensions.assumption_discipline,
    );
    append_dimension_row(
        &mut output,
        "Consistency",
        &baseline.dimensions.cross_document_consistency,
        &candidate.dimensions.cross_document_consistency,
    );
    append_dimension_row(
        &mut output,
        "Evidence Quality",
        &baseline.dimensions.evidence_quality,
        &candidate.dimensions.evidence_quality,
    );
    append_dimension_row(
        &mut output,
        "Traceability",
        &baseline.dimensions.traceability,
        &candidate.dimensions.traceability,
    );
    append_dimension_row(
        &mut output,
        "Actionability",
        &baseline.dimensions.actionability,
        &candidate.dimensions.actionability,
    );
    append_dimension_row(
        &mut output,
        "Artifact Completeness",
        &baseline.dimensions.artifact_quality,
        &candidate.dimensions.artifact_quality,
    );
    append_optional_number_row(
        &mut output,
        "Unsupported Decision Rate",
        baseline.metrics.unsupported_decision_rate,
        candidate.metrics.unsupported_decision_rate,
        "%",
    );
    append_optional_number_row(
        &mut output,
        "Traceability Coverage",
        baseline.metrics.requirement_traceability_coverage,
        candidate.metrics.requirement_traceability_coverage,
        "%",
    );
    append_optional_number_row(
        &mut output,
        "Duplicate Question Rate",
        baseline.metrics.duplicate_question_rate,
        candidate.metrics.duplicate_question_rate,
        "%",
    );
    append_optional_number_row(
        &mut output,
        "Answer Reuse Rate",
        baseline.metrics.answer_reuse_rate,
        candidate.metrics.answer_reuse_rate,
        "%",
    );
    append_optional_number_row(
        &mut output,
        "Repeated Research Rate",
        baseline.metrics.repeated_research_rate,
        candidate.metrics.repeated_research_rate,
        "%",
    );
    append_optional_u64_row(
        &mut output,
        "Context Supplied",
        baseline.metrics.context_supplied,
        candidate.metrics.context_supplied,
    );
    append_optional_u64_row(
        &mut output,
        "Retrieval Calls",
        baseline.metrics.retrieval_calls,
        candidate.metrics.retrieval_calls,
    );
    append_optional_number_row(
        &mut output,
        "Stale Retrieval Rate",
        baseline.metrics.stale_retrieval_rate,
        candidate.metrics.stale_retrieval_rate,
        "%",
    );
    append_optional_usize_row(
        &mut output,
        "Cross-Project Leakage",
        baseline.metrics.cross_project_leakage,
        candidate.metrics.cross_project_leakage,
    );
    output.push_str("\n## Resource Differences\n\n| Resource | Baseline | Candidate | Change |\n|---|---:|---:|---:|\n");
    append_optional_number_row(
        &mut output,
        "Execution Time (seconds)",
        baseline.metrics.execution_time_seconds,
        candidate.metrics.execution_time_seconds,
        "",
    );
    append_optional_number_row(
        &mut output,
        "Human Interaction Time (seconds)",
        baseline.metrics.human_interaction_time_seconds,
        candidate.metrics.human_interaction_time_seconds,
        "",
    );
    append_optional_u64_row(
        &mut output,
        "User Questions",
        baseline.metrics.user_questions,
        candidate.metrics.user_questions,
    );
    append_optional_u64_row(
        &mut output,
        "Model Calls",
        baseline.metrics.model_calls,
        candidate.metrics.model_calls,
    );
    append_optional_u64_row(
        &mut output,
        "Token Usage",
        baseline.metrics.token_usage,
        candidate.metrics.token_usage,
    );
    append_optional_number_row(
        &mut output,
        "Cost",
        baseline.metrics.cost,
        candidate.metrics.cost,
        "",
    );
    output.push_str("\nResource and retrieval metrics are diagnostic. A better retrieval metric does not prove that the final project foundation is better, and DRPFS differences should be interpreted alongside resource differences.\n");
    output
}

/// Appends one fixed dimension score with its maximum.
fn append_dimension_row(
    output: &mut String,
    label: &str,
    baseline: &DimensionScore,
    candidate: &DimensionScore,
) {
    output.push_str(&format!(
        "| {label} | {:.1}/{} | {:.1}/{} | {:+.1} |\n",
        baseline.score,
        baseline.max,
        candidate.score,
        candidate.max,
        candidate.score - baseline.score
    ));
}

/// Appends one always-available numeric metric row.
fn append_number_row(
    output: &mut String,
    label: &str,
    baseline: f64,
    candidate: f64,
    suffix: &str,
) {
    output.push_str(&format!(
        "| {label} | {baseline:.1}{suffix} | {candidate:.1}{suffix} | {:+.1}{suffix} |\n",
        candidate - baseline
    ));
}

/// Appends one optional floating-point metric without substituting zero.
fn append_optional_number_row(
    output: &mut String,
    label: &str,
    baseline: Option<f64>,
    candidate: Option<f64>,
    suffix: &str,
) {
    let baseline_text = format_optional_number(baseline, suffix);
    let candidate_text = format_optional_number(candidate, suffix);
    let change = match (baseline, candidate) {
        (Some(baseline), Some(candidate)) => format!("{:+.1}{suffix}", candidate - baseline),
        _ => "Unavailable".to_owned(),
    };
    output.push_str(&format!(
        "| {label} | {baseline_text} | {candidate_text} | {change} |\n"
    ));
}

/// Appends one optional 64-bit count without substituting zero.
fn append_optional_u64_row(
    output: &mut String,
    label: &str,
    baseline: Option<u64>,
    candidate: Option<u64>,
) {
    let baseline_text =
        baseline.map_or_else(|| "Unavailable".to_owned(), |value| value.to_string());
    let candidate_text =
        candidate.map_or_else(|| "Unavailable".to_owned(), |value| value.to_string());
    let change = match (baseline, candidate) {
        (Some(baseline), Some(candidate)) => format_signed_count(candidate, baseline),
        _ => "Unavailable".to_owned(),
    };
    output.push_str(&format!(
        "| {label} | {baseline_text} | {candidate_text} | {change} |\n"
    ));
}

/// Appends one optional platform-sized count without substituting zero.
fn append_optional_usize_row(
    output: &mut String,
    label: &str,
    baseline: Option<usize>,
    candidate: Option<usize>,
) {
    let baseline = baseline.and_then(|value| u64::try_from(value).ok());
    let candidate = candidate.and_then(|value| u64::try_from(value).ok());
    append_optional_u64_row(output, label, baseline, candidate);
}

/// Formats one optional floating value for a comparison cell.
fn format_optional_number(value: Option<f64>, suffix: &str) -> String {
    value.map_or_else(
        || "Unavailable".to_owned(),
        |value| format!("{value:.1}{suffix}"),
    )
}

/// Formats an exact signed difference between two unsigned counts without overflow.
fn format_signed_count(candidate: u64, baseline: u64) -> String {
    if candidate >= baseline {
        format!("+{}", candidate - baseline)
    } else {
        format!("-{}", baseline - candidate)
    }
}

/// Writes the comparison Markdown through a unique staged sibling file.
fn write_comparison_report(output_dir: &Path, markdown: &str) -> Result<(), EvaluatorError> {
    fs::create_dir_all(output_dir).map_err(|source| io_error(output_dir, source))?;
    let id = NEXT_COMPARISON_WRITE_ID.fetch_add(1, Ordering::Relaxed);
    let final_path = output_dir.join("comparison-report.md");
    let temp_path = output_dir.join(format!(
        ".comparison-report.md.{}.{id}.tmp",
        std::process::id()
    ));
    fs::write(&temp_path, markdown).map_err(|source| io_error(&temp_path, source))?;
    if final_path.exists() {
        fs::remove_file(&final_path).map_err(|source| io_error(&final_path, source))?;
    }
    fs::rename(&temp_path, &final_path).map_err(|source| io_error(&final_path, source))
}

/// Wraps one comparison-output filesystem failure with its exact path.
fn io_error(path: &Path, source: std::io::Error) -> EvaluatorError {
    EvaluatorError::Io {
        path: path.to_path_buf(),
        source,
    }
}
