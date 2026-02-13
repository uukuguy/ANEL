"""LLM integration module - embedder, reranker, and router"""

from abc import ABC, abstractmethod
from dataclasses import dataclass
from enum import Enum
from typing import Optional
import json
import os
import subprocess
from pathlib import Path


class Provider(str, Enum):
    """LLM provider type"""
    LOCAL = "local"
    REMOTE = "remote"


@dataclass
class EmbeddingResult:
    """Embedding result"""
    embeddings: list[list[float]]
    provider: Provider
    model: str


class Embedder(ABC):
    """Base class for embedders"""

    @abstractmethod
    def embed(self, texts: list[str]) -> EmbeddingResult:
        """Generate embeddings for texts"""
        pass


class Reranker(ABC):
    """Base class for rerankers"""

    @abstractmethod
    def rerank(self, query: str, docs: list[str]) -> list[float]:
        """Rerank documents"""
        pass


class LocalEmbedder(Embedder):
    """Local embedding model using llama.cpp"""

    def __init__(self, model_name: str):
        self.model_name = model_name
        self.model_path = str(Path.home() / ".cache" / "qmd" / "models" / model_name)
        if not Path(self.model_path).exists():
            raise FileNotFoundError(f"Model not found: {self.model_path}")

    def embed(self, texts: list[str]) -> EmbeddingResult:
        """Generate embeddings using llama.cpp"""
        # Use llama.cpp CLI for embedding
        result = subprocess.run(
            ["llama-cli", "-m", self.model_path, "--embedding", "true", "-p", texts[0]],
            capture_output=True,
            text=True,
        )

        if result.returncode != 0:
            raise RuntimeError(f"llama-cli failed: {result.stderr}")

        # Parse embedding from output
        embedding = self._parse_embedding(result.stdout)

        return EmbeddingResult(
            embeddings=[embedding],
            provider=Provider.LOCAL,
            model=self.model_name,
        )

    def _parse_embedding(self, output: str) -> list[float]:
        """Parse embedding from llama.cpp output"""
        # Look for [0.1, 0.2, ...] pattern
        start = output.find("[")
        end = output.rfind("]")
        if start == -1 or end == -1:
            raise ValueError("No embedding found in output")

        embed_str = output[start:end + 1]
        return json.loads(embed_str)


class RemoteEmbedder(Embedder):
    """Remote embedding API"""

    def __init__(self, model: str, api_url: str, api_key: str):
        self.model = model
        self.api_url = api_url
        self.api_key = api_key

    def embed(self, texts: list[str]) -> EmbeddingResult:
        """Generate embeddings using remote API"""
        # Placeholder - implement with actual API call
        return EmbeddingResult(
            embeddings=[[0.0] * 768 for _ in texts],
            provider=Provider.REMOTE,
            model=self.model,
        )


class LocalReranker(Reranker):
    """Local reranking model"""

    def __init__(self, model_name: str):
        self.model_name = model_name
        self.model_path = str(Path.home() / ".cache" / "qmd" / "models" / model_name)
        if not Path(self.model_path).exists():
            raise FileNotFoundError(f"Model not found: {self.model_path}")

    def rerank(self, query: str, docs: list[str]) -> list[float]:
        """Rerank documents using local model"""
        # Format: BGE-reranker prompt format
        prompt = f"{query}</s>{docs[0]}"

        result = subprocess.run(
            ["llama-cli", "-m", self.model_path, "-p", prompt],
            capture_output=True,
            text=True,
        )

        if result.returncode != 0:
            raise RuntimeError(f"llama-cli failed: {result.stderr}")

        # Placeholder - needs logit extraction for yes/no token scoring
        scores = [0.5] * len(docs)
        return scores


class RemoteReranker(Reranker):
    """Remote reranking API"""

    def __init__(self, model: str, api_url: str, api_key: str):
        self.model = model
        self.api_url = api_url
        self.api_key = api_key

    def rerank(self, query: str, docs: list[str]) -> list[float]:
        """Rerank documents using remote API"""
        # Placeholder - implement with actual API call
        return [1.0 / (i + 1) for i in range(len(docs))]


class Router:
    """Main LLM router"""

    def __init__(self, config):
        self.config = config
        self.local_embedder: Optional[Embedder] = None
        self.remote_embedder: Optional[Embedder] = None
        self.local_reranker: Optional[Reranker] = None
        self.remote_reranker: Optional[Reranker] = None

    def init_embedder(self):
        """Initialize embedder from config"""
        if not self.config.models or not self.config.models.embed:
            return

        # Try local embedder first
        if self.config.models.embed.local:
            try:
                self.local_embedder = LocalEmbedder(self.config.models.embed.local)
                return
            except FileNotFoundError:
                pass

        # Try remote embedder
        if self.config.models.embed.remote:
            self.remote_embedder = RemoteEmbedder(
                self.config.models.embed.remote,
                self.config.models.embed.get("api_url", ""),
                self.config.models.embed.get("api_key", "")
            )

    def has_embedder(self) -> bool:
        return self.local_embedder is not None or self.remote_embedder is not None

    def has_reranker(self) -> bool:
        return self.local_reranker is not None or self.remote_reranker is not None

    def embed(self, texts: list[str]) -> EmbeddingResult:
        """Generate embeddings"""
        if self.local_embedder:
            return self.local_embedder.embed(texts)
        if self.remote_embedder:
            return self.remote_embedder.embed(texts)
        raise RuntimeError("No embedder available")

    def rerank(self, query: str, docs: list[str]) -> list[float]:
        """Rerank documents"""
        if self.local_reranker:
            return self.local_reranker.rerank(query, docs)
        if self.remote_reranker:
            return self.remote_reranker.rerank(query, docs)
        raise RuntimeError("No reranker available")


class QueryExpander:
    """Query expansion with synonyms"""

    EXPANSION_TERMS = {
        "how": ["how to", "guide", "tutorial"],
        "what": ["what is", "definition", "explanation"],
        "why": ["reason", "explanation", "purpose"],
        "config": ["configuration", "settings", "setup"],
        "install": ["installation", "setup", "deployment"],
        "error": ["error", "issue", "problem", "bug"],
        "api": ["api", "interface", "endpoint"],
        "doc": ["documentation", "docs", "guide"],
    }

    def expand_query(self, query: str) -> list[str]:
        """Expand query with synonyms"""
        words = query.lower().split()
        expanded = [query]

        for word in words:
            if word in self.EXPANSION_TERMS:
                expanded.extend(self.EXPANSION_TERMS[word])

        return expanded
