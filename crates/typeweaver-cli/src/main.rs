use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use typeweaver_bench::{
    default_preview_path, default_report_path, render_preview_text, run_report,
};
use typeweaver_core::{
    AssetStatus, BenchmarkProfile, PREVIEW_FILE_NAME, REGISTRY_DIR_NAME, REGISTRY_FILE_NAME,
};
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
        "list" => handle_list(&args[1..]),
        "profiles" => handle_profiles(&args[1..]),
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

    let registry_root = parse_registry_root_arg(args, 1, "ingest")?;
    let registry_dir = registry_dir_from_root(&registry_root);

    let registry = ingest_dir(source_dir).map_err(|e| e.to_string())?;
    let registry_path = save_registry_at(&registry_dir, &registry).map_err(|e| e.to_string())?;

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

fn handle_list(args: &[String]) -> Result<(), String> {
    if args.first().is_some_and(|arg| is_help_flag(arg)) {
        println!("{}", list_usage());
        return Ok(());
    }

    let registry_root = parse_registry_root_arg(args, 0, "list")?;
    let registry_dir = registry_dir_from_root(&registry_root);
    let registry = load_registry_at(&registry_dir).map_err(|e| {
        let expected = registry_dir.join(REGISTRY_FILE_NAME);
        format!("{e}. expected a registry at {}", expected.to_string_lossy())
    })?;

    if registry.assets.is_empty() {
        return Err(format!(
            "registry is empty at {}. run 'cargo run -- ingest <dir>' first",
            registry_dir.join(REGISTRY_FILE_NAME).display()
        ));
    }

    println!("font_id\tfamily\tstyle\tlicense\tstatus");
    for asset in &registry.assets {
        println!(
            "{}\t{}\t{}\t{}\t{}",
            asset.id,
            asset.family_name.as_deref().unwrap_or("-"),
            asset.style_name.as_deref().unwrap_or("-"),
            asset.license_normalized.as_str(),
            asset.status.as_str()
        );
    }

    Ok(())
}

fn handle_profiles(args: &[String]) -> Result<(), String> {
    if args.first().is_some_and(|arg| is_help_flag(arg)) {
        println!("{}", profiles_usage());
        return Ok(());
    }

    if !args.is_empty() {
        return Err(format!(
            "profiles does not accept positional arguments\n\n{}",
            profiles_usage()
        ));
    }

    for profile in BenchmarkProfile::all() {
        println!("{}\t{}", profile.as_str(), profile.description());
    }
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
        .map_err(|e| format!("{e}. run 'cargo run -- profiles' to list valid profiles"))?;

    let registry_dir = registry_dir_from_root(&bench_opts.registry_root);
    let registry_path = registry_dir.join(REGISTRY_FILE_NAME);

    let registry = load_registry_at(&registry_dir).map_err(|e| {
        format!(
            "{e}. expected a registry at {}",
            registry_path.to_string_lossy()
        )
    })?;

    if registry.assets.is_empty() {
        return Err(format!(
            "registry is empty at {}. run 'cargo run -- ingest <dir>' first",
            registry_path.display()
        ));
    }

    let asset = registry
        .assets
        .iter()
        .find(|candidate| candidate.id == *font_id)
        .ok_or_else(|| {
            let available = registry
                .assets
                .iter()
                .map(|candidate| candidate.id.as_str())
                .take(5)
                .collect::<Vec<_>>();
            format!(
                "font id '{}' not found in {}. available ids: {}",
                font_id,
                registry_path.display(),
                available.join(", ")
            )
        })?;

    let default_report_path = default_report_path(&bench_opts.registry_root, font_id, profile);
    let preview_path = default_preview_path(&bench_opts.registry_root, font_id, profile);

    if let Some(parent) = default_report_path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            format!(
                "failed to create report directory {}: {e}",
                parent.display()
            )
        })?;
    }

    let mut report = run_report(asset, profile);
    report.artifacts.report_path = Some(relative_report_path(font_id, profile));
    report.artifacts.preview_files = vec![PREVIEW_FILE_NAME.to_string()];

    fs::write(&default_report_path, report.to_json_pretty()).map_err(|e| {
        format!(
            "failed to write report JSON to {}: {e}",
            default_report_path.display()
        )
    })?;

    let preview_text = render_preview_text(asset);
    fs::write(&preview_path, preview_text).map_err(|e| {
        format!(
            "failed to write preview file {}: {e}",
            preview_path.display()
        )
    })?;

    if let Some(explicit_out) = bench_opts.out {
        validate_output_path(&explicit_out)?;
        if let Some(parent) = explicit_out.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                format!(
                    "failed to create output directory {}: {e}",
                    parent.display()
                )
            })?;
        }
        fs::write(&explicit_out, report.to_json_pretty()).map_err(|e| {
            format!(
                "failed to write report JSON to explicit --out path {}: {e}",
                explicit_out.display()
            )
        })?;
        println!(
            "report={} preview={} copied_to={}",
            default_report_path.display(),
            preview_path.display(),
            explicit_out.display()
        );
    } else {
        println!(
            "report={} preview={}",
            default_report_path.display(),
            preview_path.display()
        );
    }

    Ok(())
}

