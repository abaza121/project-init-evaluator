# Specification: Project Initiation Evaluator

## Objective

Build an independent, local Rust evaluator that grades a generated project-foundation package against its original user brief. The evaluator must inspect rather than repair the submitted package, produce `validation-report.json` and `validation-report.md`, and support an optional baseline-versus-candidate comparison that evaluates each package independently before comparing results.

Success means another developer can run the evaluator non-interactively, inspect every deduction back to submitted evidence, and obtain the fixed 100-point Decision-Ready Project Foundation Score (DRPFS) plus all calculable deterministic metrics from `init.md`.

## Acceptance Criteria

1. A single-package command accepts an original brief, generated project directory, optional project model, optional validation metadata, a package identifier, and an output directory.
2. A comparison command accepts one brief and baseline/candidate package inputs, evaluates both independently, writes their reports into separate output directories, and writes `comparison-report.md`.
3. Evaluation never modifies the brief, generated package, model, or metadata.
4. The seven dimension maxima remain exactly 20, 15, 15, 15, 15, 15, and 5; their sum and `drpfs` remain internally consistent.
5. Every negative finding includes severity, criterion, description, artifact/file, optional related identifier, supporting evidence, and recommended correction.
6. Metrics that cannot be calculated from authoritative input are JSON `null` and are described as unavailable in Markdown rather than guessed.
7. Artifact traversal is deterministic, rejects invalid input paths, ignores symlinks and evaluator output directories, and reads only supported text formats.
8. Reports explicitly surface assumption promotion, research-after-decision, decorative research, document drift, traceability theater, false completeness, over-engineering, under-specification, user override failure, and evidence misuse when evidence supports those classifications.
9. Report JSON matches the structure required by `init.md`; Markdown contains the required scorecard and detailed sections.
10. The repository passes formatting, compilation, tests, and Clippy with warnings denied.

## Non-Goals

- Repairing or rewriting evaluated artifacts.
- Trusting a generator-owned vector index as evaluation evidence.
- Requiring LanceDB, an external language model, network access, or API credentials.
- Treating lexical similarity as proof of preservation, contradiction, trace validity, or evidence support.
- Inventing unavailable resource, retrieval, or user-interaction telemetry.
- Publishing a release version or deploying the evaluator.

## Observable Contract

### Explicit Behavior

- The rubric and weights in `init.md` are immutable.
- Both JSON and Markdown reports are required for every evaluated package.
- Baseline and candidate scoring is isolated and order-independent.
- Only evidence-backed negative findings are emitted.
- Deterministic metrics are calculated from authoritative submitted data, never semantic similarity.
- External source correctness is not claimed unless independently verified; submitted citations are assessed for traceability and linkage only.

### Policy Decisions

- Semantic retrieval is optional, so the initial implementation uses transparent token-based candidate discovery and direct rule checks. Ambiguous semantic relationships are surfaced for review and scored conservatively.
- Supported artifact formats are UTF-8 `.md`, `.markdown`, `.txt`, `.json`, `.toml`, `.yaml`, and `.yml` files. Other files are listed as skipped rather than decoded speculatively.
- When validation metadata supplies required artifact names, question/answer telemetry, or unresolved finding data, it is authoritative for the applicable deterministic metric. Otherwise the metric is unavailable.
- Requirements are extracted from explicit list items, identified requirement records, and normative statements containing terms such as “must,” “should,” “need,” or “require.” Boilerplate and headings alone are not requirements.
- Candidate coverage requires meaningful normalized-token overlap. Negated candidate statements are contradiction candidates, but contradiction deductions require a shared subject and evidence excerpt.
- Fractional rubric calculations are rounded to one decimal point; `drpfs` is the one-decimal sum of dimension scores.

### Implementation Invariants

- Dimension scores are finite, non-negative, and never exceed their fixed maxima.
- The reported total equals the sum of dimension scores.
- Missing optional inputs never mutate or invalidate available evidence.
- A rejected input produces no partial final report.
- File order and finding order are stable across runs on unchanged inputs.
- Package identifiers and output roots partition baseline and candidate data completely.
- Every evidence reference names a collected source file and includes a bounded excerpt.
- Internal-link counts, artifact completion, acceptance-criteria coverage, traceability coverage, unsupported-decision rate, assumption-labeling rate, evidence-linkage rate, unresolved findings, and answer-adoption rate never use similarity scores as their arithmetic source.

