use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn run_cli(args: &[&str]) -> std::process::Output {
    let bin = env!("CARGO_BIN_EXE_typeweaver-cli");
    Command::new(bin)
        .args(args)
        .output()
        .expect("cli process should run")
}

fn workspace_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(Path::parent)
        .expect("workspace root should exist")
        .to_path_buf()
}

fn temp_registry_root() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time should be monotonic")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("typeweaver-cli-test-{nonce}"));
    fs::create_dir_all(&dir).expect("temp registry root should be creatable");
    dir
}

fn extract_first_font_id(registry_json: &str) -> Option<String> {
    let marker = "\"id\": \"";
    let start = registry_json.find(marker)?;
    let rest = &registry_json[start + marker.len()..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

#[test]
fn ingest_and_bench_end_to_end_produces_report() {
    let root = workspace_root();
    let fonts_dir = root.join("fixtures/fonts");
    let registry_root = temp_registry_root();

    let ingest = run_cli(&[
        "ingest",
        &fonts_dir.to_string_lossy(),
        "--registry-root",
        &registry_root.to_string_lossy(),
    ]);
    assert!(ingest.status.success(), "ingest stderr: {:?}", ingest.stderr);

    let registry_path = registry_root.join("registry.json");
    let registry_raw = fs::read_to_string(&registry_path).expect("registry.json should exist");
    let font_id = extract_first_font_id(&registry_raw).expect("registry should contain one font id");

    let bench = run_cli(&[
        "bench",
        &font_id,
        "--profile",
        "web_light_default",
        "--registry-root",
        &registry_root.to_string_lossy(),
    ]);
    assert!(bench.status.success(), "bench stderr: {:?}", bench.stderr);

    let report_path = registry_root
        .join("reports")
        .join(format!("{font_id}-web_light_default.json"));
    assert!(report_path.exists(), "report path should exist");
    let report_raw = fs::read_to_string(report_path).expect("report should be readable");
    assert!(report_raw.contains("\"schema_id\": \"typeweaver.report_card.v1\""));

    let _ = fs::remove_dir_all(registry_root);
}

#[test]
fn ingest_fails_with_missing_directory() {
    let output = run_cli(&["ingest", "/tmp/typeweaver-does-not-exist"]);
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(stderr.contains("ingest directory does not exist"));
}

#[test]
fn bench_fails_with_bad_profile_slug() {
    let root = workspace_root();
    let fonts_dir = root.join("fixtures/fonts");
    let registry_root = temp_registry_root();

    let ingest = run_cli(&[
        "ingest",
        &fonts_dir.to_string_lossy(),
        "--registry-root",
        &registry_root.to_string_lossy(),
    ]);
    assert!(ingest.status.success(), "ingest stderr: {:?}", ingest.stderr);

    let registry_path = registry_root.join("registry.json");
    let registry_raw = fs::read_to_string(&registry_path).expect("registry.json should exist");
    let font_id = extract_first_font_id(&registry_raw).expect("registry should contain one font id");

    let output = run_cli(&[
        "bench",
        &font_id,
        "--profile",
        "bad_profile",
        "--registry-root",
        &registry_root.to_string_lossy(),
    ]);
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(stderr.contains("unsupported profile"));

    let _ = fs::remove_dir_all(registry_root);
}

#[test]
fn bench_fails_with_missing_font_id() {
    let root = workspace_root();
    let fonts_dir = root.join("fixtures/fonts");
    let registry_root = temp_registry_root();

    let ingest = run_cli(&[
        "ingest",
        &fonts_dir.to_string_lossy(),
        "--registry-root",
        &registry_root.to_string_lossy(),
    ]);
    assert!(ingest.status.success(), "ingest stderr: {:?}", ingest.stderr);

    let output = run_cli(&[
        "bench",
        "font-does-not-exist",
        "--profile",
        "web_light_default",
        "--registry-root",
        &registry_root.to_string_lossy(),
    ]);
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(stderr.contains("font id 'font-does-not-exist' not found"));

    let _ = fs::remove_dir_all(registry_root);
}
