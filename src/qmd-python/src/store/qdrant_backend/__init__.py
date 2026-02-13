"""Qdrant backend for QMD vector search."""

from dataclasses import dataclass
from typing import List, Optional
from qdrant_client import QdrantClient
from qdrant_client.models import Distance, VectorParams, PointStruct


@dataclass
class QdrantConfig:
    """Qdrant configuration."""

    url: str = "http://localhost:6333"
    api_key: Optional[str] = None
    collection: str = "qmd_documents"
    vector_size: int = 384


class QdrantBackend:
    """Qdrant vector database backend."""

    def __init__(self, config: QdrantConfig):
        """Initialize Qdrant backend."""
        self.config = config
        self.client = QdrantClient(
            url=config.url,
            api_key=config.api_key,
        )
        self._ensure_collection()

    def _ensure_collection(self) -> None:
        """Ensure collection exists, create if not."""
        collections = self.client.get_collections().collections
        collection_names = [c.name for c in collections]

        if self.config.collection not in collection_names:
            self.client.create_collection(
                collection_name=self.config.collection,
                vectors_config=VectorParams(
                    size=self.config.vector_size,
                    distance=Distance.COSINE,
                ),
            )

    def upsert_vectors(
        self,
        vectors: List[dict],
    ) -> None:
        """Upsert vectors into Qdrant."""
        points = []
        for v in vectors:
            points.append(
                PointStruct(
                    id=v["id"],
                    vector=v["vector"],
                    payload={
                        "path": v.get("path", ""),
                        "title": v.get("title", ""),
                        "body": v.get("body", ""),
                        "hash": v.get("hash", ""),
                        "collection": v.get("collection", ""),
                    },
                )
            )

        if points:
            self.client.upsert(
                collection_name=self.config.collection,
                points=points,
            )

    def search(
        self,
        query_vector: List[float],
        limit: int = 10,
    ) -> List[dict]:
        """Search vectors."""
        results = self.client.search(
            collection_name=self.config.collection,
            query_vector=query_vector,
            limit=limit,
        )

        return [
            {
                "id": r.id,
                "score": r.score,
                "path": r.payload.get("path", ""),
                "title": r.payload.get("title", ""),
                "body": r.payload.get("body", ""),
                "hash": r.payload.get("hash", ""),
                "collection": r.payload.get("collection", ""),
            }
            for r in results
        ]

    def delete_collection(self) -> None:
        """Delete collection."""
        self.client.delete_collection(collection_name=self.config.collection)
