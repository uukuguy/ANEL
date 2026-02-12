package store

import (
	"unicode/utf8"
)

// Chunk represents a document chunk
type Chunk struct {
	Seq int
	Pos int
	Text string
}

const (
	DefaultChunkSize = 3200
	DefaultOverlap   = 480
)

// ChunkDocument splits a document into chunks
func ChunkDocument(text string, chunkSize, overlap int) []Chunk {
	if chunkSize <= 0 {
		chunkSize = DefaultChunkSize
	}
	if overlap <= 0 {
		overlap = DefaultOverlap
	}

	if len(text) <= chunkSize {
		return []Chunk{{Seq: 0, Pos: 0, Text: text}}
	}

	var chunks []Chunk
	pos := 0
	seq := 0

	for pos < len(text) {
		end := pos + chunkSize
		if end > len(text) {
			end = len(text)
		}

		// Try to break at sentence boundary
		if end < len(text) {
			// Look for sentence endings: . ! ? followed by space or newline
			for i := end - 1; i > end-200 && i > pos; i-- {
				if text[i] == '.' || text[i] == '!' || text[i] == '?' {
					if i+1 < len(text) && (text[i+1] == ' ' || text[i+1] == '\n') {
						end = i + 1
						break
					}
				}
			}
		}

		// Ensure we don't break in the middle of a UTF-8 character
		for !utf8.RuneStart(text[end]) && end > pos {
			end--
		}

		chunk := Chunk{
			Seq: seq,
			Pos: pos,
			Text: text[pos:end],
		}
		chunks = append(chunks, chunk)

		seq++
		pos = end - overlap
		if pos <= 0 {
			break
		}
	}

	return chunks
}

// CountTokens estimates token count (approximate: 1 token ≈ 4 chars)
func CountTokens(text string) int {
	return len(text) / 4
}
