# Project Initiation Evaluator

`project-initiation-evaluator` is an independent, read-only grader for project-foundation packages. It implements the fixed Decision-Ready Project Foundation Score (DRPFS) rubric in [`init.md`](init.md) and is being delivered as verified incremental slices.

## Evaluate One Package

```powershell
cargo run -- evaluate `
  --brief .\path\to\ORIGINAL_BRIEF.md `
  --generated .\path\to\generated-project `
  --output .\reports
```

Optional inputs are accepted with `--project-model` and `--metadata`. The command writes `validation-report.json` and `validation-report.md` without modifying submitted artifacts.

## Compare Baseline and Candidate

```powershell
cargo run -- compare `
  --brief .\path\to\ORIGINAL_BRIEF.md `
  --baseline .\path\to\baseline `
  --candidate .\path\to\candidate `
  --output .\comparison
```

The command evaluates both packages independently, writes their reports below `comparison\baseline` and `comparison\candidate`, and then writes `comparison-report.md`. Optional per-side models and metadata use the `--baseline-model`, `--candidate-model`, `--baseline-metadata`, and `--candidate-metadata` flags.

## Commands

```powershell
cargo run -- evaluate --help
cargo run -- compare --help
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

The complete behavior contract and design rationale are in [`Docs/SPECIFICATION.md`](Docs/SPECIFICATION.md).

## Evaluation Boundaries

- Supported generated artifacts are UTF-8 Markdown, text, JSON, TOML, and YAML files.
- Default limits are 10,000 supported artifacts, 20,000 discovered entries, 2 MiB per input file, and 32 MiB for the collected package.
- Symlinks are recorded as skipped and never followed.
- Deterministic token overlap discovers coverage and plausibility candidates; it does not provide language-model semantic equivalence and can miss low-vocabulary paraphrases.
- External citation correctness is not independently verified. Reports assess submitted citation identity and decision linkage and state this scope explicitly.
- LanceDB and external model APIs are not required. A future semantic retrieval adapter can be added without changing deterministic metric arithmetic.
