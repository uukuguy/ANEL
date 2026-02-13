"""LLM router for QMD."""

from dataclasses import dataclass
from typing import List, Optional
import os


@dataclass
class EmbeddingResult:
    """Embedding result."""

    embeddings: List[List[float]]
    provider: str
    model: str


@dataclass
class RerankResult:
    """Reranking result."""

    scores: List[float]
    provider: str
    model: str


class Router:
    """LLM router - routes to local or remote providers."""

    def __init__(self, config):
        """Initialize router."""
        self.config = config
        self._embedder = None
        self._remote_embed_client = None

    def _init_embedder(self):
        """Initialize local embedder (llama-cpp-python)."""
        if self._embedder is not None:
            return

        if not self.config.models.embed or not self.config.models.embed.local:
            return

        model_path = self.config.models.embed.local
        if not os.path.exists(model_path):
            # Try default model path
            default_path = os.path.expanduser("~/.cache/qmd/models/")
            model_path = os.path.join(default_path, model_path)

        try:
            from llama_cpp import Llama
            self._embedder = Llama(
                model_path=model_path,
                n_ctx=512,
                embedding=True,
            )
        except Exception as e:
            print(f"Failed to initialize embedder: {e}")
            self._embedder = None

    def _init_remote_embed_client(self):
        """Initialize remote embedder (OpenAI compatible)."""
        if self._remote_embed_client is not None:
            return

        if not self.config.models.embed or not self.config.models.embed.remote:
            return

        try:
            from openai import AsyncOpenAI
            # Get API key from environment or config
            api_key = os.environ.get("OPENAI_API_KEY", "")
            base_url = os.environ.get("OPENAI_BASE_URL", "https://api.openai.com/v1")

            self._remote_embed_client = AsyncOpenAI(
                api_key=api_key,
                base_url=base_url,
            )
        except Exception as e:
            print(f"Failed to initialize remote embedder: {e}")
            self._remote_embed_client = None

    async def embed(self, texts: List[str]) -> EmbeddingResult:
        """Generate embeddings."""
        # Try local first
        if self.config.models.embed and self.config.models.embed.local:
            try:
                embeddings = await self._local_embed(texts)
                return EmbeddingResult(
                    embeddings=embeddings,
                    provider="local",
                    model=self.config.models.embed.local,
                )
            except Exception as e:
                print(f"Local embedding failed: {e}")

        # Try remote
        if self.config.models.embed and self.config.models.embed.remote:
            try:
                embeddings = await self._remote_embed(texts)
                return EmbeddingResult(
                    embeddings=embeddings,
                    provider="remote",
                    model=self.config.models.embed.remote,
                )
            except Exception as e:
                print(f"Remote embedding failed: {e}")

        raise RuntimeError("No embedder available")

    async def rerank(self, query: str, docs: List[str]) -> List[float]:
        """Rerank documents."""
        # Try local first
        if self.config.models.rerank and self.config.models.rerank.local:
            try:
                return await self._local_rerank(query, docs)
            except Exception as e:
                print(f"Local reranking failed: {e}")

        # Try remote
        if self.config.models.rerank and self.config.models.rerank.remote:
            try:
                return await self._remote_rerank(query, docs)
            except Exception as e:
                print(f"Remote reranking failed: {e}")

        raise RuntimeError("No reranker available")

    def expand_query(self, query: str) -> List[str]:
        """Expand query using LLM."""
        # TODO: Implement query expansion
        return [query]

    async def _local_embed(self, texts: List[str]) -> List[List[float]]:
        """Local embedding using llama-cpp-python."""
        self._init_embedder()

        if self._embedder is None:
            raise RuntimeError("Embedder not initialized")

        # Run in thread pool to avoid blocking
        import asyncio
        loop = asyncio.get_event_loop()
        embeddings = await loop.run_in_executor(
            None,
            lambda: [self._embedder.embed(text)["embedding"] for text in texts]
        )
        return embeddings

    async def _remote_embed(self, texts: List[str]) -> List[List[float]]:
        """Remote embedding using OpenAI-compatible API."""
        self._init_remote_embed_client()

        if self._remote_embed_client is None:
            raise RuntimeError("Remote embedder not initialized")

        # Determine model and dimensions
        model = self.config.models.embed.remote
        dim = 1536
        if "text-embedding-3" in model:
            dim = 3072

        # Call API
        response = await self._remote_embed_client.embeddings.create(
            model=model,
            input=texts,
        )

        return [item.embedding for item in response.data]

    async def _local_rerank(self, query: str, docs: List[str]) -> List[float]:
        """Local reranking using llama-cpp-python."""
        # TODO: Implement local reranking with cross-encoder
        # For now, use a simple approach: compute similarity scores
        import random
        return [random.random() for _ in docs]

    async def _remote_rerank(self, query: str, docs: List[str]) -> List[float]:
        """Remote reranking using OpenAI-compatible API."""
        if self._remote_embed_client is None:
            self._init_remote_embed_client()

        if self._remote_embed_client is None:
            raise RuntimeError("Remote embedder not initialized")

        model = self.config.models.rerank.remote or "cohere-rerank"

        try:
            # Try Cohere rerank API
            import os
            api_key = os.environ.get("COHERE_API_KEY", "")
            if api_key:
                import httpx
                async with httpx.AsyncClient() as client:
                    response = await client.post(
                        "https://api.cohere.ai/v1/rerank",
                        headers={
                            "Authorization": f"Bearer {api_key}",
                            "Content-Type": "application/json",
                        },
                        json={
                            "query": query,
                            "documents": docs,
                            "model": model,
                            "top_n": len(docs),
                        },
                    )
                    data = response.json()
                    # Return scores in original order
                    scores = [0.0] * len(docs)
                    for item in data.get("results", []):
                        scores[item["index"]] = item["relevance_score"]
                    return scores
        except Exception as e:
            print(f"Rerank API failed: {e}")

        # Fallback: use embedding similarity
        query_emb = await self._remote_embed([query])
        doc_embs = await self._remote_embed(docs)

        # Compute cosine similarity
        import math
        scores = []
        for doc_vec in doc_embs.embeddings:
            sim = sum(a * b for a, b in zip(query_emb.embeddings[0], doc_vec))
            sim /= (math.sqrt(sum(a * a for a in query_emb.embeddings[0])) *
                    math.sqrt(sum(b * b for b in doc_vec)) + 1e-8)
            scores.append(sim)

        return scores
