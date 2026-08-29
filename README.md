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

## Commands

```powershell
cargo run -- evaluate --help
cargo run -- compare --help
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

The complete behavior contract and design rationale are in [`Docs/SPECIFICATION.md`](Docs/SPECIFICATION.md).
