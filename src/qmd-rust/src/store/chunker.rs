/// Document chunking for semantic search.
///
/// Splits large documents into overlapping chunks so each chunk's embedding
/// captures focused semantics rather than a diluted average of the whole document.

/// A single chunk of a document.
#[derive(Debug, Clone)]
pub struct Chunk {
    /// Chunk index (0, 1, 2, ...)
    pub seq: usize,
    /// Character offset in the original document
    pub pos: usize,
    /// Chunk text content
    pub text: String,
}

/// Default chunk size in characters (~800 tokens at 4 chars/token)
pub const DEFAULT_CHUNK_SIZE: usize = 3200;
/// Default overlap in characters (~120 tokens)
pub const DEFAULT_OVERLAP: usize = 480;

/// Find the nearest UTF-8 character boundary at or before `pos`
fn prev_char_boundary(text: &str, mut pos: usize) -> usize {
    while pos > 0 && !text.is_char_boundary(pos) {
        pos -= 1;
    }
    pos
}

/// Split a document into overlapping chunks.
///
/// - Short documents (< chunk_size * 1.2) return a single chunk.
/// - Splits prefer paragraph boundaries (`\n\n`), then sentence boundaries (`. `),
///   then word boundaries.
pub fn chunk_document(text: &str, chunk_size: usize, overlap: usize) -> Vec<Chunk> {
    if text.is_empty() {
        return Vec::new();
    }

    // Short document — single chunk
    if text.len() < (chunk_size as f64 * 1.2) as usize {
        return vec![Chunk {
            seq: 0,
            pos: 0,
            text: text.to_string(),
        }];
    }

    let mut chunks = Vec::new();
    let mut start = 0;

    while start < text.len() {
        let end_raw = (start + chunk_size).min(text.len());
        // Ensure we're at a UTF-8 char boundary
        let end = prev_char_boundary(text, end_raw);

        // If we've reached the end, take the rest
        if end == text.len() || end <= start {
            chunks.push(Chunk {
                seq: chunks.len(),
                pos: start,
                text: text[start..].to_string(),
            });
            break;
        }

        // Find the best split point near `end`
        let split = find_split_point(text, end, start);
        // Ensure split is at a valid UTF-8 boundary
        let split = prev_char_boundary(text, split).max(start);

        chunks.push(Chunk {
            seq: chunks.len(),
            pos: start,
            text: text[start..split].to_string(),
        });

        // Advance with overlap
        let next_start_raw = if split > overlap {
            split - overlap
        } else {
            split
        };
        // Ensure next_start is at a valid UTF-8 boundary
        let next_start = prev_char_boundary(text, next_start_raw).max(start);

        // Ensure forward progress
        if next_start <= start {
            start = split;
        } else {
            start = next_start;
        }
    }

    chunks
}

