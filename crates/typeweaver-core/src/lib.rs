use std::fmt;

pub const REGISTRY_DIR_NAME: &str = "registry";
pub const REGISTRY_FILE_NAME: &str = "registry.json";
pub const REPORTS_DIR_NAME: &str = "reports";
pub const REPORT_FILE_NAME: &str = "report.json";
pub const PREVIEW_FILE_NAME: &str = "preview.txt";

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

    pub fn description(&self) -> &'static str {
        match self {
            Self::WebLightDefault => {
                "Balanced desktop/web readability profile with default contrast assumptions"
            }
            Self::MobileDarkLowContrast => {
                "Mobile-oriented dark background profile with tighter low-contrast tolerance"
            }
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
    pub estimated_coverage: f32,
    pub notes: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReportMetadata {
    pub generated_at_utc: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReportFontIdentity {
    pub font_id: String,
    pub family_name: Option<String>,
    pub style_name: Option<String>,
    pub normalized_license: String,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReportBenchmark {
    pub profile: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReportCorpusSummary {
    pub line_count: usize,
    pub char_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReportArtifacts {
    pub report_path: Option<String>,
    pub preview_files: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OcrScore {
    pub expected: String,
    pub recognized: String,
    pub char_accuracy: f32,
    pub word_accuracy: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReportMeasurements {
    pub score: f32,
    pub line_density: f32,
    pub confusion_penalty: f32,
    pub estimated_coverage: f32,
    pub notes: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReportCard {
    pub schema_id: String,
    pub report_version: String,
    pub metadata: ReportMetadata,
    pub font: ReportFontIdentity,
    pub benchmark: ReportBenchmark,
    pub corpus: ReportCorpusSummary,
    pub artifacts: ReportArtifacts,
    pub measurements: ReportMeasurements,
    pub ocr_score: Option<OcrScore>,
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

fn push_json_string_field(target: &mut String, key: &str, value: &str, trailing_comma: bool) {
    target.push_str(&format!("    \"{key}\": \"{}\"", escape_json(value)));
    if trailing_comma {
        target.push(',');
    }
    target.push('\n');
}

fn push_json_optional_string_field(
    target: &mut String,
    key: &str,
    value: &Option<String>,
    trailing_comma: bool,
) {
    target.push_str(&format!("    \"{key}\": "));
    match value {
        Some(v) => target.push_str(&format!("\"{}\"", escape_json(v))),
        None => target.push_str("null"),
    }
    if trailing_comma {
        target.push(',');
    }
    target.push('\n');
}

impl ReportCard {
    pub fn to_json_pretty(&self) -> String {
        let mut json = String::new();
        json.push_str("{\n");
        json.push_str(&format!(
            "  \"schema_id\": \"{}\",\n",
            escape_json(&self.schema_id)
        ));
        json.push_str(&format!(
            "  \"report_version\": \"{}\",\n",
            escape_json(&self.report_version)
        ));

        json.push_str("  \"metadata\": {\n");
        push_json_string_field(
            &mut json,
            "generated_at_utc",
            &self.metadata.generated_at_utc,
            false,
        );
        json.push_str("  },\n");

        json.push_str("  \"font\": {\n");
        push_json_string_field(&mut json, "font_id", &self.font.font_id, true);
        push_json_optional_string_field(&mut json, "family_name", &self.font.family_name, true);
        push_json_optional_string_field(&mut json, "style_name", &self.font.style_name, true);
        push_json_string_field(
            &mut json,
            "normalized_license",
            &self.font.normalized_license,
            true,
        );
        push_json_string_field(&mut json, "status", &self.font.status, false);
        json.push_str("  },\n");

        json.push_str("  \"benchmark\": {\n");
        push_json_string_field(&mut json, "profile", &self.benchmark.profile, false);
        json.push_str("  },\n");

        json.push_str("  \"corpus\": {\n");
        json.push_str(&format!(
            "    \"line_count\": {},\n",
            self.corpus.line_count
        ));
        json.push_str(&format!("    \"char_count\": {}\n", self.corpus.char_count));
        json.push_str("  },\n");

        json.push_str("  \"artifacts\": {\n");
        push_json_optional_string_field(
            &mut json,
            "report_path",
            &self.artifacts.report_path,
            true,
        );
        json.push_str("    \"preview_files\": [");
        for (idx, file) in self.artifacts.preview_files.iter().enumerate() {
            if idx > 0 {
                json.push_str(", ");
            }
            json.push_str(&format!("\"{}\"", escape_json(file)));
        }
        json.push_str("]\n");
        json.push_str("  },\n");

        json.push_str("  \"measurements\": {\n");
        json.push_str(&format!("    \"score\": {:.4},\n", self.measurements.score));
        json.push_str(&format!(
            "    \"line_density\": {:.4},\n",
            self.measurements.line_density
        ));
        json.push_str(&format!(
            "    \"confusion_penalty\": {:.4},\n",
            self.measurements.confusion_penalty
        ));
        json.push_str(&format!(
            "    \"estimated_coverage\": {:.4},\n",
            self.measurements.estimated_coverage
        ));
        push_json_string_field(&mut json, "notes", &self.measurements.notes, false);
        json.push_str("  },\n");

        match &self.ocr_score {
            Some(ocr) => {
                json.push_str("  \"ocr_score\": {\n");
                push_json_string_field(&mut json, "expected", &ocr.expected, true);
                push_json_string_field(&mut json, "recognized", &ocr.recognized, true);
                json.push_str(&format!("    \"char_accuracy\": {:.4},\n", ocr.char_accuracy));
                json.push_str(&format!("    \"word_accuracy\": {:.4}\n", ocr.word_accuracy));
                json.push_str("  }\n");
            }
            None => {
                json.push_str("  \"ocr_score\": null\n");
            }
        }

        json.push('}');
        json
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn benchmark_profile_parser_is_strict() {
        assert!(BenchmarkProfile::from_slug("web_light_default").is_ok());
        assert!(BenchmarkProfile::from_slug("mobile_dark_low_contrast").is_ok());
        assert!(BenchmarkProfile::from_slug("Web_Light_Default").is_err());
    }

    #[test]
    fn report_card_json_is_deterministic() {
        let report = ReportCard {
            schema_id: "typeweaver.report_card.v1".to_string(),
            report_version: "phase1-v1".to_string(),
            metadata: ReportMetadata {
                generated_at_utc: "unix:0".to_string(),
            },
            font: ReportFontIdentity {
                font_id: "font-123".to_string(),
                family_name: Some("Fixture".to_string()),
                style_name: Some("Regular".to_string()),
                normalized_license: "mit".to_string(),
                status: "approved".to_string(),
            },
            benchmark: ReportBenchmark {
                profile: "web_light_default".to_string(),
            },
            corpus: ReportCorpusSummary {
                line_count: 5,
                char_count: 100,
            },
            artifacts: ReportArtifacts {
                report_path: Some("reports/font-123/web_light_default/report.json".to_string()),
                preview_files: vec!["preview.txt".to_string()],
            },
            measurements: ReportMeasurements {
                score: 0.8,
                line_density: 20.0,
                confusion_penalty: 0.08,
                estimated_coverage: 0.88,
                notes: "phase1".to_string(),
            },
            ocr_score: None,
        };

        let first = report.to_json_pretty();
        let second = report.to_json_pretty();
        assert_eq!(first, second);
        assert!(first.contains("\"benchmark\": {"));
        assert!(first.contains("\"artifacts\": {"));
        assert!(first.contains("\"measurements\": {"));
    }
}
