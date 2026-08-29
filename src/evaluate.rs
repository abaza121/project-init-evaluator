use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::corpus::{CorpusLimits, collect_corpus};
use crate::error::EvaluatorError;
use crate::extract::extract_entities;
use crate::input::PackageInput;
use crate::metrics::calculate_metrics;
use crate::report::{ValidationReport, render_validation_json, render_validation_markdown};
use crate::score::score_corpus;

static NEXT_REPORT_WRITE_ID: AtomicU64 = AtomicU64::new(0);

/// Builds a normalized report without writing any output files.
pub fn build_validation_report(
    input: &PackageInput,
    limits: CorpusLimits,
) -> Result<ValidationReport, EvaluatorError> {
    let corpus = collect_corpus(input, limits)?;
    let extraction = extract_entities(&corpus);
    let metrics = calculate_metrics(&corpus, &extraction);
    Ok(score_corpus(&corpus, &extraction, metrics))
}

/// Evaluates one package and writes the required report pair after successful scoring.
pub fn evaluate_package(
    input: &PackageInput,
    limits: CorpusLimits,
) -> Result<ValidationReport, EvaluatorError> {
    let report = build_validation_report(input, limits)?;
    write_validation_reports(&input.output_dir, &report)?;
    Ok(report)
}

/// Renders and transactionally replaces the JSON and Markdown validation reports.
pub fn write_validation_reports(
    output_dir: &Path,
    report: &ValidationReport,
) -> Result<(), EvaluatorError> {
    let json = render_validation_json(report)
        .map_err(|source| EvaluatorError::Serialization { source })?;
    let markdown = render_validation_markdown(report);
    fs::create_dir_all(output_dir).map_err(|source| io_error(output_dir, source))?;
    let id = NEXT_REPORT_WRITE_ID.fetch_add(1, Ordering::Relaxed);
    let json_final = output_dir.join("validation-report.json");
    let markdown_final = output_dir.join("validation-report.md");
    let json_temp = temporary_path(output_dir, "validation-report.json", id, "tmp");
    let markdown_temp = temporary_path(output_dir, "validation-report.md", id, "tmp");
    fs::write(&json_temp, json).map_err(|source| io_error(&json_temp, source))?;
    if let Err(error) = fs::write(&markdown_temp, markdown) {
        remove_if_present(&json_temp);
        return Err(io_error(&markdown_temp, error));
    }
    replace_report_pair(&json_temp, &json_final, &markdown_temp, &markdown_final, id)
}

/// Replaces both final reports and restores the previous pair after any promotion failure.
fn replace_report_pair(
    json_temp: &Path,
    json_final: &Path,
    markdown_temp: &Path,
    markdown_final: &Path,
    id: u64,
) -> Result<(), EvaluatorError> {
    let output_dir = json_final.parent().unwrap_or_else(|| Path::new("."));
    let json_backup = temporary_path(output_dir, "validation-report.json", id, "bak");
    let markdown_backup = temporary_path(output_dir, "validation-report.md", id, "bak");
    let json_had_backup = backup_existing(json_final, &json_backup)?;
    let markdown_had_backup = match backup_existing(markdown_final, &markdown_backup) {
        Ok(existed) => existed,
        Err(error) => {
            restore_backup(&json_backup, json_final, json_had_backup);
            remove_if_present(json_temp);
            remove_if_present(markdown_temp);
            return Err(error);
        }
    };

    if let Err(source) = fs::rename(json_temp, json_final) {
        restore_backup(&json_backup, json_final, json_had_backup);
        restore_backup(&markdown_backup, markdown_final, markdown_had_backup);
        remove_if_present(markdown_temp);
        return Err(io_error(json_final, source));
    }
    if let Err(source) = fs::rename(markdown_temp, markdown_final) {
        remove_if_present(json_final);
        restore_backup(&json_backup, json_final, json_had_backup);
        restore_backup(&markdown_backup, markdown_final, markdown_had_backup);
        return Err(io_error(markdown_final, source));
    }
    remove_if_present(&json_backup);
    remove_if_present(&markdown_backup);
    Ok(())
}

/// Moves an existing report aside and returns whether a backup was created.
fn backup_existing(final_path: &Path, backup_path: &Path) -> Result<bool, EvaluatorError> {
    if !final_path.exists() {
        return Ok(false);
    }
    fs::rename(final_path, backup_path).map_err(|source| io_error(final_path, source))?;
    Ok(true)
}

/// Restores a previously moved report when a pair replacement fails.
fn restore_backup(backup: &Path, final_path: &Path, existed: bool) {
    if existed {
        let _ = fs::rename(backup, final_path);
    }
}

/// Builds a unique sibling path for staged or backup report content.
fn temporary_path(output_dir: &Path, name: &str, id: u64, suffix: &str) -> PathBuf {
    output_dir.join(format!(".{name}.{}.{id}.{suffix}", std::process::id()))
}

/// Removes one exact temporary report path when it exists.
fn remove_if_present(path: &Path) {
    if path.exists() {
        let _ = fs::remove_file(path);
    }
}

/// Wraps one filesystem failure with its exact target path.
fn io_error(path: &Path, source: std::io::Error) -> EvaluatorError {
    EvaluatorError::Io {
        path: path.to_path_buf(),
        source,
    }
}
