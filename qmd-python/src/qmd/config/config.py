"""Configuration management for QMD"""

from dataclasses import dataclass, field
from enum import Enum
from pathlib import Path
from typing import Optional

import yaml


class BM25Backend(str, Enum):
    SQLITE_FTS5 = "sqlite_fts5"
    LANCEDB = "lancedb"


class VectorBackend(str, Enum):
    QMD_BUILTIN = "qmd_builtin"
    LANCEDB = "lancedb"


@dataclass
class CollectionConfig:
    name: str
    path: str
    pattern: Optional[str] = None
    description: Optional[str] = None


@dataclass
class LLMModelConfig:
    local: Optional[str] = None
    remote: Optional[str] = None


@dataclass
class ModelsConfig:
    embed: Optional[LLMModelConfig] = None
    rerank: Optional[LLMModelConfig] = None
    query_expansion: Optional[LLMModelConfig] = None


@dataclass
class VectorBackendConfig:
    backend: VectorBackend = VectorBackend.QMD_BUILTIN
    model: str = "nomic-embed-text-v1.5"


@dataclass
class BM25BackendConfig:
    backend: BM25Backend = BM25Backend.SQLITE_FTS5


@dataclass
class Config:
    bm25: BM25BackendConfig = field(default_factory=BM25BackendConfig)
    vector: VectorBackendConfig = field(default_factory=VectorBackendConfig)
    collections: list[CollectionConfig] = field(default_factory=list)
    models: ModelsConfig = field(default_factory=ModelsConfig)
    cache_path: str = str(Path.home() / ".cache" / "qmd")

    @classmethod
    def default(cls) -> "Config":
        return cls()

    def save(self, config_path: str = "") -> None:
        if not config_path:
            config_path = str(Path.home() / ".config" / "qmd" / "index.yaml")

        Path(config_path).parent.mkdir(parents=True, exist_ok=True)

        with open(config_path, "w") as f:
            yaml.dump(self.to_dict(), f, default_flow_style=False)

    def to_dict(self) -> dict:
        return {
            "bm25": self.bm25.backend.value if isinstance(self.bm25.backend, BM25Backend) else self.bm25.backend,
            "vector": {
                "backend": self.vector.backend.value if isinstance(self.vector.backend, VectorBackend) else self.vector.backend,
                "model": self.vector.model,
            },
            "collections": [
                {
                    "name": c.name,
                    "path": c.path,
                    "pattern": c.pattern,
                    "description": c.description,
                }
                for c in self.collections
            ],
            "models": {
                "embed": self.models.embed.__dict__ if self.models.embed else None,
                "rerank": self.models.rerank.__dict__ if self.models.rerank else None,
                "query_expansion": self.models.query_expansion.__dict__ if self.models.query_expansion else None,
            },
            "cache_path": self.cache_path,
        }

    @classmethod
    def load(cls, config_path: str = "") -> "Config":
        if not config_path:
            config_path = str(Path.home() / ".config" / "qmd" / "index.yaml")

        if not Path(config_path).exists():
            return cls.default()

        with open(config_path) as f:
            data = yaml.safe_load(f)

        if not data:
            return cls.default()

        cfg = cls()
        cfg.cache_path = data.get("cache_path", cfg.cache_path)

        if "bm25" in data:
            cfg.bm25 = BM25BackendConfig(backend=BM25Backend(data["bm25"]))

        if "vector" in data:
            cfg.vector = VectorBackendConfig(
                backend=VectorBackend(data["vector"].get("backend", "qmd_builtin")),
                model=data["vector"].get("model", "nomic-embed-text-v1.5"),
            )

        if "collections" in data:
            cfg.collections = [
                CollectionConfig(
                    name=c["name"],
                    path=c["path"],
                    pattern=c.get("pattern"),
                    description=c.get("description"),
                )
                for c in data["collections"]
            ]

        return cfg

    def get_collection(self, name: str) -> Optional[CollectionConfig]:
        for col in self.collections:
            if col.name == name:
                return col
        return None

    def add_collection(self, collection: CollectionConfig) -> None:
        self.collections.append(collection)

    def remove_collection(self, name: str) -> None:
        self.collections = [c for c in self.collections if c.name != name]

    def cache_dir_for(self, collection: str) -> Path:
        return Path(self.cache_path) / collection

    def db_path_for(self, collection: str) -> Path:
        return self.cache_dir_for(collection) / "index.db"
