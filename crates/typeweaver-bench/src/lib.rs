use std::path::{Path, PathBuf};

use typeweaver_core::{
    BenchmarkProfile, FontAsset, PREVIEW_FILE_NAME, ProfileMetrics, REPORT_FILE_NAME,
    REPORTS_DIR_NAME, ReportArtifacts, ReportBenchmark, ReportCard, ReportCorpusSummary,
    ReportFontIdentity, ReportMeasurements, ReportMetadata,
};
use typeweaver_render::{RenderedCorpus, render_fixed_latin_corpus};

fn run_profile_from_rendered(
    rendered: &RenderedCorpus,
    asset: &FontAsset,
    profile: BenchmarkProfile,
) -> ProfileMetrics {
    let line_density = rendered.corpus.char_count() as f32 / rendered.corpus.line_count() as f32;

    let profile_bias = match profile {
        BenchmarkProfile::WebLightDefault => 0.08,
        BenchmarkProfile::MobileDarkLowContrast => -0.04,
    };

    let confusion_penalty = match profile {
        BenchmarkProfile::WebLightDefault => 0.08,
        BenchmarkProfile::MobileDarkLowContrast => 0.15,
    };

    let mut score = rendered.estimated_coverage + profile_bias - confusion_penalty;
    score = score.clamp(0.0, 1.0);

    let notes = format!(
        "coverage={:.3}; profile={}; status={}",
        rendered.estimated_coverage,
        profile.as_str(),
        asset.status.as_str()
    );

    ProfileMetrics {
        profile,
        score,
        line_density,
        confusion_penalty,
        estimated_coverage: rendered.estimated_coverage,
        notes,
    }
}

pub fn run_profile(asset: &FontAsset, profile: BenchmarkProfile) -> ProfileMetrics {
    let rendered = render_fixed_latin_corpus(asset);
    run_profile_from_rendered(&rendered, asset, profile)
}

pub fn run_report(asset: &FontAsset, profile: BenchmarkProfile) -> ReportCard {
    let rendered = render_fixed_latin_corpus(asset);
    let metrics = run_profile_from_rendered(&rendered, asset, profile);

    ReportCard {
        schema_id: "typeweaver.report_card.v1".to_string(),
        report_version: "phase1-v1".to_string(),
        metadata: ReportMetadata {
            generated_at_utc: "unix:0".to_string(),
        },
        font: ReportFontIdentity {
            font_id: asset.id.clone(),
            family_name: asset.family_name.clone(),
            style_name: asset.style_name.clone(),
            normalized_license: asset.license_normalized.as_str().to_string(),
            status: asset.status.as_str().to_string(),
        },
        benchmark: ReportBenchmark {
            profile: profile.as_str().to_string(),
        },
        corpus: ReportCorpusSummary {
            line_count: rendered.corpus.line_count(),
            char_count: rendered.corpus.char_count(),
        },
        artifacts: ReportArtifacts {
            report_path: None,
            preview_files: Vec::new(),
        },
        measurements: ReportMeasurements {
            score: metrics.score,
            line_density: metrics.line_density,
            confusion_penalty: metrics.confusion_penalty,
            estimated_coverage: metrics.estimated_coverage,
            notes: metrics.notes,
        },
        ocr_score: None,
    }
}

pub fn report_run_dir(registry_root: &Path, font_id: &str, profile: BenchmarkProfile) -> PathBuf {
    registry_root
        .join(REPORTS_DIR_NAME)
        .join(font_id)
        .join(profile.as_str())
}

pub fn default_report_path(
    registry_root: &Path,
    font_id: &str,
    profile: BenchmarkProfile,
) -> PathBuf {
    report_run_dir(registry_root, font_id, profile).join(REPORT_FILE_NAME)
}

pub fn default_preview_path(
    registry_root: &Path,
    font_id: &str,
    profile: BenchmarkProfile,
) -> PathBuf {
    report_run_dir(registry_root, font_id, profile).join(PREVIEW_FILE_NAME)
}

pub fn render_preview_text(asset: &FontAsset) -> String {
    let rendered = render_fixed_latin_corpus(asset);
    let mut out = String::new();
    out.push_str("# TypeWeaver Phase 1 Preview\n");
    out.push_str(&format!("font_id: {}\n", asset.id));
    out.push_str(&format!("font_file: {}\n", asset.file_name));
    out.push('\n');
    out.push_str(&rendered.rendered_lines.join("\n"));
    out.push('\n');
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use typeweaver_core::{AssetStatus, NormalizedLicense};

    fn fixture_asset() -> FontAsset {
        FontAsset {
            id: "font-fixture".to_string(),
            path: "fixtures/font.ttf".to_string(),
            file_name: "font.ttf".to_string(),
            family_name: Some("Fixture".to_string()),
            style_name: Some("Regular".to_string()),
            license_raw: Some("MIT".to_string()),
            license_normalized: NormalizedLicense::Mit,
            status: AssetStatus::Approved,
            status_reason: "approved".to_string(),
            file_size_bytes: 2048,
        }
    }

    #[test]
    fn supports_exact_phase1_profiles() {
        assert!(BenchmarkProfile::from_slug("web_light_default").is_ok());
        assert!(BenchmarkProfile::from_slug("mobile_dark_low_contrast").is_ok());
        assert!(BenchmarkProfile::from_slug("other").is_err());
    }

    #[test]
    fn report_contains_stable_shape() {
        let report = run_report(&fixture_asset(), BenchmarkProfile::WebLightDefault);
        let json = report.to_json_pretty();
        assert!(json.contains("\"schema_id\": \"typeweaver.report_card.v1\""));
        assert!(json.contains("\"font\": {"));
        assert!(json.contains("\"benchmark\": {"));
        assert!(json.contains("\"measurements\": {"));
        assert!(json.contains("\"profile\": \"web_light_default\""));
    }

    #[test]
    fn report_generation_stamp_is_deterministic() {
        let report = run_report(&fixture_asset(), BenchmarkProfile::WebLightDefault);
        assert_eq!(report.metadata.generated_at_utc, "unix:0");
    }

    #[test]
    fn default_report_paths_are_predictable() {
        let root = Path::new(".typeweaver");
        let path = default_report_path(root, "font-fixture", BenchmarkProfile::WebLightDefault);
        assert_eq!(
            path,
            PathBuf::from(".typeweaver/reports/font-fixture/web_light_default/report.json")
        );
        let preview = default_preview_path(root, "font-fixture", BenchmarkProfile::WebLightDefault);
        assert_eq!(
            preview,
            PathBuf::from(".typeweaver/reports/font-fixture/web_light_default/preview.txt")
        );
    }
}
