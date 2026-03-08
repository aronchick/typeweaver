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
    assert!(stdout.contains("bench <font-id>"));
    assert!(stdout.contains("ingest --help"));
    assert!(stdout.contains("bench --help"));
}

#[test]
fn ingest_help_is_successful_and_mentions_registry_root() {
    let output = run_cli(&["ingest", "--help"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("Usage:"));
    assert!(stdout.contains("ingest <dir>"));
    assert!(stdout.contains("--registry-root <dir>"));
}

#[test]
fn bench_help_is_successful_and_mentions_profiles_and_out_path() {
    let output = run_cli(&["bench", "--help"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("Usage:"));
    assert!(stdout.contains("--profile <slug>"));
    assert!(stdout.contains("web_light_default|mobile_dark_low_contrast"));
    assert!(stdout.contains("--out <file>"));
}
