use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::error::EvaluatorError;
use crate::input::PackageInput;

/// Defines inclusive resource bounds for one package collection operation.
#[derive(Clone, Copy, Debug)]
pub struct CorpusLimits {
    /// Maximum supported generated artifacts.
    pub max_artifacts: usize,
    /// Maximum files, directories, and symlinks visited during discovery.
    pub max_discovered_entries: usize,
    /// Maximum UTF-8 bytes in any brief, artifact, model, or metadata file.
    pub max_artifact_bytes: usize,
    /// Maximum combined UTF-8 bytes across all collected inputs.
    pub max_total_bytes: usize,
}

impl Default for CorpusLimits {
    /// Returns conservative defaults suitable for project documentation packages.
    fn default() -> Self {
        Self {
            max_artifacts: 10_000,
            max_discovered_entries: 20_000,
            max_artifact_bytes: 2 * 1024 * 1024,
            max_total_bytes: 32 * 1024 * 1024,
        }
    }
}

/// Stores one evaluator-owned copy of a submitted UTF-8 source artifact.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Artifact {
    /// Stable package-relative path, or `ORIGINAL_BRIEF` for the brief.
    pub relative_path: PathBuf,
    /// Submitted UTF-8 content copied without modification.
    pub content: String,
    /// Stable FNV-1a content hash encoded as lowercase hexadecimal.
    pub content_hash: String,
}

/// Contains isolated source material collected for one package evaluation.
#[derive(Clone, Debug)]
pub struct Corpus {
    /// Stable identity supplied for this package.
    pub package_id: String,
    /// Original user brief retained separately as the fidelity authority.
    pub brief: Artifact,
    /// Supported generated artifacts in deterministic relative-path order.
    pub artifacts: Vec<Artifact>,
    /// Parsed optional project model.
    pub project_model: Option<Value>,
    /// Parsed optional validation metadata.
    pub metadata: Option<Value>,
    /// Unsupported files and symlinks skipped in deterministic path order.
    pub skipped_files: Vec<PathBuf>,
}

/// Validates all inputs and builds a bounded read-only corpus atomically.
pub fn collect_corpus(
    input: &PackageInput,
    limits: CorpusLimits,
) -> Result<Corpus, EvaluatorError> {
    validate_input(input)?;
    let mut total_bytes = 0usize;
    let brief = read_artifact(
        &input.brief,
        PathBuf::from("ORIGINAL_BRIEF"),
        limits,
        &mut total_bytes,
    )?;
    let project_model = read_optional_json(&input.project_model, limits, &mut total_bytes)?;
    let metadata = read_optional_json(&input.metadata, limits, &mut total_bytes)?;
    let (artifacts, mut skipped_files) =
        collect_generated(&input.generated_dir, limits, &mut total_bytes)?;
    skipped_files.sort();

    Ok(Corpus {
        package_id: input.package_id.trim().to_owned(),
        brief,
        artifacts,
        project_model,
        metadata,
        skipped_files,
    })
}

/// Validates path types, package identity, and output isolation before reading content.
fn validate_input(input: &PackageInput) -> Result<(), EvaluatorError> {
    require_regular_file(&input.brief)?;
    require_directory(&input.generated_dir)?;
    if input.package_id.trim().is_empty() {
        return Err(EvaluatorError::EmptyPackageId);
    }
    for path in [&input.project_model, &input.metadata]
        .into_iter()
        .flatten()
    {
        require_regular_file(path)?;
    }

    let generated = comparison_path(&input.generated_dir)?;
    let output = comparison_path(&input.output_dir)?;
    if output.starts_with(&generated) {
        return Err(EvaluatorError::UnsafeOutputOverlap { output, generated });
    }
    Ok(())
}

