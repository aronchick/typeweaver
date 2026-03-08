use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use typeweaver_core::{
    AssetStatus, FontAsset, NormalizedLicense, REGISTRY_FILE_NAME, Registry, escape_json,
};

#[derive(Debug)]
pub enum RegistryError {
    Io(io::Error),
    Parse(String),
    NotFound(String),
}

impl std::fmt::Display for RegistryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Parse(err) => write!(f, "parse error: {err}"),
            Self::NotFound(id) => write!(f, "font asset not found: {id}"),
        }
    }
}

impl From<io::Error> for RegistryError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

pub fn normalize_license(raw: Option<&str>) -> NormalizedLicense {
    let Some(raw) = raw else {
        return NormalizedLicense::Unknown;
    };

    let normalized = raw.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        return NormalizedLicense::Unknown;
    }

    if contains_any(
        &normalized,
        &[
            "ambiguous",
            "unsure",
            "unclear",
            "tbd",
            "unknown provenance",
            "unverified",
        ],
    ) {
        return NormalizedLicense::Ambiguous;
    }

    if contains_any(
        &normalized,
        &[
            "mixed license",
            "mixed",
            "dual license pack",
            "dual-license pack",
        ],
    ) {
        return NormalizedLicense::Mixed;
    }

    let has_pd = normalized.contains("public domain") || contains_token(&normalized, "pd");
    let has_cc0 = normalized.contains("cc0") || normalized.contains("creative commons zero");
    let has_mit = contains_token(&normalized, "mit") || normalized.contains("mit license");
    let has_apache = contains_token(&normalized, "apache")
        && contains_any(
            &normalized,
            &[
                "2.0",
                "version 2",
                "version 2.0",
                "apache-2",
                "apache 2",
                "apache-2.0",
            ],
        );
    let has_ofl = contains_token(&normalized, "ofl") || normalized.contains("open font license");
    let has_gpl = contains_license_family_token(&normalized, "gpl")
        || contains_license_family_token(&normalized, "lgpl")
        || contains_license_family_token(&normalized, "agpl");

    let approved_count = [has_pd, has_cc0, has_mit, has_apache]
        .into_iter()
        .filter(|v| *v)
        .count();
    let rejected_count = [has_ofl, has_gpl].into_iter().filter(|v| *v).count();

    if approved_count + rejected_count > 1 {
        return NormalizedLicense::Mixed;
    }

    if contains_any(
        &normalized,
        &["unknown", "undisclosed", "no license", "not specified"],
    ) {
        return NormalizedLicense::Unknown;
    }

    if has_pd {
        return NormalizedLicense::PublicDomain;
    }
    if has_cc0 {
        return NormalizedLicense::Cc0;
    }
    if has_mit {
        return NormalizedLicense::Mit;
    }
    if has_apache {
        return NormalizedLicense::Apache20;
    }
    if has_ofl {
        return NormalizedLicense::Ofl;
    }
    if has_gpl {
        return NormalizedLicense::GplVariant;
    }

    NormalizedLicense::Unknown
}

pub fn classify_status(license: &NormalizedLicense) -> (AssetStatus, String) {
    match license {
        NormalizedLicense::PublicDomain
        | NormalizedLicense::Cc0
        | NormalizedLicense::Mit
        | NormalizedLicense::Apache20 => {
            (AssetStatus::Approved, "approved license class".to_string())
        }
        NormalizedLicense::Ofl => (
            AssetStatus::Rejected,
            "OFL is rejected in Phase 1".to_string(),
        ),
        NormalizedLicense::GplVariant => (
            AssetStatus::Rejected,
            "GPL variants are rejected in Phase 1".to_string(),
        ),
        NormalizedLicense::Mixed => (
            AssetStatus::Rejected,
            "mixed-license packs are rejected in Phase 1".to_string(),
        ),
        NormalizedLicense::Unknown => (
            AssetStatus::Quarantined,
            "unknown license: quarantine required".to_string(),
        ),
        NormalizedLicense::Ambiguous => (
            AssetStatus::Quarantined,
            "ambiguous provenance/license: quarantine required".to_string(),
        ),
    }
}

