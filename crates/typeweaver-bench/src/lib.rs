use std::time::{SystemTime, UNIX_EPOCH};

use typeweaver_core::{BenchmarkProfile, FontAsset, ProfileMetrics, REPORTS_DIR_NAME, ReportCard};
use typeweaver_render::render_fixed_latin_corpus;

pub fn run_profile(asset: &FontAsset, profile: BenchmarkProfile) -> ProfileMetrics {
    let rendered = render_fixed_latin_corpus(asset);
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
    if score < 0.0 {
        score = 0.0;
    }
    if score > 1.0 {
        score = 1.0;
    }

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
        notes,
    }
}

pub fn run_report(asset: &FontAsset, profile: Option<BenchmarkProfile>) -> ReportCard {
    let rendered = render_fixed_latin_corpus(asset);
    let profile_metrics = match profile {
        Some(p) => vec![run_profile(asset, p)],
        None => BenchmarkProfile::all()
            .into_iter()
            .map(|p| run_profile(asset, p))
            .collect(),
    };

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    ReportCard {
        report_version: "phase1-v1".to_string(),
        generated_at_utc: format!("unix:{now}"),
        font_id: asset.id.clone(),
        font_family: asset.family_name.clone(),
        profile_metrics,
        corpus_line_count: rendered.corpus.line_count(),
        corpus_char_count: rendered.corpus.char_count(),
    }
}

pub fn default_report_path(font_id: &str) -> String {
    format!("{REPORTS_DIR_NAME}/{font_id}.json")
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
    fn report_contains_json_output() {
        let report = run_report(&fixture_asset(), Some(BenchmarkProfile::WebLightDefault));
        let json = report.to_json_pretty();
        assert!(json.contains("\"font_id\": \"font-fixture\""));
        assert!(json.contains("\"profile\": \"web_light_default\""));
    }
}
