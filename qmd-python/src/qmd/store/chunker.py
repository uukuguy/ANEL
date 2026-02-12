"""Document chunking utilities"""

from dataclasses import dataclass


@dataclass
class Chunk:
    """Represents a document chunk"""
    seq: int
    pos: int
    text: str


DEFAULT_CHUNK_SIZE = 3200  # ~800 tokens
DEFAULT_OVERLAP = 480      # ~120 tokens


def chunk_document(text: str, chunk_size: int = DEFAULT_CHUNK_SIZE, overlap: int = DEFAULT_OVERLAP) -> list[Chunk]:
    """Split a document into overlapping chunks"""
    if len(text) <= chunk_size:
        return [Chunk(seq=0, pos=0, text=text)]

    chunks = []
    pos = 0
    seq = 0

    while pos < len(text):
        end = pos + chunk_size
        if end > len(text):
            end = len(text)

        # Try to break at sentence boundary
        if end < len(text):
            # Look for sentence endings: . ! ? followed by space or newline
            for i in range(end - 1, max(end - 200, pos), -1):
                if text[i] in '.!?' and i + 1 < len(text) and text[i + 1] in ' \n':
                    end = i + 1
                    break

        chunk = Chunk(seq=seq, pos=pos, text=text[pos:end])
        chunks.append(chunk)

        seq += 1
        pos = end - overlap
        if pos <= 0:
            break

    return chunks


def count_tokens(text: str) -> int:
    """Estimate token count (approximate: 1 token ≈ 4 chars)"""
    return len(text) // 4