/// Requires a non-symlink regular file at the supplied path.
fn require_regular_file(path: &Path) -> Result<(), EvaluatorError> {
    let metadata = fs::symlink_metadata(path).map_err(|_| EvaluatorError::InvalidPath {
        path: path.to_path_buf(),
        expected: "a regular file",
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(EvaluatorError::InvalidPath {
            path: path.to_path_buf(),
            expected: "a regular file",
        });
    }
    Ok(())
}

/// Requires a non-symlink directory at the supplied path.
fn require_directory(path: &Path) -> Result<(), EvaluatorError> {
    let metadata = fs::symlink_metadata(path).map_err(|_| EvaluatorError::InvalidPath {
        path: path.to_path_buf(),
        expected: "a directory",
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(EvaluatorError::InvalidPath {
            path: path.to_path_buf(),
            expected: "a directory",
        });
    }
    Ok(())
}

/// Converts a possibly relative path into a lexical absolute path.
fn absolute_path(path: &Path) -> Result<PathBuf, EvaluatorError> {
    if path.is_absolute() {
        return Ok(path.to_path_buf());
    }
    std::env::current_dir()
        .map(|current| current.join(path))
        .map_err(|source| EvaluatorError::Io {
            path: path.to_path_buf(),
            source,
        })
}

/// Canonicalizes the nearest existing ancestor and restores missing descendants.
fn comparison_path(path: &Path) -> Result<PathBuf, EvaluatorError> {
    let absolute = absolute_path(path)?;
    let mut ancestor = absolute.as_path();
    let mut missing = Vec::new();
    while !ancestor.exists() {
        let name = ancestor
            .file_name()
            .ok_or_else(|| EvaluatorError::InvalidPath {
                path: path.to_path_buf(),
                expected: "a resolvable path",
            })?;
        missing.push(name.to_os_string());
        ancestor = ancestor
            .parent()
            .ok_or_else(|| EvaluatorError::InvalidPath {
                path: path.to_path_buf(),
                expected: "a resolvable path",
            })?;
    }

    let mut resolved = fs::canonicalize(ancestor).map_err(|source| EvaluatorError::Io {
        path: ancestor.to_path_buf(),
        source,
    })?;
    for component in missing.into_iter().rev() {
        resolved.push(component);
    }
    Ok(resolved)
}

/// Reads and parses one optional JSON input after applying corpus limits.
fn read_optional_json(
    path: &Option<PathBuf>,
    limits: CorpusLimits,
    total_bytes: &mut usize,
) -> Result<Option<Value>, EvaluatorError> {
    let Some(path) = path else {
        return Ok(None);
    };
    let artifact = read_artifact(path, path.clone(), limits, total_bytes)?;
    serde_json::from_str(&artifact.content)
        .map(Some)
        .map_err(|source| EvaluatorError::InvalidJson {
            path: path.clone(),
            source,
        })
}

/// Collects supported generated files without following symlinks.
fn collect_generated(
    root: &Path,
    limits: CorpusLimits,
    total_bytes: &mut usize,
) -> Result<(Vec<Artifact>, Vec<PathBuf>), EvaluatorError> {
    let (mut files, skipped) = discover_files(root, limits)?;
    files.sort();

    let mut artifacts = Vec::with_capacity(files.len());
    for path in files {
        let relative = path.strip_prefix(root).unwrap_or(&path).to_path_buf();
        artifacts.push(read_artifact(&path, relative, limits, total_bytes)?);
    }
    Ok((artifacts, skipped))
}

/// Discovers regular files with an explicit directory stack and bounded retained state.
fn discover_files(
    root: &Path,
    limits: CorpusLimits,
) -> Result<(Vec<PathBuf>, Vec<PathBuf>), EvaluatorError> {
    let mut directories = vec![root.to_path_buf()];
    let mut files = Vec::new();
    let mut skipped = Vec::new();
    let mut discovered = 0usize;
    while let Some(directory) = directories.pop() {
        let mut entries = read_sorted_directory(&directory)?;
        for entry in entries.drain(..) {
            discovered = discovered
                .checked_add(1)
                .ok_or(EvaluatorError::TooManyEntries {
                    limit: limits.max_discovered_entries,
                })?;
            if discovered > limits.max_discovered_entries {
                return Err(EvaluatorError::TooManyEntries {
                    limit: limits.max_discovered_entries,
                });
            }
            let path = entry.path();
            let relative = path.strip_prefix(root).unwrap_or(&path).to_path_buf();
            let file_type = entry.file_type().map_err(|source| EvaluatorError::Io {
                path: path.clone(),
                source,
            })?;
            if file_type.is_symlink() {
                skipped.push(relative);
            } else if file_type.is_dir() {
                directories.push(path);
            } else if file_type.is_file() && is_supported(&path) {
                files.push(path);
                if files.len() > limits.max_artifacts {
                    return Err(EvaluatorError::TooManyArtifacts {
                        limit: limits.max_artifacts,
                    });
                }
            } else {
                skipped.push(relative);
            }
        }
    }
    Ok((files, skipped))
}

/// Reads one directory into deterministic file-name order with path-aware errors.
fn read_sorted_directory(directory: &Path) -> Result<Vec<fs::DirEntry>, EvaluatorError> {
    let mut entries = fs::read_dir(directory)
        .map_err(|source| EvaluatorError::Io {
            path: directory.to_path_buf(),
            source,
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|source| EvaluatorError::Io {
            path: directory.to_path_buf(),
            source,
        })?;
    entries.sort_by_key(|entry| entry.file_name());
    Ok(entries)
}

/// Recognizes textual project-artifact extensions supported by the evaluator.
fn is_supported(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| {
            matches!(
                extension.to_ascii_lowercase().as_str(),
                "md" | "markdown" | "txt" | "json" | "toml" | "yaml" | "yml"
            )
        })
        .unwrap_or(false)
}

/// Reads one UTF-8 artifact and updates aggregate capacity only after validation.
fn read_artifact(
    path: &Path,
    relative_path: PathBuf,
    limits: CorpusLimits,
    total_bytes: &mut usize,
) -> Result<Artifact, EvaluatorError> {
    let metadata = fs::metadata(path).map_err(|source| EvaluatorError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    let declared_len = usize::try_from(metadata.len()).unwrap_or(usize::MAX);
    if declared_len > limits.max_artifact_bytes {
        return Err(EvaluatorError::ArtifactTooLarge {
            path: path.to_path_buf(),
            limit: limits.max_artifact_bytes,
        });
    }
    total_bytes
        .checked_add(declared_len)
        .filter(|total| *total <= limits.max_total_bytes)
        .ok_or(EvaluatorError::PackageTooLarge {
            limit: limits.max_total_bytes,
        })?;
    let bytes = fs::read(path).map_err(|source| EvaluatorError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    if bytes.len() > limits.max_artifact_bytes {
        return Err(EvaluatorError::ArtifactTooLarge {
            path: path.to_path_buf(),
            limit: limits.max_artifact_bytes,
        });
    }
    let next_total = total_bytes
        .checked_add(bytes.len())
        .filter(|total| *total <= limits.max_total_bytes)
        .ok_or(EvaluatorError::PackageTooLarge {
            limit: limits.max_total_bytes,
        })?;
    let content = String::from_utf8(bytes).map_err(|_| EvaluatorError::InvalidUtf8 {
        path: path.to_path_buf(),
    })?;
    *total_bytes = next_total;
    Ok(Artifact {
        content_hash: stable_hash(content.as_bytes()),
        relative_path,
        content,
    })
}

/// Produces a stable dependency-free FNV-1a hash for evidence identity.
fn stable_hash(bytes: &[u8]) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}
