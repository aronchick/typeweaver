use typeweaver_core::{Corpus, FontAsset};

#[derive(Debug, Clone, PartialEq)]
pub struct RenderedCorpus {
    pub corpus: Corpus,
    pub rendered_lines: Vec<String>,
    pub estimated_coverage: f32,
}

pub fn render_fixed_latin_corpus(asset: &FontAsset) -> RenderedCorpus {
    let corpus = Corpus::latin_phase1();
    let rendered_lines = corpus
        .lines
        .iter()
        .enumerate()
        .map(|(idx, line)| {
            format!(
                "[{}:{}] {}",
                asset.family_name
                    .as_deref()
                    .unwrap_or(asset.file_name.as_str()),
                idx + 1,
                line
            )
        })
        .collect::<Vec<_>>();

    // Deterministic, file-size based heuristic to approximate corpus support.
    let size_signal = (asset.file_size_bytes % 4096) as f32 / 4096.0;
    let estimated_coverage = (0.70 + size_signal * 0.30).min(1.0);

    RenderedCorpus {
        corpus,
        rendered_lines,
        estimated_coverage,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use typeweaver_core::{AssetStatus, FontAsset, NormalizedLicense};

    #[test]
    fn renders_all_required_corpus_lines() {
        let asset = FontAsset {
            id: "font-1".to_string(),
            path: "fixtures/font.ttf".to_string(),
            file_name: "font.ttf".to_string(),
            family_name: Some("FixtureSans".to_string()),
            style_name: Some("Regular".to_string()),
            license_raw: Some("MIT".to_string()),
            license_normalized: NormalizedLicense::Mit,
            status: AssetStatus::Approved,
            status_reason: "approved".to_string(),
            file_size_bytes: 1024,
        };

        let rendered = render_fixed_latin_corpus(&asset);
        assert_eq!(rendered.corpus.line_count(), 5);
        assert_eq!(rendered.rendered_lines.len(), 5);
        assert!(rendered.rendered_lines[0].contains("ABCDEFGHIJKLMNOPQRSTUVWXYZ"));
    }
}