pub fn ingest_dir(dir: &Path) -> Result<Registry, RegistryError> {
    if !dir.exists() {
        return Err(RegistryError::Parse(format!(
            "ingest directory does not exist: {}",
            dir.display()
        )));
    }
    if !dir.is_dir() {
        return Err(RegistryError::Parse(format!(
            "ingest path is not a directory: {}",
            dir.display()
        )));
    }

    let mut assets = Vec::new();
    let mut candidate_paths = fs::read_dir(dir)?
        .map(|entry| entry.map(|v| v.path()))
        .collect::<Result<Vec<_>, io::Error>>()?;
    candidate_paths.retain(|path| path.is_file() && is_font_candidate(path));
    candidate_paths.sort();

    let mut seen_hashes = HashSet::new();
    for path in candidate_paths {
        let file_hash = hash_file_contents(&path)?;
        if !seen_hashes.insert(file_hash) {
            continue;
        }
        let metadata = fs::metadata(&path)?;
        let file_name = path
            .file_name()
            .map(|v| v.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string());
        let (family_name, style_name) = split_family_style_from_name(&file_name);
        let license_raw = read_license_sidecar(&path)?;
        let license_normalized = normalize_license(license_raw.as_deref());
        let (status, status_reason) = classify_status(&license_normalized);

        assets.push(FontAsset {
            id: deterministic_font_id(&path)?,
            path: path.to_string_lossy().to_string(),
            file_name,
            family_name,
            style_name,
            license_raw,
            license_normalized,
            status,
            status_reason,
            file_size_bytes: metadata.len(),
        });
    }

    assets.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(Registry { assets })
}

pub fn save_registry_at(root: &Path, registry: &Registry) -> Result<PathBuf, RegistryError> {
    fs::create_dir_all(root)?;
    let target = root.join(REGISTRY_FILE_NAME);
    fs::write(&target, registry_to_json(registry))?;
    Ok(target)
}

pub fn load_registry_at(root: &Path) -> Result<Registry, RegistryError> {
    let target = root.join(REGISTRY_FILE_NAME);
    let raw = fs::read_to_string(&target)?;
    parse_registry_json(&raw)
}

pub fn find_asset<'a>(
    registry: &'a Registry,
    font_id: &str,
) -> Result<&'a FontAsset, RegistryError> {
    registry
        .assets
        .iter()
        .find(|a| a.id == font_id)
        .ok_or_else(|| RegistryError::NotFound(font_id.to_string()))
}

pub fn registry_to_json(registry: &Registry) -> String {
    let mut assets = registry.assets.clone();
    assets.sort_by(|a, b| a.id.cmp(&b.id));

    let mut out = String::new();
    out.push_str("{\n  \"assets\": [\n");
    for (idx, asset) in assets.iter().enumerate() {
        out.push_str("    {\n");
        out.push_str(&format!("      \"id\": \"{}\",\n", escape_json(&asset.id)));
        out.push_str(&format!(
            "      \"path\": \"{}\",\n",
            escape_json(&asset.path)
        ));
        out.push_str(&format!(
            "      \"file_name\": \"{}\",\n",
            escape_json(&asset.file_name)
        ));

        match &asset.family_name {
            Some(v) => out.push_str(&format!("      \"family_name\": \"{}\",\n", escape_json(v))),
            None => out.push_str("      \"family_name\": null,\n"),
        }
        match &asset.style_name {
            Some(v) => out.push_str(&format!("      \"style_name\": \"{}\",\n", escape_json(v))),
            None => out.push_str("      \"style_name\": null,\n"),
        }
        match &asset.license_raw {
            Some(v) => out.push_str(&format!("      \"license_raw\": \"{}\",\n", escape_json(v))),
            None => out.push_str("      \"license_raw\": null,\n"),
        }

        out.push_str(&format!(
            "      \"license_normalized\": \"{}\",\n",
            asset.license_normalized.as_str()
        ));
        out.push_str(&format!(
            "      \"status\": \"{}\",\n",
            asset.status.as_str()
        ));
        out.push_str(&format!(
            "      \"status_reason\": \"{}\",\n",
            escape_json(&asset.status_reason)
        ));
        out.push_str(&format!(
            "      \"file_size_bytes\": {}\n",
            asset.file_size_bytes
        ));
        out.push_str("    }");
        if idx + 1 != assets.len() {
            out.push(',');
        }
        out.push('\n');
    }
    out.push_str("  ]\n}\n");
    out
}

