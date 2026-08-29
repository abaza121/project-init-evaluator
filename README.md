# Project Initiation Evaluator

`project-initiation-evaluator` is an independent, read-only grader for project-foundation packages. It implements the fixed Decision-Ready Project Foundation Score (DRPFS) rubric in [`init.md`](init.md) and is being delivered as verified incremental slices.

## Status

The repository currently exposes the final command-line shape. Evaluation and report generation are added by the subsequent milestones in [`Docs/IMPLEMENTATION_PLAN.md`](Docs/IMPLEMENTATION_PLAN.md).

## Commands

```powershell
cargo run -- evaluate --help
cargo run -- compare --help
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

The complete behavior contract and design rationale are in [`Docs/SPECIFICATION.md`](Docs/SPECIFICATION.md).
