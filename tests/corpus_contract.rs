use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use project_initiation_evaluator::{CorpusLimits, EvaluatorError, PackageInput, collect_corpus};

static NEXT_WORKSPACE_ID: AtomicU64 = AtomicU64::new(0);

/// Owns an isolated temporary directory used by one filesystem contract test.
struct TestWorkspace {
    root: PathBuf,
}

impl TestWorkspace {
    /// Creates a unique workspace below the operating system temporary directory.
    fn new() -> Result<Self, Box<dyn Error>> {
        let id = NEXT_WORKSPACE_ID.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "project-initiation-evaluator-{}-{id}",
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

    /// Creates the standard valid input paths for this workspace.
    fn input(&self) -> Result<PackageInput, Box<dyn Error>> {
        let brief = self.write("brief.md", "Build a local evaluator.")?;
        let generated_dir = self.path("generated");
        fs::create_dir_all(&generated_dir)?;
        Ok(PackageInput {
            brief,
            generated_dir,
            project_model: None,
            metadata: None,
            package_id: "test-package".to_owned(),
            output_dir: self.path("reports"),
        })
    }
}

impl Drop for TestWorkspace {
    /// Removes only the uniquely named temporary workspace owned by this test.
    fn drop(&mut self) {
        if self.root.starts_with(std::env::temp_dir()) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
}

/// Proves supported artifacts are collected once in stable relative-path order.
#[test]
fn corpus_collects_supported_files_in_deterministic_order() -> Result<(), Box<dyn Error>> {
    let workspace = TestWorkspace::new()?;
    let input = workspace.input()?;
    workspace.write("generated/zeta.txt", "last")?;
    workspace.write("generated/nested/alpha.md", "first")?;
    workspace.write("generated/ignored.bin", "ignored")?;

    let corpus = collect_corpus(&input, CorpusLimits::default())?;
    let paths = corpus
        .artifacts
        .iter()
        .map(|artifact| artifact.relative_path.to_string_lossy().replace('\\', "/"))
        .collect::<Vec<_>>();

    assert_eq!(paths, ["nested/alpha.md", "zeta.txt"]);
    assert_eq!(corpus.skipped_files, [PathBuf::from("ignored.bin")]);
    Ok(())
}

/// Proves the configured per-artifact byte boundary accepts the limit and rejects its neighbor.
#[test]
fn corpus_enforces_exact_artifact_byte_boundary_without_mutation() -> Result<(), Box<dyn Error>> {
    let workspace = TestWorkspace::new()?;
    let input = workspace.input()?;
    fs::write(&input.brief, "brief")?;
    let artifact = workspace.write("generated/requirements.md", "12345")?;
    let limits = CorpusLimits {
        max_artifact_bytes: 5,
        ..CorpusLimits::default()
    };

    let corpus = collect_corpus(&input, limits)?;
    assert_eq!(corpus.artifacts.len(), 1);
    assert_eq!(fs::read_to_string(&artifact)?, "12345");

    fs::write(&artifact, "123456")?;
    let error = collect_corpus(&input, limits).unwrap_err();
    assert!(matches!(error, EvaluatorError::ArtifactTooLarge { .. }));
    assert_eq!(fs::read_to_string(&artifact)?, "123456");
    Ok(())
}

/// Proves malformed optional JSON rejects atomically and a corrected retry succeeds.
#[test]
fn corpus_rejects_malformed_model_then_recovers_after_correction() -> Result<(), Box<dyn Error>> {
    let workspace = TestWorkspace::new()?;
    let mut input = workspace.input()?;
    let model = workspace.write("PROJECT_MODEL.json", "{not-json")?;
    input.project_model = Some(model.clone());

    let error = collect_corpus(&input, CorpusLimits::default()).unwrap_err();
    assert!(matches!(error, EvaluatorError::InvalidJson { .. }));
    assert!(!input.output_dir.exists());

    fs::write(&model, r#"{"requirements": []}"#)?;
    let corpus = collect_corpus(&input, CorpusLimits::default())?;
    assert!(corpus.project_model.is_some());
    assert!(!input.output_dir.exists());
    Ok(())
}

/// Proves artifact capacity accepts the exact count and rejects the next supported file.
#[test]
fn corpus_enforces_exact_artifact_count_boundary() -> Result<(), Box<dyn Error>> {
    let workspace = TestWorkspace::new()?;
    let input = workspace.input()?;
    workspace.write("generated/one.md", "one")?;
    workspace.write("generated/two.md", "two")?;
    let limits = CorpusLimits {
        max_artifacts: 2,
        ..CorpusLimits::default()
    };

    assert_eq!(collect_corpus(&input, limits)?.artifacts.len(), 2);
    workspace.write("generated/three.md", "three")?;
    let error = collect_corpus(&input, limits).unwrap_err();
    assert!(matches!(error, EvaluatorError::TooManyArtifacts { .. }));
    Ok(())
}

/// Proves aggregate capacity counts the brief and generated artifacts together.
#[test]
fn corpus_enforces_exact_total_byte_boundary() -> Result<(), Box<dyn Error>> {
    let workspace = TestWorkspace::new()?;
    let input = workspace.input()?;
    fs::write(&input.brief, "brief")?;
    let artifact = workspace.write("generated/requirements.md", "12345")?;
    let limits = CorpusLimits {
        max_artifact_bytes: 10,
        max_total_bytes: 10,
        ..CorpusLimits::default()
    };

    assert_eq!(collect_corpus(&input, limits)?.artifacts.len(), 1);
    fs::write(artifact, "123456")?;
    let error = collect_corpus(&input, limits).unwrap_err();
    assert!(matches!(error, EvaluatorError::PackageTooLarge { .. }));
    Ok(())
}

/// Proves report output cannot be placed inside the submitted package tree.
#[test]
fn corpus_rejects_output_that_overlaps_generated_package() -> Result<(), Box<dyn Error>> {
    let workspace = TestWorkspace::new()?;
    let mut input = workspace.input()?;
    input.output_dir = input.generated_dir.join("reports");

    let error = collect_corpus(&input, CorpusLimits::default()).unwrap_err();
    assert!(matches!(error, EvaluatorError::UnsafeOutputOverlap { .. }));
    assert!(!input.output_dir.exists());
    Ok(())
}

/// Proves missing required paths are rejected before any output is created.
#[test]
fn corpus_rejects_missing_brief_without_partial_output() -> Result<(), Box<dyn Error>> {
    let workspace = TestWorkspace::new()?;
    let mut input = workspace.input()?;
    input.brief = workspace.path("missing.md");

    let error = collect_corpus(&input, CorpusLimits::default()).unwrap_err();
    assert!(matches!(error, EvaluatorError::InvalidPath { .. }));
    assert!(!input.output_dir.exists());
    Ok(())
}

/// Proves skipped files and directories remain bounded at the exact discovered-entry limit.
#[test]
fn corpus_enforces_exact_discovered_entry_boundary() -> Result<(), Box<dyn Error>> {
    let workspace = TestWorkspace::new()?;
    let input = workspace.input()?;
    workspace.write("generated/one.bin", "one")?;
    workspace.write("generated/two.bin", "two")?;
    let limits = CorpusLimits {
        max_discovered_entries: 2,
        ..CorpusLimits::default()
    };

    assert_eq!(collect_corpus(&input, limits)?.skipped_files.len(), 2);
    workspace.write("generated/three.bin", "three")?;
    let error = collect_corpus(&input, limits).unwrap_err();
    assert!(matches!(error, EvaluatorError::TooManyEntries { .. }));
    Ok(())
}