### Evaluator Risks

- Paraphrases with little vocabulary overlap can be missed.
- The same vocabulary can express opposite or unrelated meaning.
- Free-form project-model schemas may not expose authoritative entities consistently.
- Markdown links can be anchors, external URLs, generated outputs, or intentionally unresolved placeholders.
- Very large packages can cause excessive memory use if every file is loaded without limits.
- Contradictions, assumptions, and evidence misuse need conservative thresholds to avoid unsupported findings.

Mitigations are bounded file/package sizes, stable extraction, evidence-rich findings, null unavailable metrics, schema-tolerant JSON traversal, and adversarial tests for negation, interleaving, malformed input, duplicate IDs, broken links, package isolation, and boundary sizes.

## Architecture and Data Flow

```text
CLI request
   -> validated package inputs
   -> deterministic artifact corpus
   -> extracted requirements/decisions/assumptions/evidence/traces
   -> authoritative deterministic metrics
   -> seven rubric dimension evaluators
   -> normalized validation report
   -> JSON + Markdown renderers

comparison request
   -> baseline evaluation (isolated)
   -> candidate evaluation (isolated)
   -> comparison-report.md
```

For a C# developer, the crate is comparable to a project/assembly. Public Rust modules are namespace-like boundaries with enforced privacy. Report structs are DTO-like types. Data-bearing enums are closed discriminated unions. `Result<T, E>` makes recoverable file, input, and rendering failures explicit instead of throwing exceptions. Functions borrow corpus data (`&Corpus`) when callers retain ownership and move report values only when ownership naturally transfers.

## Modules and Public Boundaries

- `cli`: declarative command-line DTOs and dispatch.
- `error`: one recoverable application error enum with path context.
- `input`: validated request types and optional metadata/model loading.
- `corpus`: deterministic, bounded, read-only artifact discovery.
- `extract`: normalized entities and evidence locations.
- `metrics`: authoritative deterministic metric calculations.
- `score`: seven dimension evaluators and invariant normalization.
- `report`: JSON schema types and Markdown rendering.
- `evaluate`: isolated single-package orchestration and transactional report-pair writing.
- `compare`: independent two-package orchestration and comparison rendering.

The library exposes request and report types plus `evaluate` and `compare`. The binary is a thin adapter that parses arguments, calls the library, and returns a non-zero exit code with a concise diagnostic on failure.

## Core Data Types

- `PackageRequest`: brief, generated directory, optional model/metadata, package ID, and output directory.
- `ComparisonRequest`: shared brief plus isolated baseline and candidate requests.
- `Artifact`: package-relative source path, content, hash, and format.
- `EvidenceRef`: file, line, bounded excerpt, and optional entity ID.
- `Requirement`, `Decision`, `Assumption`, `EvidenceClaim`, `TraceLink`, and `OpenQuestion`: normalized evaluator-owned entities.
- `MetricValue`: available percentage/count or unavailable reason.
- `Finding`: required negative-finding fields and critical-failure classification.
- `DimensionScore`: score, maximum, and findings.
- `ValidationReport`: DRPFS, dimensions, metrics, critical failures, strengths, and improvements.

## Error Handling and Limits

- Missing paths, non-directory package roots, malformed supplied JSON, unreadable UTF-8, unsafe output overlap, and limit exhaustion return typed errors with path context.
- Default limits: 2 MiB per input file, 32 MiB per package, 10,000 supported artifacts, and 20,000 discovered files/directories/symlinks. Exact-boundary tests cover accepted and rejected neighbors.
- Symlinks are not followed, preventing traversal outside the submitted directory and cycles.
- Reports are rendered in memory and written only after evaluation succeeds. Each target file is written through a sibling temporary file and renamed to reduce partial-output risk.

## CLI

