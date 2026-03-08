use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use typeweaver_bench::run_report;
use typeweaver_core::{AssetStatus, BenchmarkProfile, REGISTRY_FILE_NAME, REPORTS_DIR_NAME};
use typeweaver_registry::{ingest_dir, load_registry_at, save_registry_at};

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
        other => Err(format!("unknown command '{other}'\n\n{}", usage())),
    }
}

fn handle_ingest(args: &[String]) -> Result<(), String> {
    if args.first().is_some_and(|arg| is_help_flag(arg)) {
        println!("{}", ingest_usage());
        return Ok(());
    }

    if args.is_empty() {
        return Err("ingest requires <dir>\n\n".to_string() + &ingest_usage());
    }
    if args[0].starts_with('-') {
        return Err("ingest requires a directory path as the first argument".to_string());
    }

    let source_dir = Path::new(&args[0]);
    if !source_dir.exists() {
        return Err(format!(
            "ingest directory does not exist: {}",
            source_dir.display()
        ));
    }
    if !source_dir.is_dir() {
        return Err(format!(
            "ingest path is not a directory: {}",
            source_dir.display()
        ));
    }

    let registry_root = parse_ingest_registry_root(args)?;

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
        return Err("bench requires <font-id>\n\n".to_string() + &bench_usage());
    }
    if args[0].starts_with('-') {
        return Err("bench requires a font id as the first argument".to_string());
    }

    let font_id = &args[0];
    let bench_opts = parse_bench_options(args)?;
    let profile_raw = bench_opts.profile.ok_or_else(|| {
        "bench requires --profile <slug> (web_light_default or mobile_dark_low_contrast)"
            .to_string()
    })?;
    let profile = BenchmarkProfile::from_slug(&profile_raw)
        .map_err(|e| format!("{e}. use --help for supported profile slugs"))?;

    let registry_root = bench_opts.registry_root;

    let registry_path = registry_root.join(REGISTRY_FILE_NAME);
    let registry = load_registry_at(&registry_root).map_err(|e| {
        format!(
            "{e}. expected a registry at {}",
            registry_path.to_string_lossy()
        )
    })?;
    let asset = registry
        .assets
        .iter()
        .find(|asset| asset.id == *font_id)
        .ok_or_else(|| {
            let mut available = registry
                .assets
                .iter()
                .map(|asset| asset.id.as_str())
                .take(5)
                .collect::<Vec<_>>();
            if available.is_empty() {
                available.push("<none>");
            }
            format!(
                "font id '{}' not found in {}. available ids: {}",
                font_id,
                registry_path.display(),
                available.join(", ")
            )
        })?;

    let report = run_report(asset, Some(profile));
    let output_path = bench_opts.out.unwrap_or_else(|| {
            registry_root.join(REPORTS_DIR_NAME).join(format!(
                "{}-{}.json",
                font_id,
                profile.as_str()
            ))
        });

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(&output_path, report.to_json_pretty()).map_err(|e| e.to_string())?;

    println!("report={}", output_path.display());
    Ok(())
}

fn parse_ingest_registry_root(args: &[String]) -> Result<PathBuf, String> {
    let mut idx = 1usize;
    let mut registry_root = PathBuf::from(".typeweaver");
    while idx < args.len() {
        match args[idx].as_str() {
            "--registry-root" => {
                let value = args
                    .get(idx + 1)
                    .ok_or_else(|| "missing value for --registry-root".to_string())?;
                registry_root = PathBuf::from(value);
                idx += 2;
            }
            other if other.starts_with('-') => {
                return Err(format!("unknown option for ingest: {other}"));
            }
            other => {
                return Err(format!("unexpected argument for ingest: {other}"));
            }
        }
    }
    Ok(registry_root)
}

struct BenchOptions {
    profile: Option<String>,
    registry_root: PathBuf,
    out: Option<PathBuf>,
}

fn parse_bench_options(args: &[String]) -> Result<BenchOptions, String> {
    let mut idx = 1usize;
    let mut profile = None;
    let mut registry_root = PathBuf::from(".typeweaver");
    let mut out = None;
    while idx < args.len() {
        match args[idx].as_str() {
            "--profile" => {
                let value = args
                    .get(idx + 1)
                    .ok_or_else(|| "missing value for --profile".to_string())?;
                profile = Some(value.clone());
                idx += 2;
            }
            "--registry-root" => {
                let value = args
                    .get(idx + 1)
                    .ok_or_else(|| "missing value for --registry-root".to_string())?;
                registry_root = PathBuf::from(value);
                idx += 2;
            }
            "--out" => {
                let value = args
                    .get(idx + 1)
                    .ok_or_else(|| "missing value for --out".to_string())?;
                out = Some(PathBuf::from(value));
                idx += 2;
            }
            other if other.starts_with('-') => return Err(format!("unknown option for bench: {other}")),
            other => return Err(format!("unexpected argument for bench: {other}")),
        }
    }

    Ok(BenchOptions {
        profile,
        registry_root,
        out,
    })
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
        "  --profile <slug>       Benchmark profile slug (required)",
        "  --registry-root <dir>  Root containing registry.json and reports/ (default: .typeweaver)",
        "  --out <file>           Optional explicit report output path (default: <registry-root>/reports/<font-id>-<profile>.json)",
    ]
    .join("\n")
}
