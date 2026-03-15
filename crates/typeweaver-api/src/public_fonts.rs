use std::collections::{HashMap, HashSet};

use reqwest::Client;
use serde::Serialize;
use serde_json::Value;

const GOOGLE_FONTS_METADATA_URL: &str = "https://fonts.google.com/metadata/fonts";
const GOOGLE_FONTS_CSS_URL: &str = "https://fonts.googleapis.com/css2";
const FALLBACK_QUERY_LIMIT: usize = 12;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublicFontRecord {
    pub family: String,
    pub category: String,
    pub source: String,
    pub declared_license: Option<String>,
    pub spotlight_rank: Option<usize>,
}

#[derive(Clone, Debug)]
pub struct PublicFontCatalog {
    pub fonts: Vec<PublicFontRecord>,
    pub degraded: bool,
    pub source: String,
    pub note: Option<String>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct PublicFontSearchEntry {
    pub family: String,
    pub category: String,
    pub source: String,
    pub license_hint: String,
    pub declared_license: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct PublicFontSearchResponse {
    pub query: String,
    pub source: String,
    pub degraded: bool,
    pub note: Option<String>,
    pub fonts: Vec<PublicFontSearchEntry>,
}

#[derive(Clone, Debug)]
pub struct ResolvedPublicFont {
    pub family: String,
    pub file_name: String,
    pub bytes: Vec<u8>,
    pub declared_license: Option<String>,
    pub source: String,
}

#[derive(Clone, Copy)]
struct FallbackPublicFont {
    family: &'static str,
    category: &'static str,
    declared_license: Option<&'static str>,
    spotlight_rank: Option<usize>,
}

const FALLBACK_PUBLIC_FONTS: &[FallbackPublicFont] = &[
    FallbackPublicFont {
        family: "Inter",
        category: "Sans Serif",
        declared_license: Some("SIL Open Font License, Version 1.1"),
        spotlight_rank: Some(0),
    },
    FallbackPublicFont {
        family: "IBM Plex Sans",
        category: "Sans Serif",
        declared_license: Some("SIL Open Font License, Version 1.1"),
        spotlight_rank: Some(1),
    },
    FallbackPublicFont {
        family: "Source Serif 4",
        category: "Serif",
        declared_license: Some("SIL Open Font License, Version 1.1"),
        spotlight_rank: Some(2),
    },
    FallbackPublicFont {
        family: "Roboto",
        category: "Sans Serif",
        declared_license: Some("SIL Open Font License, Version 1.1"),
        spotlight_rank: Some(3),
    },
    FallbackPublicFont {
        family: "Roboto Mono",
        category: "Monospace",
        declared_license: Some("SIL Open Font License, Version 1.1"),
        spotlight_rank: Some(4),
    },
    FallbackPublicFont {
        family: "Roboto Slab",
        category: "Serif",
        declared_license: Some("Apache License Version 2.0"),
        spotlight_rank: Some(5),
    },
    FallbackPublicFont {
        family: "Roboto Condensed",
        category: "Sans Serif",
        declared_license: Some("SIL Open Font License, Version 1.1"),
        spotlight_rank: Some(6),
    },
    FallbackPublicFont {
        family: "Archivo",
        category: "Sans Serif",
        declared_license: Some("SIL Open Font License, Version 1.1"),
        spotlight_rank: Some(7),
    },
    FallbackPublicFont {
        family: "Arimo",
        category: "Sans Serif",
        declared_license: Some("Apache License Version 2.0"),
        spotlight_rank: Some(8),
    },
    FallbackPublicFont {
        family: "Tinos",
        category: "Serif",
        declared_license: Some("Apache License Version 2.0"),
        spotlight_rank: Some(9),
    },
    FallbackPublicFont {
        family: "Cousine",
        category: "Monospace",
        declared_license: Some("Apache License Version 2.0"),
        spotlight_rank: Some(10),
    },
    FallbackPublicFont {
        family: "Space Grotesk",
        category: "Sans Serif",
        declared_license: Some("SIL Open Font License, Version 1.1"),
        spotlight_rank: Some(11),
    },
    FallbackPublicFont {
        family: "Work Sans",
        category: "Sans Serif",
        declared_license: Some("SIL Open Font License, Version 1.1"),
        spotlight_rank: None,
    },
    FallbackPublicFont {
        family: "Merriweather",
        category: "Serif",
        declared_license: Some("SIL Open Font License, Version 1.1"),
        spotlight_rank: None,
    },
    FallbackPublicFont {
        family: "Lora",
        category: "Serif",
        declared_license: Some("SIL Open Font License, Version 1.1"),
        spotlight_rank: None,
    },
    FallbackPublicFont {
        family: "Noto Sans",
        category: "Sans Serif",
        declared_license: Some("SIL Open Font License, Version 1.1"),
        spotlight_rank: None,
    },
    FallbackPublicFont {
        family: "Noto Serif",
        category: "Serif",
        declared_license: Some("SIL Open Font License, Version 1.1"),
        spotlight_rank: None,
    },
    FallbackPublicFont {
        family: "Fira Sans",
        category: "Sans Serif",
        declared_license: Some("SIL Open Font License, Version 1.1"),
        spotlight_rank: None,
    },
    FallbackPublicFont {
        family: "Manrope",
        category: "Sans Serif",
        declared_license: Some("SIL Open Font License, Version 1.1"),
        spotlight_rank: None,
    },
    FallbackPublicFont {
        family: "PT Serif",
        category: "Serif",
        declared_license: Some("SIL Open Font License, Version 1.1"),
        spotlight_rank: None,
    },
];

pub async fn load_public_font_catalog(client: &Client) -> PublicFontCatalog {
    match fetch_google_fonts_catalog(client).await {
        Ok(fonts) => PublicFontCatalog {
            fonts,
            degraded: false,
            source: "google_fonts_metadata".to_string(),
            note: None,
        },
        Err(error) => PublicFontCatalog {
            fonts: fallback_public_fonts(),
            degraded: true,
            source: "fallback_catalog".to_string(),
            note: Some(format!(
                "Live Google Fonts metadata was unavailable, so TypeWeaver is showing the built-in public catalog instead. ({error})"
            )),
        },
    }
}

pub fn search_public_font_catalog(
    catalog: &PublicFontCatalog,
    query: &str,
    limit: Option<usize>,
) -> PublicFontSearchResponse {
    let requested_limit = limit.unwrap_or(FALLBACK_QUERY_LIMIT).clamp(1, 24);
    let normalized_query = normalize_query(query);
    let fonts = if normalized_query.is_empty() {
        catalog
            .fonts
            .iter()
            .filter(|font| font.spotlight_rank.is_some())
            .take(requested_limit)
            .cloned()
            .collect::<Vec<_>>()
    } else {
        let tokens = normalized_query
            .split_whitespace()
            .filter(|token| !token.is_empty())
            .map(str::to_string)
            .collect::<Vec<_>>();

        let mut scored = catalog
            .fonts
            .iter()
            .filter_map(|font| {
                score_font_match(font, &normalized_query, &tokens).map(|score| (score, font))
            })
            .collect::<Vec<_>>();
        scored.sort_by(|left, right| {
            left.0
                .cmp(&right.0)
                .then_with(|| left.1.family.cmp(&right.1.family))
        });
        scored
            .into_iter()
            .take(requested_limit)
            .map(|(_, font)| font.clone())
            .collect::<Vec<_>>()
    };

    PublicFontSearchResponse {
        query: query.trim().to_string(),
        source: catalog.source.clone(),
        degraded: catalog.degraded,
        note: catalog.note.clone(),
        fonts: fonts
            .into_iter()
            .map(|font| PublicFontSearchEntry {
                family: font.family,
                category: font.category,
                source: font.source,
                license_hint: font
                    .declared_license
                    .clone()
                    .unwrap_or_else(|| "License review after ingest".to_string()),
                declared_license: font.declared_license,
            })
            .collect(),
    }
}

pub async fn resolve_public_font(
    client: &Client,
    family: &str,
    declared_license: Option<&str>,
) -> Result<ResolvedPublicFont, String> {
    let trimmed_family = family.trim();
    if trimmed_family.is_empty() {
        return Err("font family is required".to_string());
    }

    let css_url = google_fonts_css_url(trimmed_family)?;
    let css_response = client
        .get(css_url)
        .send()
        .await
        .map_err(|error| format!("could not fetch font stylesheet: {error}"))?;
    if !css_response.status().is_success() {
        return Err(format!(
            "font stylesheet returned {}",
            css_response.status()
        ));
    }

    let css = css_response
        .text()
        .await
        .map_err(|error| format!("could not read font stylesheet: {error}"))?;
    let font_url = extract_stylesheet_font_url(&css)
        .ok_or_else(|| "could not find a downloadable font file in the stylesheet".to_string())?;
    let font_response = client
        .get(font_url.clone())
        .send()
        .await
        .map_err(|error| format!("could not download the font file: {error}"))?;
    if !font_response.status().is_success() {
        return Err(format!("font file returned {}", font_response.status()));
    }

    let content_type = font_response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(|value| value.to_string());
    let bytes = font_response
        .bytes()
        .await
        .map_err(|error| format!("could not read the font file: {error}"))?
        .to_vec();
    let file_name = file_name_for_family(trimmed_family, &font_url, content_type.as_deref())?;

    Ok(ResolvedPublicFont {
        family: trimmed_family.to_string(),
        file_name,
        bytes,
        declared_license: declared_license
            .map(str::trim)
            .filter(|license| !license.is_empty())
            .map(str::to_string),
        source: "google_fonts".to_string(),
    })
}

async fn fetch_google_fonts_catalog(client: &Client) -> Result<Vec<PublicFontRecord>, String> {
    let response = client
        .get(GOOGLE_FONTS_METADATA_URL)
        .send()
        .await
        .map_err(|error| format!("request failed: {error}"))?;
    if !response.status().is_success() {
        return Err(format!("metadata returned {}", response.status()));
    }
    let body = response
        .text()
        .await
        .map_err(|error| format!("metadata read failed: {error}"))?;
    parse_google_fonts_metadata(&body)
}

fn parse_google_fonts_metadata(raw: &str) -> Result<Vec<PublicFontRecord>, String> {
    let cleaned = raw
        .trim_start()
        .strip_prefix(")]}'")
        .map(str::trim_start)
        .unwrap_or(raw)
        .trim_start();
    let value: Value =
        serde_json::from_str(cleaned).map_err(|error| format!("metadata parse failed: {error}"))?;
    let families = value
        .get("familyMetadataList")
        .and_then(Value::as_array)
        .or_else(|| value.get("familyList").and_then(Value::as_array))
        .or_else(|| value.as_array())
        .ok_or_else(|| "metadata did not contain a font list".to_string())?;

    let overrides = fallback_overrides();
    let mut seen = HashSet::new();
    let mut fonts = Vec::new();
    for item in families {
        let Some(family) = item.get("family").and_then(Value::as_str) else {
            continue;
        };
        if !seen.insert(family.to_ascii_lowercase()) {
            continue;
        }

        let override_entry = overrides.get(&family.to_ascii_lowercase());
        let category = override_entry
            .map(|entry| entry.category.to_string())
            .or_else(|| {
                item.get("category")
                    .and_then(Value::as_str)
                    .map(normalize_category)
            })
            .unwrap_or_else(|| "Unknown".to_string());
        let declared_license = override_entry
            .and_then(|entry| entry.declared_license.map(|license| license.to_string()));
        let spotlight_rank = override_entry.and_then(|entry| entry.spotlight_rank);

        fonts.push(PublicFontRecord {
            family: family.to_string(),
            category,
            source: "google_fonts_metadata".to_string(),
            declared_license,
            spotlight_rank,
        });
    }

    if fonts.is_empty() {
        return Err("metadata did not contain any font families".to_string());
    }

    for fallback in fallback_public_fonts() {
        if seen.insert(fallback.family.to_ascii_lowercase()) {
            fonts.push(fallback);
        }
    }

    fonts.sort_by(|left, right| {
        left.spotlight_rank
            .unwrap_or(usize::MAX)
            .cmp(&right.spotlight_rank.unwrap_or(usize::MAX))
            .then_with(|| left.family.cmp(&right.family))
    });
    Ok(fonts)
}

fn fallback_public_fonts() -> Vec<PublicFontRecord> {
    FALLBACK_PUBLIC_FONTS
        .iter()
        .map(|entry| PublicFontRecord {
            family: entry.family.to_string(),
            category: entry.category.to_string(),
            source: "fallback_catalog".to_string(),
            declared_license: entry.declared_license.map(str::to_string),
            spotlight_rank: entry.spotlight_rank,
        })
        .collect()
}

fn fallback_overrides() -> HashMap<String, FallbackPublicFont> {
    FALLBACK_PUBLIC_FONTS
        .iter()
        .map(|entry| (entry.family.to_ascii_lowercase(), *entry))
        .collect()
}

fn normalize_query(raw: &str) -> String {
    raw.trim()
        .to_ascii_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn normalize_category(raw: &str) -> String {
    match raw.trim().to_ascii_lowercase().as_str() {
        "sans+serif" | "sans serif" | "sans-serif" | "sans" => "Sans Serif".to_string(),
        "serif" => "Serif".to_string(),
        "display" => "Display".to_string(),
        "handwriting" => "Handwriting".to_string(),
        "monospace" | "mono" => "Monospace".to_string(),
        other if other.is_empty() => "Unknown".to_string(),
        other => {
            let mut chars = other.chars();
            let first = chars.next().unwrap_or_default().to_ascii_uppercase();
            let rest = chars.collect::<String>();
            format!("{first}{rest}")
        }
    }
}

fn score_font_match(
    font: &PublicFontRecord,
    normalized_query: &str,
    tokens: &[String],
) -> Option<(usize, usize)> {
    let family = font.family.to_ascii_lowercase();
    if family == normalized_query {
        return Some((0, font.spotlight_rank.unwrap_or(usize::MAX)));
    }
    if family.starts_with(normalized_query) {
        return Some((1, font.spotlight_rank.unwrap_or(usize::MAX)));
    }

    let family_words = family
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .filter(|word| !word.is_empty())
        .collect::<Vec<_>>();
    let every_token_prefix = tokens
        .iter()
        .all(|token| family_words.iter().any(|word| word.starts_with(token)));
    if every_token_prefix {
        return Some((2, font.spotlight_rank.unwrap_or(usize::MAX)));
    }

    let every_token_substring = tokens.iter().all(|token| family.contains(token));
    if every_token_substring {
        return Some((3, font.spotlight_rank.unwrap_or(usize::MAX)));
    }

    None
}

fn google_fonts_css_url(family: &str) -> Result<reqwest::Url, String> {
    let mut url = reqwest::Url::parse(GOOGLE_FONTS_CSS_URL)
        .map_err(|error| format!("invalid stylesheet base URL: {error}"))?;
    url.query_pairs_mut()
        .append_pair("family", family)
        .append_pair("display", "swap");
    Ok(url)
}

fn extract_stylesheet_font_url(css: &str) -> Option<String> {
    let mut current_subset = String::new();
    let mut fallback_url = None;
    for line in css.lines() {
        let trimmed = line.trim();
        if let Some(comment) = trimmed
            .strip_prefix("/*")
            .and_then(|value| value.strip_suffix("*/"))
        {
            current_subset = comment.trim().to_ascii_lowercase();
            continue;
        }

        let Some(url) = extract_first_url(trimmed) else {
            continue;
        };

        if current_subset.contains("latin") {
            return Some(url);
        }
        if fallback_url.is_none() {
            fallback_url = Some(url);
        }
    }

    fallback_url
}

fn extract_first_url(block: &str) -> Option<String> {
    let start = block.find("url(")?;
    let rest = &block[start + 4..];
    let end = rest.find(')')?;
    Some(
        rest[..end]
            .trim()
            .trim_matches('"')
            .trim_matches('\'')
            .to_string(),
    )
}

fn file_name_for_family(
    family: &str,
    font_url: &str,
    content_type: Option<&str>,
) -> Result<String, String> {
    let url =
        reqwest::Url::parse(font_url).map_err(|error| format!("invalid font file URL: {error}"))?;
    let extension = infer_extension_from_url(&url)
        .or_else(|| infer_extension_from_content_type(content_type))
        .ok_or_else(|| "font file type must be .ttf, .otf, .woff, or .woff2".to_string())?;
    Ok(format!(
        "{}-Regular.{extension}",
        sanitize_family_stem(family)
    ))
}

fn sanitize_family_stem(family: &str) -> String {
    let mut out = String::with_capacity(family.len());
    let mut last_dash = false;
    for ch in family.chars() {
        let mapped = if ch.is_ascii_alphanumeric() {
            Some(ch)
        } else if matches!(ch, ' ' | '-' | '_' | '+') {
            Some('-')
        } else {
            None
        };
        if let Some(mapped) = mapped {
            if mapped == '-' {
                if last_dash {
                    continue;
                }
                last_dash = true;
            } else {
                last_dash = false;
            }
            out.push(mapped);
        }
    }
    out.trim_matches('-').to_string()
}

fn infer_extension_from_url(url: &reqwest::Url) -> Option<String> {
    let extension = url
        .path_segments()
        .and_then(|segments| segments.last())
        .and_then(|segment| segment.rsplit_once('.'))
        .map(|(_, extension)| extension.to_ascii_lowercase())?;
    if matches!(extension.as_str(), "ttf" | "otf" | "woff" | "woff2") {
        Some(extension)
    } else {
        None
    }
}

fn infer_extension_from_content_type(content_type: Option<&str>) -> Option<String> {
    match content_type?.split(';').next()?.trim() {
        "font/ttf" | "application/x-font-ttf" | "application/font-sfnt" => Some("ttf".to_string()),
        "font/otf" | "application/x-font-otf" | "application/vnd.ms-opentype" => {
            Some("otf".to_string())
        }
        "font/woff" | "application/font-woff" => Some("woff".to_string()),
        "font/woff2" | "application/font-woff2" => Some("woff2".to_string()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        PublicFontCatalog, extract_stylesheet_font_url, file_name_for_family,
        parse_google_fonts_metadata, search_public_font_catalog,
    };

    #[test]
    fn parses_google_fonts_metadata_with_xssi_prefix() {
        let fonts = parse_google_fonts_metadata(
            r#")]}'
{"familyMetadataList":[{"family":"Roboto","category":"sans-serif"},{"family":"IBM Plex Sans","category":"sans-serif"}]}
"#
        )
        .unwrap();
        let roboto = fonts.iter().find(|font| font.family == "Roboto").unwrap();
        let plex = fonts
            .iter()
            .find(|font| font.family == "IBM Plex Sans")
            .unwrap();
        assert_eq!(
            roboto.declared_license.as_deref(),
            Some("SIL Open Font License, Version 1.1")
        );
        assert_eq!(
            plex.declared_license.as_deref(),
            Some("SIL Open Font License, Version 1.1")
        );
    }

    #[test]
    fn search_prefers_prefix_matches() {
        let catalog = PublicFontCatalog {
            fonts: parse_google_fonts_metadata(
                ")]}'\n{\"familyMetadataList\":[{\"family\":\"Roboto\",\"category\":\"sans-serif\"},{\"family\":\"Source Serif 4\",\"category\":\"serif\"}]}"
            )
            .unwrap(),
            degraded: false,
            source: "google_fonts_metadata".to_string(),
            note: None,
        };
        let response = search_public_font_catalog(&catalog, "rob", Some(5));
        assert_eq!(
            response.fonts.first().map(|font| font.family.as_str()),
            Some("Roboto")
        );
    }

    #[test]
    fn extracts_latin_font_url_from_stylesheet() {
        let css = r#"
        /* cyrillic */
        @font-face {
          src: url(https://fonts.gstatic.com/s/example-cyrillic.woff2) format('woff2');
        }
        /* latin */
        @font-face {
          src: url(https://fonts.gstatic.com/s/example-latin.woff2) format('woff2');
        }
        "#;
        assert_eq!(
            extract_stylesheet_font_url(css),
            Some("https://fonts.gstatic.com/s/example-latin.woff2".to_string())
        );
    }

    #[test]
    fn builds_stable_family_file_names() {
        assert_eq!(
            file_name_for_family(
                "IBM Plex Sans",
                "https://fonts.gstatic.com/s/plex/v1/font.woff2",
                Some("font/woff2")
            )
            .unwrap(),
            "IBM-Plex-Sans-Regular.woff2"
        );
    }
}