```text
cargo run -- evaluate \
  --brief path/to/ORIGINAL_BRIEF.md \
  --generated path/to/generated-project \
  [--project-model path/to/PROJECT_MODEL.json] \
  [--metadata path/to/VALIDATION_METADATA.json] \
  [--package-id PACKAGE] \
  --output path/to/reports

cargo run -- compare \
  --brief path/to/ORIGINAL_BRIEF.md \
  --baseline path/to/baseline \
  --candidate path/to/candidate \
  [--baseline-model path] [--candidate-model path] \
  [--baseline-metadata path] [--candidate-metadata path] \
  --output path/to/comparison
```

## Technology and Dependencies

- Rust 1.98.0, stable edition 2024 (detected locally).
- `clap` 4.6 with `derive` for typed subcommands. Its documented derive API maps `Parser` and `Subcommand` to Rust structs/enums: https://docs.rs/clap/latest/clap/_derive/
- `serde` 1.0 with `derive` for stable report DTO serialization: https://serde.rs/derive.html
- `serde_json` 1.0 for tolerant JSON input and pretty report output: https://docs.rs/serde_json/latest/serde_json/fn.to_writer_pretty.html
- The standard library handles sorted recursive discovery. `std::fs::read_dir` does not guarantee ordering, so entries are explicitly sorted: https://doc.rust-lang.org/stable/std/fs/fn.read_dir.html

No other runtime or development dependencies are planned. Broad compatible semver requirements keep patch updates available; `Cargo.lock` pins the initialized application build.

## Commands

```text
Build:  cargo build
Run:    cargo run -- --help
Test:   cargo test
Format: cargo fmt --check
Check:  cargo check
Lint:   cargo clippy --all-targets --all-features -- -D warnings
```

## Project Structure

```text
src/                 library modules and thin binary
tests/               unit-contract, CLI, scale, and end-to-end black-box tests
Docs/                specification, plan, and changelog guidance
init.md              authoritative evaluator brief
CHANGELOG.md          Keep a Changelog unreleased history
```

## Code Style and Commenting

Every function, struct, and enum—including private and test items—has a behavior-oriented Rust documentation comment. Inline comments are reserved for non-obvious algorithms or functions longer than 18 lines.

```rust
/// Describes a submitted artifact without transferring ownership of its source text.
pub struct Artifact {
    pub relative_path: PathBuf,
    pub content: String,
}

/// Calculates metrics from authoritative extracted entities.
pub fn calculate_metrics(corpus: &Corpus) -> Metrics {
    // The caller retains the corpus; immutable borrowing avoids a clone.
    Metrics::from(corpus)
}
```

Production paths avoid `unwrap`, `expect`, and panics. Tests may use explicit helpers returning `Result` so test setup failures remain diagnostic.

## Testing Strategy

- Unit tests beside pure extraction, metric, scoring, and rendering modules.
- Integration tests in `tests/` for CLI contracts, output files, read-only package behavior, and baseline isolation.
- Temporary fixture directories use unique names and remove only their exact verified paths.
- Red-green development per vertical slice.
- Boundary families: zero/one/many artifacts, exact byte/file limits, malformed optional JSON between valid inputs, duplicate identifiers, missing metrics, negation, cross-package leakage, broken links, traceability theater, and stable output ordering.

## Boundaries

- Always: preserve evaluator independence, cite deductions, keep scoring bounded, update the changelog for notable behavior, run the quality suite before commits, and commit only green increments.
- Ask first: add an external service/API, persist data outside report outputs, change the fixed rubric, introduce a database, or publish a release.
- Never: modify evaluated packages, trust submitted embeddings as authoritative, invent unavailable values, follow symlinks, commit secrets/build output, or infer hidden implementation identity.

## Deliverables

- Rust crate with library and CLI.
- `validation-report.json` and `validation-report.md` for a single package.
- Per-package reports plus `comparison-report.md` for comparison mode.
- README usage documentation, fixtures/tests, specification, plan, changelog, and incremental Git history.

## Open Questions Resolved by Reversible Defaults

- No CLI syntax was supplied: use explicit subcommands and path flags.
- No required-artifact manifest schema was supplied: accept tolerant JSON metadata and return null when authoritative counts are unavailable.
- No external model or embedding provider was supplied: remain offline and deterministic.
- No release version was requested: keep all changes under `Unreleased`.