pub fn parse_registry_json(raw: &str) -> Result<Registry, RegistryError> {
    let mut assets = Vec::new();
    for object in split_objects(raw) {
        if !object.contains("\"id\"") {
            continue;
        }

        let id = read_string_field(&object, "id")?.unwrap_or_default();
        let path = read_string_field(&object, "path")?.unwrap_or_default();
        let file_name = read_string_field(&object, "file_name")?.unwrap_or_default();
        let family_name = read_string_field(&object, "family_name")?;
        let style_name = read_string_field(&object, "style_name")?;
        let license_raw = read_string_field(&object, "license_raw")?;
        let license_normalized = parse_license(
            &read_string_field(&object, "license_normalized")?
                .unwrap_or_else(|| "unknown".to_string()),
        );
        let status = parse_status(
            &read_string_field(&object, "status")?.unwrap_or_else(|| "quarantined".to_string()),
        );
        let status_reason = read_string_field(&object, "status_reason")?.unwrap_or_default();
        let file_size_bytes = read_u64_field(&object, "file_size_bytes")?.unwrap_or(0);

        assets.push(FontAsset {
            id,
            path,
            file_name,
            family_name,
            style_name,
            license_raw,
            license_normalized,
            status,
            status_reason,
            file_size_bytes,
        });
    }
    assets.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(Registry { assets })
}

fn is_font_candidate(path: &Path) -> bool {
    let Some(ext) = path.extension() else {
        return false;
    };
    matches!(
        ext.to_string_lossy().to_ascii_lowercase().as_str(),
        "ttf" | "otf" | "woff" | "woff2"
    )
}

fn read_license_sidecar(font_path: &Path) -> Result<Option<String>, RegistryError> {
    let mut candidates = vec![
        font_path.with_extension("license"),
        font_path.with_extension("LICENSE"),
        font_path.with_extension("txt"),
    ];

    if let Some(stem) = font_path.file_stem() {
        let dir = font_path.parent().unwrap_or_else(|| Path::new("."));
        candidates.push(dir.join(format!("{}.license", stem.to_string_lossy())));
    }

    for candidate in candidates {
        if candidate.exists() && candidate.is_file() {
            let raw = fs::read_to_string(candidate)?;
            let cleaned = raw.trim().to_string();
            if cleaned.is_empty() {
                return Ok(None);
            }
            return Ok(Some(cleaned));
        }
    }

    Ok(None)
}

fn split_family_style_from_name(file_name: &str) -> (Option<String>, Option<String>) {
    let base = file_name
        .trim_end_matches(".ttf")
        .trim_end_matches(".otf")
        .trim_end_matches(".woff")
        .trim_end_matches(".woff2");

    let normalized = base.replace(['_', '-'], " ");
    let mut parts: Vec<String> = normalized
        .split_whitespace()
        .map(|s| s.to_string())
        .collect();

    if parts.is_empty() {
        return (None, None);
    }

    if parts.len() == 1 {
        return (Some(parts.remove(0)), None);
    }

    let style = parts.pop();
    (Some(parts.join(" ")), style)
}

fn deterministic_font_id(path: &Path) -> Result<String, RegistryError> {
    let seed = hash_file_contents(path)?;
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in seed.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    Ok(format!("font-{hash:016x}"))
}

fn hash_file_contents(path: &Path) -> Result<String, RegistryError> {
    let content = fs::read(path)?;
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in &content {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    Ok(format!("{hash:016x}"))
}

fn contains_any(input: &str, patterns: &[&str]) -> bool {
    patterns.iter().any(|pattern| input.contains(pattern))
}

fn contains_token(input: &str, token: &str) -> bool {
    input
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|part| !part.is_empty())
        .any(|part| part == token)
}

fn split_objects(raw: &str) -> Vec<String> {
    let mut objects = Vec::new();
    let mut depth = 0i32;
    let mut start = None;
    for (idx, ch) in raw.char_indices() {
        if ch == '{' {
            if depth == 1 {
                start = Some(idx);
            }
            depth += 1;
        } else if ch == '}' {
            depth -= 1;
            if depth == 1
                && let Some(s) = start.take()
            {
                objects.push(raw[s..=idx].to_string());
            }
        }
    }
    objects
}

