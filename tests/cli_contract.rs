use std::process::Command;

/// Confirms the command shell advertises both required evaluation modes.
#[test]
fn help_lists_evaluate_and_compare_subcommands() {
    let output = Command::new(env!("CARGO_BIN_EXE_project-initiation-evaluator"))
        .arg("--help")
        .output()
        .expect("the test binary should be runnable");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success());
    assert!(stdout.contains("evaluate"));
    assert!(stdout.contains("compare"));
}
