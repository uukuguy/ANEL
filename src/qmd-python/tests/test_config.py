"""Tests for QMD configuration module."""

import pytest
from pathlib import Path

import sys
sys.path.insert(0, str(__import__("pathlib").Path(__file__).parent.parent / "src"))

from config.mod import (
    Config,
    CollectionConfig,
    BM25Config,
    VectorConfig,
    QdrantConfig,
    LLMModelConfig,
    ModelsConfig,
    DEFAULT_CONFIG_PATH,
    DEFAULT_CACHE_PATH,
)


class TestDefaultConfig:
    def test_bm25_backend(self):
        cfg = Config()
        assert cfg.bm25.backend == "sqlite_fts5"

    def test_vector_backend(self):
        cfg = Config()
        assert cfg.vector.backend == "qmd_builtin"

    def test_vector_model(self):
        cfg = Config()
        assert cfg.vector.model == "embeddinggemma-300M"

    def test_qdrant_defaults(self):
        cfg = Config()
        assert cfg.vector.qdrant.url == "http://localhost:6333"
        assert cfg.vector.qdrant.collection == "qmd_documents"
        assert cfg.vector.qdrant.vector_size == 384
        assert cfg.vector.qdrant.api_key is None

    def test_empty_collections(self):
        cfg = Config()
        assert cfg.collections == []

    def test_cache_path(self):
        cfg = Config()
        assert cfg.cache_path == Path("~/.cache/qmd")

    def test_models_default_none(self):
        cfg = Config()
        assert cfg.models.embed is None
        assert cfg.models.rerank is None
        assert cfg.models.query_expansion is None


class TestConfigFromDict:
    def test_minimal(self):
        cfg = Config.from_dict({"bm25": {"backend": "sqlite_fts5"}})
        assert cfg.bm25.backend == "sqlite_fts5"
        # Defaults preserved
        assert cfg.vector.backend == "qmd_builtin"

    def test_with_collections(self):
        cfg = Config.from_dict({
            "collections": [
                {"name": "notes", "path": "~/notes", "pattern": "**/*.md"},
                {"name": "docs", "path": "~/docs"},
            ]
        })
        assert len(cfg.collections) == 2
        assert cfg.collections[0].name == "notes"
        assert cfg.collections[0].pattern == "**/*.md"
        assert cfg.collections[1].name == "docs"
        assert cfg.collections[1].pattern is None

    def test_vector_backend_override(self):
        cfg = Config.from_dict({
            "vector": {
                "backend": "qdrant",
                "model": "embeddinggemma-300M",
            }
        })
        assert cfg.vector.backend == "qdrant"
        assert cfg.vector.model == "embeddinggemma-300M"

    def test_custom_cache_path(self):
        cfg = Config.from_dict({"cache_path": "/tmp/qmd-test"})
        assert cfg.cache_path == Path("/tmp/qmd-test")

    def test_empty_dict(self):
        cfg = Config.from_dict({})
        assert cfg.bm25.backend == "sqlite_fts5"
        assert cfg.vector.backend == "qmd_builtin"


class TestConfigToDict:
    def test_roundtrip(self):
        cfg = Config()
        data = cfg.to_dict()
        assert data["bm25"]["backend"] == "sqlite_fts5"
        assert data["vector"]["backend"] == "qmd_builtin"
        assert data["vector"]["model"] == "embeddinggemma-300M"
        assert data["collections"] == []

    def test_with_collections(self):
        cfg = Config()
        cfg.collections = [
            CollectionConfig(name="test", path=Path("/tmp/test"), pattern="*.md"),
        ]
        data = cfg.to_dict()
        assert len(data["collections"]) == 1
        assert data["collections"][0]["name"] == "test"
        assert data["collections"][0]["pattern"] == "*.md"


class TestDBPath:
    def test_basic(self):
        cfg = Config()
        cfg.cache_path = Path("/tmp/qmd-test")
        path = cfg.db_path_for("notes")
        assert path == Path("/tmp/qmd-test/notes/index.db")

    def test_different_collections(self):
        cfg = Config()
        cfg.cache_path = Path("/data")
        assert cfg.db_path_for("a") == Path("/data/a/index.db")
        assert cfg.db_path_for("b") == Path("/data/b/index.db")


class TestCollectionConfig:
    def test_basic(self):
        col = CollectionConfig(name="notes", path=Path("/notes"))
        assert col.name == "notes"
        assert col.path == Path("/notes")
        assert col.pattern is None
        assert col.description is None

    def test_with_optional_fields(self):
        col = CollectionConfig(
            name="docs", path=Path("/docs"),
            pattern="**/*.md", description="Documentation"
        )
        assert col.pattern == "**/*.md"
        assert col.description == "Documentation"


class TestBM25Config:
    def test_default(self):
        cfg = BM25Config()
        assert cfg.backend == "sqlite_fts5"

    def test_lancedb(self):
        cfg = BM25Config(backend="lancedb")
        assert cfg.backend == "lancedb"


class TestVectorConfig:
    def test_default(self):
        cfg = VectorConfig()
        assert cfg.backend == "qmd_builtin"
        assert cfg.model == "embeddinggemma-300M"

    def test_qdrant(self):
        cfg = VectorConfig(
            backend="qdrant",
            qdrant=QdrantConfig(url="http://q:6333", vector_size=768),
        )
        assert cfg.backend == "qdrant"
        assert cfg.qdrant.vector_size == 768


class TestQdrantConfig:
    def test_defaults(self):
        cfg = QdrantConfig()
        assert cfg.url == "http://localhost:6333"
        assert cfg.api_key is None
        assert cfg.collection == "qmd_documents"
        assert cfg.vector_size == 384


class TestLLMModelConfig:
    def test_defaults(self):
        cfg = LLMModelConfig()
        assert cfg.local is None
        assert cfg.remote is None

    def test_local(self):
        cfg = LLMModelConfig(local="embeddinggemma-300M")
        assert cfg.local == "embeddinggemma-300M"

    def test_remote(self):
        cfg = LLMModelConfig(remote="text-embedding-3-small")
        assert cfg.remote == "text-embedding-3-small"


class TestConstants:
    def test_default_config_path(self):
        assert DEFAULT_CONFIG_PATH == "~/.config/qmd/index.yaml"

    def test_default_cache_path(self):
        assert DEFAULT_CACHE_PATH == "~/.cache/qmd"