fn read_string_field(object: &str, field: &str) -> Result<Option<String>, RegistryError> {
    let needle = format!("\"{field}\":");
    let Some(index) = object.find(&needle) else {
        return Ok(None);
    };
    let rest = object[index + needle.len()..].trim_start();
    if rest.starts_with("null") {
        return Ok(None);
    }
    if !rest.starts_with('"') {
        return Err(RegistryError::Parse(format!(
            "field '{field}' is not a string"
        )));
    }

    let mut escaped = false;
    let mut result = String::new();
    for c in rest[1..].chars() {
        if escaped {
            match c {
                'n' => result.push('\n'),
                'r' => result.push('\r'),
                't' => result.push('\t'),
                '"' => result.push('"'),
                '\\' => result.push('\\'),
                other => result.push(other),
            }
            escaped = false;
            continue;
        }
        if c == '\\' {
            escaped = true;
            continue;
        }
        if c == '"' {
            break;
        }
        result.push(c);
    }
    Ok(Some(result))
}

fn read_u64_field(object: &str, field: &str) -> Result<Option<u64>, RegistryError> {
    let needle = format!("\"{field}\":");
    let Some(index) = object.find(&needle) else {
        return Ok(None);
    };
    let rest = object[index + needle.len()..].trim_start();
    let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        return Err(RegistryError::Parse(format!(
            "field '{field}' is not numeric"
        )));
    }
    let value = digits
        .parse::<u64>()
        .map_err(|_| RegistryError::Parse(format!("field '{field}' parse error")))?;
    Ok(Some(value))
}

fn parse_license(raw: &str) -> NormalizedLicense {
    match raw {
        "public_domain" => NormalizedLicense::PublicDomain,
        "cc0" => NormalizedLicense::Cc0,
        "mit" => NormalizedLicense::Mit,
        "apache_2_0" => NormalizedLicense::Apache20,
        "ofl" => NormalizedLicense::Ofl,
        "gpl_variant" => NormalizedLicense::GplVariant,
        "ambiguous" => NormalizedLicense::Ambiguous,
        "mixed" => NormalizedLicense::Mixed,
        _ => NormalizedLicense::Unknown,
    }
}

fn parse_status(raw: &str) -> AssetStatus {
    match raw {
        "approved" => AssetStatus::Approved,
        "rejected" => AssetStatus::Rejected,
        _ => AssetStatus::Quarantined,
    }
}