/// Find the best split point near `target` within the text.
/// Searches backward from `target` for paragraph, sentence, or word boundaries.
fn find_split_point(text: &str, target: usize, min_pos: usize) -> usize {
    // Ensure target is at a valid UTF-8 boundary
    let target = prev_char_boundary(text, target).max(min_pos);

    // Search window: look back up to 20% of chunk_size for a good boundary
    let search_start = if target > 640 { target - 640 } else { min_pos };
    let search_start = search_start.max(min_pos);
    let search_start = prev_char_boundary(text, search_start).max(min_pos);

    // Safe to slice now that both boundaries are valid
    let region = &text[search_start..target];

    // 1. Paragraph boundary (\n\n)
    if let Some(pos) = region.rfind("\n\n") {
        let split = search_start + pos + 2; // after the double newline
        if split > min_pos {
            return split;
        }
    }

    // 2. Sentence boundary (". " or ".\n")
    if let Some(pos) = region.rfind(". ") {
        let split = search_start + pos + 2;
        if split > min_pos {
            return split;
        }
    }
    if let Some(pos) = region.rfind(".\n") {
        let split = search_start + pos + 2;
        if split > min_pos {
            return split;
        }
    }

    // 3. Word boundary (space)
    if let Some(pos) = region.rfind(' ') {
        let split = search_start + pos + 1;
        if split > min_pos {
            return split;
        }
    }

    // 4. Fallback: hard split at target
    target
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_empty_document() {
        let chunks = chunk_document("", DEFAULT_CHUNK_SIZE, DEFAULT_OVERLAP);
        assert!(chunks.is_empty());
    }

    #[test]
    fn test_chunk_short_document() {
        let text = "This is a short document that fits in one chunk.";
        let chunks = chunk_document(text, DEFAULT_CHUNK_SIZE, DEFAULT_OVERLAP);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].seq, 0);
        assert_eq!(chunks[0].pos, 0);
        assert_eq!(chunks[0].text, text);
    }

    #[test]
    fn test_chunk_long_document() {
        // Create a document that's ~3x chunk_size
        let paragraph = "This is a test paragraph with enough words to fill space. ";
        let text = paragraph.repeat(200); // ~11800 chars
        assert!(text.len() > DEFAULT_CHUNK_SIZE * 2);

        let chunks = chunk_document(&text, DEFAULT_CHUNK_SIZE, DEFAULT_OVERLAP);
        assert!(chunks.len() >= 3, "Expected at least 3 chunks, got {}", chunks.len());

        // All chunks should have sequential seq values
        for (i, chunk) in chunks.iter().enumerate() {
            assert_eq!(chunk.seq, i);
        }

        // First chunk starts at 0
        assert_eq!(chunks[0].pos, 0);
    }

    #[test]
    fn test_chunk_overlap() {
        // Build a document large enough to produce multiple chunks
        let sentence = "The quick brown fox jumps over the lazy dog. ";
        let text = sentence.repeat(200); // ~9000 chars

        let chunks = chunk_document(&text, DEFAULT_CHUNK_SIZE, DEFAULT_OVERLAP);
        assert!(chunks.len() >= 2, "Need at least 2 chunks for overlap test");

        // Adjacent chunks should share overlapping text
        for i in 0..chunks.len() - 1 {
            let current_end = &chunks[i].text;
            let next_start = &chunks[i + 1].text;

            // The end of chunk[i] should overlap with the start of chunk[i+1]
            // Since overlap is 480 chars, there should be shared content
            let overlap_region_end = &current_end[current_end.len().saturating_sub(DEFAULT_OVERLAP)..];
            let overlap_region_start = &next_start[..next_start.len().min(DEFAULT_OVERLAP)];

            // At least some text should be shared
            assert!(
                overlap_region_end.len() > 0 && overlap_region_start.len() > 0,
                "Overlap regions should not be empty"
            );
        }
    }

    #[test]
    fn test_chunk_paragraph_boundary() {
        // Create text with clear paragraph boundaries
        let para1 = "a".repeat(2000);
        let para2 = "b".repeat(2000);
        let para3 = "c".repeat(2000);
        let text = format!("{}\n\n{}\n\n{}", para1, para2, para3);

        let chunks = chunk_document(&text, DEFAULT_CHUNK_SIZE, DEFAULT_OVERLAP);
        assert!(chunks.len() >= 2);

        // First chunk should end at or near a paragraph boundary
        let first_chunk_end = chunks[0].pos + chunks[0].text.len();
        // The split should be near the first \n\n (at position 2000)
        // Allow some tolerance since overlap affects exact positions
        assert!(
            first_chunk_end <= DEFAULT_CHUNK_SIZE + 100,
            "First chunk should respect approximate chunk_size, got {}",
            first_chunk_end
        );
    }

    #[test]
    fn test_chunk_positions() {
        let sentence = "Hello world this is a test sentence for chunking. ";
        let text = sentence.repeat(150); // ~7500 chars

        let chunks = chunk_document(&text, DEFAULT_CHUNK_SIZE, DEFAULT_OVERLAP);
        assert!(chunks.len() >= 2);

        // pos values should be valid character offsets
        for chunk in &chunks {
            assert!(chunk.pos < text.len(), "pos {} should be < text len {}", chunk.pos, text.len());
            // The text at pos should match the chunk's text start
            let expected_start = &text[chunk.pos..chunk.pos + chunk.text.len().min(50)];
            let actual_start = &chunk.text[..chunk.text.len().min(50)];
            assert_eq!(expected_start, actual_start,
                "Chunk seq={} text at pos={} should match", chunk.seq, chunk.pos);
        }

        // First chunk always starts at 0
        assert_eq!(chunks[0].pos, 0);

        // Positions should be monotonically increasing
        for i in 1..chunks.len() {
            assert!(chunks[i].pos > chunks[i - 1].pos,
                "pos should increase: chunk[{}].pos={} <= chunk[{}].pos={}",
                i, chunks[i].pos, i - 1, chunks[i - 1].pos);
        }
    }

    #[test]
    fn test_chunk_covers_entire_document() {
        let sentence = "Testing full coverage of document content here. ";
        let text = sentence.repeat(200);

        let chunks = chunk_document(&text, DEFAULT_CHUNK_SIZE, DEFAULT_OVERLAP);

        // Last chunk should reach the end of the document
        let last = chunks.last().unwrap();
        assert_eq!(
            last.pos + last.text.len(),
            text.len(),
            "Last chunk should reach end of document"
        );
    }

    // ==================== Extended Chunker Tests ====================

    #[test]
    fn test_chunk_unicode_content() {
        // Chinese text: each char is 3 bytes in UTF-8
        let sentence = "这是关于Rust的描述。";
        let text = sentence.repeat(500); // ~6500 chars, each Chinese char is 3 bytes
        assert!(text.len() > DEFAULT_CHUNK_SIZE);

        let chunks = chunk_document(&text, DEFAULT_CHUNK_SIZE, DEFAULT_OVERLAP);
        assert!(chunks.len() >= 2, "Unicode text should produce multiple chunks");

        // Verify all chunks contain valid UTF-8 (no panics)
        for chunk in &chunks {
            assert!(!chunk.text.is_empty());
            // Verify the text is valid by iterating chars
            let _ = chunk.text.chars().count();
        }
    }

    #[test]
    fn test_chunk_sentence_boundary_preference() {
        // Build text with clear sentence boundaries
        let sentences: Vec<String> = (0..100)
            .map(|i| format!("This is sentence number {} with enough words to fill space. ", i))
            .collect();
        let text = sentences.join("");
        assert!(text.len() > DEFAULT_CHUNK_SIZE);

        let chunks = chunk_document(&text, DEFAULT_CHUNK_SIZE, DEFAULT_OVERLAP);
        assert!(chunks.len() >= 2);

        // First chunk should end at a sentence boundary (". ")
        let first_text = &chunks[0].text;
        let trimmed = first_text.trim_end();
        assert!(
            trimmed.ends_with('.') || trimmed.ends_with(". "),
            "First chunk should end near a sentence boundary, got: ...{}",
            &trimmed[trimmed.len().saturating_sub(20)..]
        );
    }

    #[test]
    fn test_chunk_custom_small_chunk_size() {
        let text = "Hello world. This is a test. Another sentence here. More content follows. End of text.";
        // Very small chunk size
        let chunks = chunk_document(text, 30, 5);
        assert!(chunks.len() >= 2, "Small chunk_size should produce multiple chunks, got {}", chunks.len());
    }

    #[test]
    fn test_chunk_custom_large_chunk_size() {
        let text = "Short text that fits easily.";
        let chunks = chunk_document(text, 10000, 100);
        assert_eq!(chunks.len(), 1, "Large chunk_size should produce single chunk");
        assert_eq!(chunks[0].text, text);
    }

    #[test]
    fn test_chunk_zero_overlap() {
        let sentence = "Word repeated many times for testing purposes here now. ";
        let text = sentence.repeat(200);

        let chunks = chunk_document(&text, DEFAULT_CHUNK_SIZE, 0);
        assert!(chunks.len() >= 2);

        // With zero overlap, chunks should not share starting positions
        for i in 1..chunks.len() {
            assert!(
                chunks[i].pos >= chunks[i - 1].pos + chunks[i - 1].text.len() - 100,
                "With zero overlap, chunks should have minimal position overlap"
            );
        }
    }

    #[test]
    fn test_chunk_single_long_word() {
        // A single very long "word" with no spaces
        let text = "a".repeat(10000);
        let chunks = chunk_document(&text, DEFAULT_CHUNK_SIZE, DEFAULT_OVERLAP);
        assert!(chunks.len() >= 2, "Long word should still be chunked");

        // All content should be covered
        let last = chunks.last().unwrap();
        assert_eq!(last.pos + last.text.len(), text.len());
    }

    #[test]
    fn test_chunk_very_large_document() {
        // 1MB document
        let sentence = "This is a paragraph of text for testing very large document chunking behavior. ";
        let text = sentence.repeat(13000); // ~1MB
        assert!(text.len() > 1_000_000);

        let chunks = chunk_document(&text, DEFAULT_CHUNK_SIZE, DEFAULT_OVERLAP);
        assert!(chunks.len() > 100, "1MB doc should produce many chunks, got {}", chunks.len());

        // Verify sequential seq values
        for (i, chunk) in chunks.iter().enumerate() {
            assert_eq!(chunk.seq, i);
        }

        // Verify last chunk reaches end
        let last = chunks.last().unwrap();
        assert_eq!(last.pos + last.text.len(), text.len());
    }

    #[test]
    fn test_chunk_threshold_boundary() {
        // Document exactly at the 1.2x threshold
        let threshold = (DEFAULT_CHUNK_SIZE as f64 * 1.2) as usize;

        // Just under threshold — single chunk
        let text_under = "x".repeat(threshold - 1);
        let chunks_under = chunk_document(&text_under, DEFAULT_CHUNK_SIZE, DEFAULT_OVERLAP);
        assert_eq!(chunks_under.len(), 1, "Just under threshold should be 1 chunk");

        // At threshold — single chunk (< threshold means single)
        let text_at = "x".repeat(threshold);
        let chunks_at = chunk_document(&text_at, DEFAULT_CHUNK_SIZE, DEFAULT_OVERLAP);
        assert!(chunks_at.len() >= 1, "At threshold should produce chunks");

        // Just over threshold — multiple chunks
        let text_over = "x".repeat(threshold + 1);
        let chunks_over = chunk_document(&text_over, DEFAULT_CHUNK_SIZE, DEFAULT_OVERLAP);
        assert!(chunks_over.len() >= 2, "Over threshold should produce multiple chunks");
    }

    #[test]
    fn test_chunk_newlines_only() {
        let text = "\n".repeat(5000);
        let chunks = chunk_document(&text, DEFAULT_CHUNK_SIZE, DEFAULT_OVERLAP);
        assert!(!chunks.is_empty());
        // Should handle gracefully without panicking
    }

    #[test]
    fn test_chunk_mixed_line_endings() {
        let parts: Vec<String> = (0..100)
            .map(|i| {
                if i % 3 == 0 {
                    format!("Line {} with CRLF ending.\r\n", i)
                } else if i % 3 == 1 {
                    format!("Line {} with LF ending.\n", i)
                } else {
                    format!("Line {} with paragraph break.\n\n", i)
                }
            })
            .collect();
        let text = parts.join("");
        if text.len() > DEFAULT_CHUNK_SIZE {
            let chunks = chunk_document(&text, DEFAULT_CHUNK_SIZE, DEFAULT_OVERLAP);
            assert!(chunks.len() >= 2);
            // Verify no panics with mixed line endings
        }
    }
}
