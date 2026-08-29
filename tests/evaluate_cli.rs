use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::Value;

static NEXT_EVALUATION_ID: AtomicU64 = AtomicU64::new(0);

/// Owns one isolated end-to-end evaluation fixture.
struct EvaluationWorkspace {
    root: PathBuf,
}

impl EvaluationWorkspace {
    /// Creates a unique workspace below the operating system temporary directory.
    fn new() -> Result<Self, Box<dyn Error>> {
        let id = NEXT_EVALUATION_ID.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "project-initiation-evaluator-e2e-{}-{id}",
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

impl Drop for EvaluationWorkspace {
    /// Removes only the uniquely named temporary workspace owned by this test.
    fn drop(&mut self) {
        if self.root.starts_with(std::env::temp_dir()) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
}

/// Proves the real CLI writes consistent required reports without modifying submitted inputs.
#[test]
fn evaluate_command_writes_json_and_markdown_reports_read_only() -> Result<(), Box<dyn Error>> {
    let workspace = EvaluationWorkspace::new()?;
    let brief = workspace.write("brief.md", "REQ-001: The tool must work offline.")?;
    let requirements = workspace.write(
        "generated/Requirements.md",
        "# Requirements\nREQ-001: The tool works offline.\n# Acceptance Criteria\nAC-001 for REQ-001: Runs without a network.",
    )?;
    workspace.write(
        "generated/Architecture.md",
        "# Decisions\nADR-001: Use a local module; supports REQ-001.\nThe component responsibility and data flow are documented.",
    )?;
    workspace.write("generated/README.md", "[Requirements](Requirements.md)")?;
    let output_dir = workspace.path("reports");
    let original_brief = fs::read(&brief)?;
    let original_requirements = fs::read(&requirements)?;

    let output = Command::new(env!("CARGO_BIN_EXE_project-initiation-evaluator"))
        .args(["evaluate", "--brief"])
        .arg(&brief)
        .arg("--generated")
        .arg(workspace.path("generated"))
        .arg("--package-id")
        .arg("candidate")
        .arg("--output")
        .arg(&output_dir)
        .output()?;

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json_path = output_dir.join("validation-report.json");
    let markdown_path = output_dir.join("validation-report.md");
    let report: Value = serde_json::from_slice(&fs::read(json_path)?)?;
    let markdown = fs::read_to_string(markdown_path)?;
    let dimension_total = report["dimensions"]
        .as_object()
        .into_iter()
        .flat_map(|dimensions| dimensions.values())
        .filter_map(|dimension| dimension["score"].as_f64())
        .sum::<f64>();

    assert_eq!(report["drpfs"].as_f64(), Some(dimension_total));
    assert!(markdown.contains("# Validation Report"));
    assert!(markdown.contains("## Deterministic Metrics"));
    assert!(markdown.contains("## Detailed Findings"));
    assert!(markdown.contains("## Recommended Improvements"));
    assert_eq!(fs::read(brief)?, original_brief);
    assert_eq!(fs::read(requirements)?, original_requirements);
    Ok(())
}

/// Proves invalid optional JSON fails before either final report is created.
#[test]
fn evaluate_command_rejects_invalid_model_without_partial_reports() -> Result<(), Box<dyn Error>> {
    let workspace = EvaluationWorkspace::new()?;
    let brief = workspace.write("brief.md", "REQ-001: The tool must work offline.")?;
    workspace.write("generated/Requirements.md", "REQ-001: Work offline.")?;
    let model = workspace.write("PROJECT_MODEL.json", "{invalid")?;
    let output_dir = workspace.path("reports");

    let output = Command::new(env!("CARGO_BIN_EXE_project-initiation-evaluator"))
        .args(["evaluate", "--brief"])
        .arg(brief)
        .arg("--generated")
        .arg(workspace.path("generated"))
        .arg("--project-model")
        .arg(model)
        .arg("--output")
        .arg(&output_dir)
        .output()?;

    assert!(!output.status.success());
    assert!(!output_dir.join("validation-report.json").exists());
    assert!(!output_dir.join("validation-report.md").exists());
    Ok(())
}