fn contains_license_family_token(input: &str, family: &str) -> bool {
    input
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|part| !part.is_empty())
        .any(|part| part == family || part.starts_with(family))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn normalize_license_maps_policy_classes() {
        assert_eq!(
            normalize_license(Some("MIT License")),
            NormalizedLicense::Mit
        );
        assert_eq!(
            normalize_license(Some("Apache License Version 2.0")),
            NormalizedLicense::Apache20
        );
        assert_eq!(normalize_license(Some("CC0-1.0")), NormalizedLicense::Cc0);
        assert_eq!(
            normalize_license(Some("Open Font License")),
            NormalizedLicense::Ofl
        );
        assert_eq!(
            normalize_license(Some("GPLv3")),
            NormalizedLicense::GplVariant
        );
        assert_eq!(normalize_license(None), NormalizedLicense::Unknown);
    }

    #[test]
    fn normalize_license_avoids_partial_token_false_positives() {
        assert_eq!(
            normalize_license(Some("Permit required")),
            NormalizedLicense::Unknown
        );
        assert_eq!(
            normalize_license(Some("MIT OR Apache-2.0")),
            NormalizedLicense::Mixed
        );
        assert_eq!(
            normalize_license(Some("Dual-License Pack: MIT + OFL")),
            NormalizedLicense::Mixed
        );
        assert_eq!(
            normalize_license(Some("unknown provenance")),
            NormalizedLicense::Ambiguous
        );
    }

    #[test]
    fn classify_status_follows_phase1_policy() {
        assert_eq!(
            classify_status(&NormalizedLicense::Mit).0,
            AssetStatus::Approved
        );
        assert_eq!(
            classify_status(&NormalizedLicense::Ofl).0,
            AssetStatus::Rejected
        );
        assert_eq!(
            classify_status(&NormalizedLicense::Unknown).0,
            AssetStatus::Quarantined
        );
    }

    #[test]
    fn registry_roundtrip_save_load() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let tmp = std::env::temp_dir().join(format!("tw-registry-{nonce}"));
        fs::create_dir_all(&tmp).unwrap();

        let registry = Registry {
            assets: vec![FontAsset {
                id: "font-1".to_string(),
                path: "fixtures/approved.ttf".to_string(),
                file_name: "approved.ttf".to_string(),
                family_name: Some("approved".to_string()),
                style_name: Some("regular".to_string()),
                license_raw: Some("MIT".to_string()),
                license_normalized: NormalizedLicense::Mit,
                status: AssetStatus::Approved,
                status_reason: "approved".to_string(),
                file_size_bytes: 12,
            }],
        };

        save_registry_at(&tmp, &registry).unwrap();
        let loaded = load_registry_at(&tmp).unwrap();
        assert_eq!(loaded.assets.len(), 1);
        assert_eq!(loaded.assets[0].id, "font-1");

        let _ = fs::remove_file(tmp.join(REGISTRY_FILE_NAME));
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn registry_json_is_sorted_by_font_id() {
        let registry = Registry {
            assets: vec![
                FontAsset {
                    id: "font-b".to_string(),
                    path: "b.ttf".to_string(),
                    file_name: "b.ttf".to_string(),
                    family_name: None,
                    style_name: None,
                    license_raw: Some("MIT".to_string()),
                    license_normalized: NormalizedLicense::Mit,
                    status: AssetStatus::Approved,
                    status_reason: "approved".to_string(),
                    file_size_bytes: 1,
                },
                FontAsset {
                    id: "font-a".to_string(),
                    path: "a.ttf".to_string(),
                    file_name: "a.ttf".to_string(),
                    family_name: None,
                    style_name: None,
                    license_raw: Some("MIT".to_string()),
                    license_normalized: NormalizedLicense::Mit,
                    status: AssetStatus::Approved,
                    status_reason: "approved".to_string(),
                    file_size_bytes: 1,
                },
            ],
        };

        let json = registry_to_json(&registry);
        let pos_a = json.find("\"id\": \"font-a\"").unwrap();
        let pos_b = json.find("\"id\": \"font-b\"").unwrap();
        assert!(pos_a < pos_b);
    }

    #[test]
    fn ingest_skips_duplicate_font_payloads() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let tmp = std::env::temp_dir().join(format!("tw-dedupe-{nonce}"));
        fs::create_dir_all(&tmp).unwrap();

        let font_a = tmp.join("A.ttf");
        let font_b = tmp.join("B.ttf");
        fs::write(&font_a, b"same-bytes").unwrap();
        fs::write(&font_b, b"same-bytes").unwrap();
        fs::write(tmp.join("A.license"), "MIT").unwrap();
        fs::write(tmp.join("B.license"), "MIT").unwrap();

        let registry = ingest_dir(&tmp).unwrap();
        assert_eq!(registry.assets.len(), 1);
        assert_eq!(registry.assets[0].file_name, "A.ttf");

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn repeated_ingest_runs_are_idempotent() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let tmp = std::env::temp_dir().join(format!("tw-idempotent-{nonce}"));
        fs::create_dir_all(&tmp).unwrap();

        fs::write(tmp.join("FixtureA.ttf"), b"fixture-a").unwrap();
        fs::write(tmp.join("FixtureA.license"), "MIT").unwrap();
        fs::write(tmp.join("FixtureB.otf"), b"fixture-b").unwrap();
        fs::write(tmp.join("FixtureB.license"), "Unknown").unwrap();

        let first = ingest_dir(&tmp).unwrap();
        let second = ingest_dir(&tmp).unwrap();
        assert_eq!(first, second);
        assert_eq!(registry_to_json(&first), registry_to_json(&second));

        let _ = fs::remove_dir_all(&tmp);
    }
}
