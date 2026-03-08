use std::process::Command;

fn run_cli(args: &[&str]) -> std::process::Output {
    let bin = env!("CARGO_BIN_EXE_typeweaver-cli");
    Command::new(bin)
        .args(args)
        .output()
        .expect("cli process should run")
}

#[test]
fn root_help_is_successful_and_mentions_subcommands() {
    let output = run_cli(&["--help"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("Usage:"));
    assert!(stdout.contains("ingest <dir>"));
    assert!(stdout.contains("list"));
    assert!(stdout.contains("profiles"));
    assert!(stdout.contains("bench <font-id>"));
}

#[test]
fn ingest_help_is_successful_and_mentions_registry_layout() {
    let output = run_cli(&["ingest", "--help"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("Usage:"));
    assert!(stdout.contains("ingest <dir>"));
    assert!(stdout.contains("registry/registry.json"));
}

#[test]
fn list_help_is_successful() {
    let output = run_cli(&["list", "--help"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("Usage:"));
    assert!(stdout.contains("list [--registry-root <dir>]"));
}

#[test]
fn profiles_help_is_successful() {
    let output = run_cli(&["profiles", "--help"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("Usage:"));
    assert!(stdout.contains("profiles"));
}

#[test]
fn bench_help_is_successful_and_mentions_output_structure() {
    let output = run_cli(&["bench", "--help"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("Usage:"));
    assert!(stdout.contains("--profile <slug>"));
    assert!(stdout.contains("web_light_default|mobile_dark_low_contrast"));
    assert!(stdout.contains("<root>/reports/<font-id>/<profile>/report.json"));
    assert!(stdout.contains("preview.txt"));
}
