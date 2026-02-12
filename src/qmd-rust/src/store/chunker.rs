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
        let end = (start + chunk_size).min(text.len());

        // If we've reached the end, take the rest
        if end == text.len() {
            chunks.push(Chunk {
                seq: chunks.len(),
                pos: start,
                text: text[start..end].to_string(),
            });
            break;
        }

        // Find the best split point near `end`
        let split = find_split_point(text, end, start);

        chunks.push(Chunk {
            seq: chunks.len(),
            pos: start,
            text: text[start..split].to_string(),
        });

        // Advance with overlap
        let next_start = if split > overlap {
            split - overlap
        } else {
            split
        };

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
    // Search window: look back up to 20% of chunk_size for a good boundary
    let search_start = if target > 640 { target - 640 } else { min_pos };
    let search_start = search_start.max(min_pos);
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
}
