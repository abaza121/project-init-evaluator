use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::Value;

static NEXT_COMPARISON_ID: AtomicU64 = AtomicU64::new(0);

/// Owns one isolated baseline-comparison fixture.
struct ComparisonWorkspace {
    root: PathBuf,
}

impl ComparisonWorkspace {
    /// Creates a unique workspace below the operating system temporary directory.
    fn new() -> Result<Self, Box<dyn Error>> {
        let id = NEXT_COMPARISON_ID.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "project-initiation-comparison-e2e-{}-{id}",
            std::process::id()
        ));
        fs::create_dir_all(&root)?;
        Ok(Self { root })
    }

    /// Returns an absolute path below this workspace.
    fn path(&self, relative: impl AsRef<Path>) -> PathBuf {
        self.root.join(relative)
    }

    /// Writes a UTF-8 fixture and creates its parent directories.
    fn write(&self, relative: impl AsRef<Path>, content: &str) -> Result<PathBuf, Box<dyn Error>> {
        let path = self.path(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, content)?;
        Ok(path)
    }
}

impl Drop for ComparisonWorkspace {
    /// Removes only the uniquely named temporary workspace owned by this test.
    fn drop(&mut self) {
        if self.root.starts_with(std::env::temp_dir()) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
}

/// Runs the comparison CLI for one baseline/candidate ordering.
fn run_compare(
    brief: &Path,
    baseline: &Path,
    candidate: &Path,
    output: &Path,
) -> Result<std::process::Output, Box<dyn Error>> {
    Ok(
        Command::new(env!("CARGO_BIN_EXE_project-initiation-evaluator"))
            .args(["compare", "--brief"])
            .arg(brief)
            .arg("--baseline")
            .arg(baseline)
            .arg("--candidate")
            .arg(candidate)
            .arg("--output")
            .arg(output)
            .output()?,
    )
}

/// Reads one generated validation report as generic JSON.
fn read_report(path: impl AsRef<Path>) -> Result<Value, Box<dyn Error>> {
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}

/// Proves comparison evaluates packages independently and remains symmetric when inputs swap.
#[test]
fn compare_command_partitions_reports_and_is_order_independent() -> Result<(), Box<dyn Error>> {
    let workspace = ComparisonWorkspace::new()?;
    let brief = workspace.write(
        "brief.md",
        "REQ-001: The tool must work offline.\nREQ-002: The tool must be documented.",
    )?;
    workspace.write(
        "baseline/Decisions.md",
        "# Decisions\nADR-001: BASELINE_ONLY uses mandatory cloud persistence.",
    )?;
    workspace.write(
        "candidate/Foundation.md",
        "# Requirements\nREQ-001: The tool works offline.\nREQ-002: The tool is documented.\n# Acceptance Criteria\nAC-001 supports REQ-001.\nAC-002 supports REQ-002.\n# Decisions\nADR-002: CANDIDATE_ONLY local module supports REQ-001 and REQ-002.\nThe component responsibility and data flow are documented.",
    )?;
    workspace.write("candidate/README.md", "[Foundation](Foundation.md)")?;
    let first_output = workspace.path("comparison-one");

    let first = run_compare(
        &brief,
        &workspace.path("baseline"),
        &workspace.path("candidate"),
        &first_output,
    )?;
    assert!(
        first.status.success(),
        "{}",
        String::from_utf8_lossy(&first.stderr)
    );
    let first_baseline = read_report(first_output.join("baseline/validation-report.json"))?;
    let first_candidate = read_report(first_output.join("candidate/validation-report.json"))?;
    let comparison = fs::read_to_string(first_output.join("comparison-report.md"))?;
    assert!(comparison.contains("| Metric | Baseline | Candidate | Change |"));
    assert!(comparison.contains("Cross-Project Leakage"));
    assert!(!first_baseline.to_string().contains("CANDIDATE_ONLY"));
    assert!(!first_candidate.to_string().contains("BASELINE_ONLY"));

    let second_output = workspace.path("comparison-two");
    let second = run_compare(
        &brief,
        &workspace.path("candidate"),
        &workspace.path("baseline"),
        &second_output,
    )?;
    assert!(
        second.status.success(),
        "{}",
        String::from_utf8_lossy(&second.stderr)
    );
    let second_baseline = read_report(second_output.join("baseline/validation-report.json"))?;
    let second_candidate = read_report(second_output.join("candidate/validation-report.json"))?;
    assert_eq!(first_baseline["drpfs"], second_candidate["drpfs"]);
    assert_eq!(first_candidate["drpfs"], second_baseline["drpfs"]);
    Ok(())
}
