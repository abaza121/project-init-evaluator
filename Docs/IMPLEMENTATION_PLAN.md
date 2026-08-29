# Implementation Plan: Project Initiation Evaluator

## Overview

Initialize a stable Rust CLI and library, then deliver the evaluator as narrow green slices. Each slice adds tests first, implements only the tested contract, updates `CHANGELOG.md` when behavior becomes user-visible, passes targeted checks, and ends in an atomic commit.

## Architecture Decisions

- Keep the binary thin and expose evaluation through library request/report types so integration tests and future adapters do not shell out unnecessarily.
- Build an evaluator-owned corpus in deterministic path order and never follow symlinks.
- Separate extraction, deterministic metrics, and qualitative rubric scoring so forbidden similarity-based arithmetic cannot leak into authoritative metrics.
- Use conservative, evidence-backed rules. Missing evidence reduces confidence or yields unavailable metrics; it never becomes invented data.
- Serialize one normalized report model to both required formats to prevent JSON/Markdown score drift.

## Task List

### Phase 1: Foundation

- [x] Task 1: Initialize repository documentation and specification
  - Acceptance: `init.md`, the local instructions, changelog rules, this specification, and this plan are versioned on `main`.
  - Verify: inspect staged diff and Git log; no generated/build files are present.
  - Files: `init.md`, `AGENTS.md`, `Docs/CHANGELOG_GUIDELINES.md`, `Docs/SPECIFICATION.md`, `Docs/IMPLEMENTATION_PLAN.md`.
  - Dependencies: none.

- [x] Task 2: Initialize a buildable crate and documented CLI shell
  - Acceptance: Cargo metadata, library/binary entry points, README, `.gitignore`, and `CHANGELOG.md` exist; `--help` exposes `evaluate` and `compare` without implementing evaluation.
  - Verify: `cargo fmt --check`, `cargo check`, `cargo test`, and CLI help.
  - Files: `Cargo.toml`, `Cargo.lock`, `.gitignore`, `src/lib.rs`, `src/main.rs`, `README.md`, `CHANGELOG.md`.
  - Dependencies: Task 1.

### Checkpoint: Foundation

- [x] The repository builds cleanly and its public contract is documented.

### Phase 2: Read-Only Evaluation Pipeline

- [x] Task 3: Validate inputs and collect a bounded deterministic corpus
  - Acceptance: valid text artifacts load in sorted order; missing/invalid paths, symlinks, unsupported files, and byte/file boundaries behave as specified; submitted files remain unchanged.
  - Verify: targeted corpus/input unit tests and full quality suite.
  - Files: `src/error.rs`, `src/input.rs`, `src/corpus.rs`, `src/lib.rs`, `CHANGELOG.md`.
  - Dependencies: Task 2.

- [x] Task 4: Extract evaluator-owned entities and authoritative metrics
  - Acceptance: requirements, decisions, assumptions, evidence, questions, IDs, links, and traces retain file/line evidence; all calculable metrics use authoritative records and unavailable values remain null.
  - Verify: extraction/metric adversarial unit tests including malformed metadata, duplicate IDs, and broken links.
  - Files: `src/extract.rs`, `src/metrics.rs`, `src/input.rs`, `src/lib.rs`, `CHANGELOG.md`.
  - Dependencies: Task 3.

### Checkpoint: Deterministic Core

- [x] Corpus and metrics tests pass at exact boundaries and after rejected input.

### Phase 3: Scoring and Reports

- [x] Task 5: Score the seven fixed rubric dimensions
  - Acceptance: dimension maxima are immutable; totals are normalized; findings are evidence-backed and complete; special failures are classified conservatively.
  - Verify: score boundary, no-evidence, contradiction, assumption-promotion, trace-theater, and invariant tests.
  - Files: `src/score.rs`, `src/report.rs`, `src/lib.rs`, `CHANGELOG.md`.
  - Dependencies: Task 4.

- [x] Task 6: Render and atomically write JSON and Markdown reports
  - Acceptance: both formats contain consistent scores/metrics and all required sections; unsuccessful evaluation leaves no final report pair.
  - Verify: renderer snapshots/structural assertions and single-package integration test.
  - Files: `src/report.rs`, `src/lib.rs`, `tests/evaluate_cli.rs`, `CHANGELOG.md`.
  - Dependencies: Task 5.

### Checkpoint: Single-Package Evaluation

- [x] A representative package produces schema-valid JSON and complete Markdown without modifying inputs.

### Phase 4: Comparison and Hardening

- [x] Task 7: Add isolated baseline/candidate comparison
  - Acceptance: both packages are evaluated independently, outputs are partitioned, changes are calculated consistently, unavailable telemetry remains explicit, and no cross-project leakage occurs.
  - Verify: order-independence and package-isolation integration tests.
  - Files: `src/compare.rs`, `src/lib.rs`, `src/main.rs`, `tests/compare_cli.rs`, `CHANGELOG.md`.
  - Dependencies: Task 6.

- [ ] Task 8: Complete documentation, adversarial verification, and review fixes
  - Acceptance: README examples run; all public/private/test items meet commenting rules; the full suite and realistic-scale checks pass; review findings are resolved or explicitly deferred.
  - Verify: formatting, check, all tests, Clippy with warnings denied, help smoke tests, Git diff/history review.
  - Files: `README.md`, test fixtures, and narrowly required review fixes.
  - Dependencies: Task 7.

### Checkpoint: Complete

- [ ] All acceptance criteria in `Docs/SPECIFICATION.md` are met.
- [ ] Every notable behavior is represented once under `CHANGELOG.md` → `Unreleased`.
- [ ] Each commit is atomic, green, and free of secrets/build output.

## Risks and Mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| Lexical matches overstate semantic preservation | High | Treat matches as candidates, require shared meaningful tokens/evidence, and score ambiguity conservatively. |
| Free-form model/metadata schemas vary | High | Traverse JSON generically, recognize explicit field aliases, and keep unavailable metrics null. |
| Large or cyclic packages exhaust resources | High | Enforce limits, never follow symlinks, sort entries, and test exact boundaries. |
| JSON and Markdown disagree | Medium | Render both from the same normalized report and assert equality in integration tests. |
| Baseline data leaks into candidate evaluation | Critical | Build separate request/corpus/extraction values and add reversed-order isolation tests. |
| Commit history captures broken states | Medium | Commit only after targeted and baseline quality commands pass. |

## Verification Cadence

After every task: targeted tests, `cargo fmt --check`, `cargo check`, staged-diff review, and secret scan. After every checkpoint and before completion: full `cargo test` and `cargo clippy --all-targets --all-features -- -D warnings`.
