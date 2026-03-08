use std::fmt;

pub const REGISTRY_FILE_NAME: &str = "registry.json";
pub const REPORTS_DIR_NAME: &str = "reports";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NormalizedLicense {
    PublicDomain,
    Cc0,
    Mit,
    Apache20,
    Ofl,
    GplVariant,
    Unknown,
    Ambiguous,
    Mixed,
}

impl NormalizedLicense {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::PublicDomain => "public_domain",
            Self::Cc0 => "cc0",
            Self::Mit => "mit",
            Self::Apache20 => "apache_2_0",
            Self::Ofl => "ofl",
            Self::GplVariant => "gpl_variant",
            Self::Unknown => "unknown",
            Self::Ambiguous => "ambiguous",
            Self::Mixed => "mixed",
        }
    }

    pub fn is_approved(&self) -> bool {
        matches!(
            self,
            Self::PublicDomain | Self::Cc0 | Self::Mit | Self::Apache20
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssetStatus {
    Approved,
    Rejected,
    Quarantined,
}

impl AssetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Quarantined => "quarantined",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FontAsset {
    pub id: String,
    pub path: String,
    pub file_name: String,
    pub family_name: Option<String>,
    pub style_name: Option<String>,
    pub license_raw: Option<String>,
    pub license_normalized: NormalizedLicense,
    pub status: AssetStatus,
    pub status_reason: String,
    pub file_size_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Registry {
    pub assets: Vec<FontAsset>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Corpus {
    pub lines: Vec<String>,
}

impl Corpus {
    pub fn latin_phase1() -> Self {
        let lines = vec![
            "ABCDEFGHIJKLMNOPQRSTUVWXYZ".to_string(),
            "abcdefghijklmnopqrstuvwxyz".to_string(),
            "0123456789".to_string(),
            "!@#$%^&*()-_=+[]{};:'\",.<>/?\\|`~".to_string(),
            "O/0 I/l/1 S/5 B/8 rn/m cl/d".to_string(),
        ];
        Self { lines }
    }

    pub fn as_text(&self) -> String {
        self.lines.join("\n")
    }

    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    pub fn char_count(&self) -> usize {
        self.lines.iter().map(|line| line.chars().count()).sum()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BenchmarkProfile {
    WebLightDefault,
    MobileDarkLowContrast,
}

impl BenchmarkProfile {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::WebLightDefault => "web_light_default",
            Self::MobileDarkLowContrast => "mobile_dark_low_contrast",
        }
    }

    pub fn all() -> [Self; 2] {
        [Self::WebLightDefault, Self::MobileDarkLowContrast]
    }

    pub fn from_slug(input: &str) -> Result<Self, ProfileParseError> {
        match input {
            "web_light_default" => Ok(Self::WebLightDefault),
            "mobile_dark_low_contrast" => Ok(Self::MobileDarkLowContrast),
            other => Err(ProfileParseError {
                input: other.to_string(),
            }),
        }
    }
}

impl fmt::Display for BenchmarkProfile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileParseError {
    pub input: String,
}

impl fmt::Display for ProfileParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "unsupported profile '{}'. expected one of: web_light_default, mobile_dark_low_contrast",
            self.input
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProfileMetrics {
    pub profile: BenchmarkProfile,
    pub score: f32,
    pub line_density: f32,
    pub confusion_penalty: f32,
    pub notes: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReportCard {
    pub report_version: String,
    pub generated_at_utc: String,
    pub font_id: String,
    pub font_family: Option<String>,
    pub profile_metrics: Vec<ProfileMetrics>,
    pub corpus_line_count: usize,
    pub corpus_char_count: usize,
}

pub fn escape_json(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for c in input.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

impl ReportCard {
    pub fn to_json_pretty(&self) -> String {
        let mut json = String::new();
        json.push_str("{\n");
        json.push_str(&format!(
            "  \"report_version\": \"{}\",\n",
            escape_json(&self.report_version)
        ));
        json.push_str(&format!(
            "  \"generated_at_utc\": \"{}\",\n",
            escape_json(&self.generated_at_utc)
        ));
        json.push_str(&format!(
            "  \"font_id\": \"{}\",\n",
            escape_json(&self.font_id)
        ));
        match &self.font_family {
            Some(family) => json.push_str(&format!(
                "  \"font_family\": \"{}\",\n",
                escape_json(family)
            )),
            None => json.push_str("  \"font_family\": null,\n"),
        }

        json.push_str("  \"profile_metrics\": [\n");
        for (idx, metric) in self.profile_metrics.iter().enumerate() {
            json.push_str("    {\n");
            json.push_str(&format!(
                "      \"profile\": \"{}\",\n",
                metric.profile.as_str()
            ));
            json.push_str(&format!("      \"score\": {:.4},\n", metric.score));
            json.push_str(&format!(
                "      \"line_density\": {:.4},\n",
                metric.line_density
            ));
            json.push_str(&format!(
                "      \"confusion_penalty\": {:.4},\n",
                metric.confusion_penalty
            ));
            json.push_str(&format!(
                "      \"notes\": \"{}\"\n",
                escape_json(&metric.notes)
            ));
            json.push_str("    }");
            if idx + 1 != self.profile_metrics.len() {
                json.push(',');
            }
            json.push('\n');
        }
        json.push_str("  ],\n");
        json.push_str(&format!(
            "  \"corpus_line_count\": {},\n",
            self.corpus_line_count
        ));
        json.push_str(&format!(
            "  \"corpus_char_count\": {}\n",
            self.corpus_char_count
        ));
        json.push('}');
        json
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_parse_accepts_known_slugs() {
        assert_eq!(
            BenchmarkProfile::from_slug("web_light_default").unwrap(),
            BenchmarkProfile::WebLightDefault
        );
        assert_eq!(
            BenchmarkProfile::from_slug("mobile_dark_low_contrast").unwrap(),
            BenchmarkProfile::MobileDarkLowContrast
        );
    }

    #[test]
    fn profile_parse_rejects_unknown_slug() {
        let err = BenchmarkProfile::from_slug("unknown").unwrap_err();
        assert_eq!(err.input, "unknown");
    }

    #[test]
    fn corpus_contains_required_lines() {
        let corpus = Corpus::latin_phase1();
        assert_eq!(corpus.line_count(), 5);
        assert!(corpus.as_text().contains("ABCDEFGHIJKLMNOPQRSTUVWXYZ"));
        assert!(corpus.as_text().contains("abcdefghijklmnopqrstuvwxyz"));
        assert!(corpus.as_text().contains("0123456789"));
        assert!(corpus.as_text().contains("O/0 I/l/1 S/5 B/8 rn/m cl/d"));
    }

    #[test]
    fn report_serialization_contains_expected_keys() {
        let report = ReportCard {
            report_version: "phase1-v1".to_string(),
            generated_at_utc: "2026-03-08T00:00:00Z".to_string(),
            font_id: "font-123".to_string(),
            font_family: Some("Example".to_string()),
            profile_metrics: vec![ProfileMetrics {
                profile: BenchmarkProfile::WebLightDefault,
                score: 0.9,
                line_density: 0.7,
                confusion_penalty: 0.1,
                notes: "ok".to_string(),
            }],
            corpus_line_count: 5,
            corpus_char_count: 90,
        };

        let json = report.to_json_pretty();
        assert!(json.contains("\"font_id\": \"font-123\""));
        assert!(json.contains("\"profile\": \"web_light_default\""));
        assert!(json.contains("\"corpus_line_count\": 5"));
    }

    #[test]
    fn approved_licenses_are_marked() {
        assert!(NormalizedLicense::PublicDomain.is_approved());
        assert!(NormalizedLicense::Cc0.is_approved());
        assert!(NormalizedLicense::Mit.is_approved());
        assert!(NormalizedLicense::Apache20.is_approved());
        assert!(!NormalizedLicense::Unknown.is_approved());
    }
}
