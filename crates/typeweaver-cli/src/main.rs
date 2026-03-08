use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use typeweaver_bench::run_report;
use typeweaver_core::{AssetStatus, BenchmarkProfile, REPORTS_DIR_NAME};
use typeweaver_registry::{find_asset, ingest_dir, load_registry_at, save_registry_at};

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    if args.is_empty() || is_help_flag(&args[0]) {
        println!("{}", usage());
        return Ok(());
    }

    match args[0].as_str() {
        "ingest" => handle_ingest(&args[1..]),
        "bench" => handle_bench(&args[1..]),
        _ => Err(usage()),
    }
}

fn handle_ingest(args: &[String]) -> Result<(), String> {
    if args.first().is_some_and(|arg| is_help_flag(arg)) {
        println!("{}", ingest_usage());
        return Ok(());
    }

    if args.is_empty() {
        return Err("ingest requires <dir>\n".to_string() + &usage());
    }

    let source_dir = Path::new(&args[0]);
    let registry_root = parse_flag_value(args, "--registry-root")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(".typeweaver"));

    let registry = ingest_dir(source_dir).map_err(|e| e.to_string())?;
    let registry_path = save_registry_at(&registry_root, &registry).map_err(|e| e.to_string())?;

    let approved = registry
        .assets
        .iter()
        .filter(|a| a.status == AssetStatus::Approved)
        .count();
    let rejected = registry
        .assets
        .iter()
        .filter(|a| a.status == AssetStatus::Rejected)
        .count();
    let quarantined = registry
        .assets
        .iter()
        .filter(|a| a.status == AssetStatus::Quarantined)
        .count();

    println!(
        "ingested={} approved={} rejected={} quarantined={} registry={}",
        registry.assets.len(),
        approved,
        rejected,
        quarantined,
        registry_path.display()
    );

    Ok(())
}

fn handle_bench(args: &[String]) -> Result<(), String> {
    if args.first().is_some_and(|arg| is_help_flag(arg)) {
        println!("{}", bench_usage());
        return Ok(());
    }

    if args.is_empty() {
        return Err("bench requires <font-id>\n".to_string() + &usage());
    }

    let font_id = &args[0];
    let profile_raw = parse_flag_value(args, "--profile")
        .ok_or_else(|| "bench requires --profile <slug>".to_string())?;
    let profile = BenchmarkProfile::from_slug(&profile_raw).map_err(|e| e.to_string())?;

    let registry_root = parse_flag_value(args, "--registry-root")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(".typeweaver"));

    let registry = load_registry_at(&registry_root).map_err(|e| e.to_string())?;
    let asset = find_asset(&registry, font_id).map_err(|e| e.to_string())?;

    let report = run_report(asset, Some(profile));
    let output_path = parse_flag_value(args, "--out")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            registry_root
                .join(REPORTS_DIR_NAME)
                .join(format!("{}-{}.json", font_id, profile.as_str()))
        });

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(&output_path, report.to_json_pretty()).map_err(|e| e.to_string())?;

    println!("report={}", output_path.display());
    Ok(())
}

fn parse_flag_value(args: &[String], name: &str) -> Option<String> {
    args.iter()
        .position(|arg| arg == name)
        .and_then(|idx| args.get(idx + 1).cloned())
}

fn is_help_flag(value: &str) -> bool {
    matches!(value, "--help" | "-h" | "help")
}

fn usage() -> String {
    [
        "Usage:",
        "  cargo run -p typeweaver-cli -- ingest <dir> [--registry-root <dir>]",
        "  cargo run -p typeweaver-cli -- bench <font-id> --profile <web_light_default|mobile_dark_low_contrast> [--registry-root <dir>] [--out <file>]",
        "",
        "Use subcommand help:",
        "  cargo run -p typeweaver-cli -- ingest --help",
        "  cargo run -p typeweaver-cli -- bench --help",
    ]
    .join("\n")
}

fn ingest_usage() -> String {
    [
        "Usage:",
        "  cargo run -p typeweaver-cli -- ingest <dir> [--registry-root <dir>]",
        "",
        "Arguments:",
        "  <dir>                  Directory containing local font files",
        "",
        "Options:",
        "  --registry-root <dir>  Output root for registry.json (default: .typeweaver)",
    ]
    .join("\n")
}

fn bench_usage() -> String {
    [
        "Usage:",
        "  cargo run -p typeweaver-cli -- bench <font-id> --profile <web_light_default|mobile_dark_low_contrast> [--registry-root <dir>] [--out <file>]",
        "",
        "Arguments:",
        "  <font-id>              Font id from registry.json",
        "",
        "Options:",
        "  --profile <slug>       Benchmark profile slug",
        "  --registry-root <dir>  Root containing registry.json and reports/ (default: .typeweaver)",
        "  --out <file>           Optional explicit report output path",
    ]
    .join("\n")
}
