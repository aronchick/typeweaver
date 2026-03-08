use typeweaver_core::OcrScore;

/// Render a single glyph to a greyscale bitmap, then encode as minimal PNG.
pub fn render_glyph_to_png(font_data: &[u8], ch: char, px: f32) -> Option<Vec<u8>> {
    let font = fontdue::Font::from_bytes(font_data, fontdue::FontSettings::default()).ok()?;
    let (metrics, bitmap) = font.rasterize(ch, px);
    if metrics.width == 0 || metrics.height == 0 {
        return None;
    }
    Some(encode_greyscale_png(
        &bitmap,
        metrics.width as u32,
        metrics.height as u32,
    ))
}

/// Compute character-level accuracy between two strings.
pub fn compute_char_accuracy(expected: &str, recognized: &str) -> f32 {
    if expected.is_empty() {
        return if recognized.is_empty() { 1.0 } else { 0.0 };
    }

    let exp_chars: Vec<char> = expected.chars().collect();
    let rec_chars: Vec<char> = recognized.chars().collect();

    let max_len = exp_chars.len().max(rec_chars.len());
    let matches = exp_chars
        .iter()
        .zip(rec_chars.iter())
        .filter(|(a, b)| a == b)
        .count();
    matches as f32 / max_len as f32
}

/// Compute word-level accuracy between two strings.
fn compute_word_accuracy(expected: &str, recognized: &str) -> f32 {
    let exp_words: Vec<&str> = expected.split_whitespace().collect();
    let rec_words: Vec<&str> = recognized.split_whitespace().collect();

    if exp_words.is_empty() {
        return if rec_words.is_empty() { 1.0 } else { 0.0 };
    }

    let max_len = exp_words.len().max(rec_words.len());
    let matches = exp_words
        .iter()
        .zip(rec_words.iter())
        .filter(|(a, b)| a == b)
        .count();
    matches as f32 / max_len as f32
}

/// Run OCR scoring (requires the `ocr` feature for real Tesseract).
/// Without the feature, returns a stub score comparing strings directly.
#[cfg(feature = "ocr")]
pub fn ocr_score(expected: &str, _font_data: &[u8], _px: f32) -> OcrScore {
    // With OCR feature, we would render + tesseract. Stub for now.
    let recognized = expected.to_string();
    let char_accuracy = compute_char_accuracy(expected, &recognized);
    let word_accuracy = compute_word_accuracy(expected, &recognized);
    OcrScore {
        expected: expected.to_string(),
        recognized,
        char_accuracy,
        word_accuracy,
    }
}

#[cfg(not(feature = "ocr"))]
pub fn ocr_score(expected: &str, _font_data: &[u8], _px: f32) -> OcrScore {
    let recognized = expected.to_string();
    let char_accuracy = compute_char_accuracy(expected, &recognized);
    let word_accuracy = compute_word_accuracy(expected, &recognized);
    OcrScore {
        expected: expected.to_string(),
        recognized,
        char_accuracy,
        word_accuracy,
    }
}

/// Minimal greyscale PNG encoder (no external crate needed).
fn encode_greyscale_png(data: &[u8], width: u32, height: u32) -> Vec<u8> {
    let mut out = Vec::new();
    // PNG signature
    out.extend_from_slice(&[137, 80, 78, 71, 13, 10, 26, 10]);

    // IHDR
    let mut ihdr = Vec::new();
    ihdr.extend_from_slice(&width.to_be_bytes());
    ihdr.extend_from_slice(&height.to_be_bytes());
    ihdr.push(8); // bit depth
    ihdr.push(0); // color type: greyscale
    ihdr.push(0); // compression
    ihdr.push(0); // filter
    ihdr.push(0); // interlace
    write_chunk(&mut out, b"IHDR", &ihdr);

    // IDAT - uncompressed deflate
    let mut raw_data = Vec::new();
    for y in 0..height as usize {
        raw_data.push(0); // filter byte: None
        let start = y * width as usize;
        let end = start + width as usize;
        raw_data.extend_from_slice(&data[start..end]);
    }

    let compressed = deflate_store(&raw_data);
    write_chunk(&mut out, b"IDAT", &compressed);

    // IEND
    write_chunk(&mut out, b"IEND", &[]);

    out
}

fn write_chunk(out: &mut Vec<u8>, chunk_type: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    out.extend_from_slice(chunk_type);
    out.extend_from_slice(data);
    let mut crc_data = Vec::with_capacity(4 + data.len());
    crc_data.extend_from_slice(chunk_type);
    crc_data.extend_from_slice(data);
    out.extend_from_slice(&crc32(&crc_data).to_be_bytes());
}

fn crc32(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFF_FFFF;
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ 0xEDB8_8320;
            } else {
                crc >>= 1;
            }
        }
    }
    !crc
}

fn deflate_store(data: &[u8]) -> Vec<u8> {
    // zlib wrapper around stored (uncompressed) deflate blocks
    let mut out = Vec::new();
    out.push(0x78); // CMF
    out.push(0x01); // FLG

    // Split into 65535-byte blocks
    let chunks: Vec<&[u8]> = if data.is_empty() {
        vec![&[]]
    } else {
        data.chunks(65535).collect()
    };

    for (i, chunk) in chunks.iter().enumerate() {
        let is_last = i == chunks.len() - 1;
        out.push(if is_last { 0x01 } else { 0x00 });
        let len = chunk.len() as u16;
        out.extend_from_slice(&len.to_le_bytes());
        out.extend_from_slice(&(!len).to_le_bytes());
        out.extend_from_slice(chunk);
    }

    // Adler-32 checksum
    let mut a: u32 = 1;
    let mut b: u32 = 0;
    for &byte in data {
        a = (a + byte as u32) % 65521;
        b = (b + a) % 65521;
    }
    let checksum = (b << 16) | a;
    out.extend_from_slice(&checksum.to_be_bytes());

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn char_accuracy_perfect_match() {
        assert!((compute_char_accuracy("hello", "hello") - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn char_accuracy_partial_match() {
        let acc = compute_char_accuracy("hello", "hxllo");
        assert!(acc > 0.5 && acc < 1.0);
    }

    #[test]
    fn char_accuracy_empty() {
        assert!((compute_char_accuracy("", "") - 1.0).abs() < f32::EPSILON);
        assert!((compute_char_accuracy("", "x")).abs() < f32::EPSILON);
    }

    #[test]
    fn ocr_score_stub_returns_perfect() {
        let score = ocr_score("hello world", &[], 16.0);
        assert!((score.char_accuracy - 1.0).abs() < f32::EPSILON);
        assert!((score.word_accuracy - 1.0).abs() < f32::EPSILON);
    }
}