fn relative_report_path(font_id: &str, profile: BenchmarkProfile) -> String {
    format!("reports/{}/{}/report.json", font_id, profile.as_str())
}

fn validate_output_path(path: &Path) -> Result<(), String> {
    if path.as_os_str().is_empty() {
        return Err("invalid output path: value is empty".to_string());
    }
    if path.is_dir() {
        return Err(format!(
            "invalid output path: {} is a directory",
            path.display()
        ));
    }
    Ok(())
}

fn registry_dir_from_root(root: &Path) -> PathBuf {
    root.join(REGISTRY_DIR_NAME)
}

fn parse_registry_root_arg(
    args: &[String],
    start_index: usize,
    command_name: &str,
) -> Result<PathBuf, String> {
    let mut idx = start_index;
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
                return Err(format!("unknown option for {command_name}: {other}"));
            }
            other => {
                return Err(format!("unexpected argument for {command_name}: {other}"));
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
            other if other.starts_with('-') => {
                return Err(format!("unknown option for bench: {other}"));
            }
            other => {
                return Err(format!("unexpected argument for bench: {other}"));
            }
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
        "  cargo run -p typeweaver-cli -- list [--registry-root <dir>]",
        "  cargo run -p typeweaver-cli -- profiles",
        "  cargo run -p typeweaver-cli -- bench <font-id> --profile <web_light_default|mobile_dark_low_contrast> [--registry-root <dir>] [--out <file>]",
        "",
        "Use subcommand help:",
        "  cargo run -p typeweaver-cli -- ingest --help",
        "  cargo run -p typeweaver-cli -- list --help",
        "  cargo run -p typeweaver-cli -- profiles --help",
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
        "  --registry-root <dir>  Output root (default: .typeweaver)",
        "                         Registry file: <root>/registry/registry.json",
    ]
    .join("\n")
}

fn list_usage() -> String {
    [
        "Usage:",
        "  cargo run -p typeweaver-cli -- list [--registry-root <dir>]",
        "",
        "Options:",
        "  --registry-root <dir>  Registry root (default: .typeweaver)",
        "                         Reads from: <root>/registry/registry.json",
    ]
    .join("\n")
}

fn profiles_usage() -> String {
    [
        "Usage:",
        "  cargo run -p typeweaver-cli -- profiles",
        "",
        "Lists benchmark profiles and descriptions.",
    ]
    .join("\n")
}

fn bench_usage() -> String {
    [
        "Usage:",
        "  cargo run -p typeweaver-cli -- bench <font-id> --profile <web_light_default|mobile_dark_low_contrast> [--registry-root <dir>] [--out <file>]",
        "",
        "Arguments:",
        "  <font-id>              Font id from registry/registry.json",
        "",
        "Options:",
        "  --profile <slug>       Benchmark profile slug (required)",
        "  --registry-root <dir>  Root containing registry/ and reports/ (default: .typeweaver)",
        "  --out <file>           Optional additional report copy target",
        "",
        "Default artifact output:",
        "  <root>/reports/<font-id>/<profile>/report.json",
        "  <root>/reports/<font-id>/<profile>/preview.txt",
    ]
    .join("\n")
}
