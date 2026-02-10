"""Configuration management for QMD."""

from dataclasses import dataclass, field
from pathlib import Path
from typing import Optional, List
import os
import yaml


DEFAULT_CONFIG_PATH = "~/.config/qmd/index.yaml"
DEFAULT_CACHE_PATH = "~/.cache/qmd"


@dataclass
class CollectionConfig:
    """Collection configuration."""

    name: str
    path: Path
    pattern: Optional[str] = None
    description: Optional[str] = None


@dataclass
class BM25Config:
    """BM25 backend configuration."""

    backend: str = "sqlite_fts5"


@dataclass
class VectorConfig:
    """Vector backend configuration."""

    backend: str = "qmd_builtin"
    model: str = "embeddinggemma-300M"


@dataclass
class LLMModelConfig:
    """LLM model configuration."""

    local: Optional[str] = None
    remote: Optional[str] = None


@dataclass
class ModelsConfig:
    """Models configuration."""

    embed: Optional[LLMModelConfig] = None
    rerank: Optional[LLMModelConfig] = None
    query_expansion: Optional[LLMModelConfig] = None


@dataclass
class Config:
    """Main configuration."""

    bm25: BM25Config = field(default_factory=BM25Config)
    vector: VectorConfig = field(default_factory=VectorConfig)
    collections: List[CollectionConfig] = field(default_factory=list)
    models: ModelsConfig = field(default_factory=ModelsConfig)
    cache_path: Path = Path("~/.cache/qmd")

    @classmethod
    def load(cls) -> "Config":
        """Load configuration from default path."""
        config_path = Path(os.path.expanduser(DEFAULT_CONFIG_PATH))

        if config_path.exists():
            with open(config_path) as f:
                data = yaml.safe_load(f) or {}
            return cls.from_dict(data)

        return cls()

    @classmethod
    def from_dict(cls, data: dict) -> "Config":
        """Create config from dictionary."""
        config = cls()

        if "bm25" in data:
            config.bm25 = BM25Config(**data["bm25"])

        if "vector" in data:
            config.vector = VectorConfig(**data["vector"])

        if "collections" in data:
            config.collections = []
            for col_data in data["collections"]:
                path = Path(os.path.expanduser(col_data["path"]))
                config.collections.append(CollectionConfig(
                    name=col_data["name"],
                    path=path,
                    pattern=col_data.get("pattern"),
                    description=col_data.get("description"),
                ))

        if "models" in data:
            config.models = ModelsConfig(**data["models"])

        if "cache_path" in data:
            config.cache_path = Path(os.path.expanduser(data["cache_path"]))

        return config

    def save(self) -> None:
        """Save configuration to default path."""
        config_path = Path(os.path.expanduser(DEFAULT_CONFIG_PATH))
        config_path.parent.mkdir(parents=True, exist_ok=True)

        data = self.to_dict()
        with open(config_path, "w") as f:
            yaml.dump(data, f)

    def to_dict(self) -> dict:
        """Convert config to dictionary."""
        return {
            "bm25": {"backend": self.bm25.backend},
            "vector": {"backend": self.vector.backend, "model": self.vector.model},
            "collections": [
                {
                    "name": c.name,
                    "path": str(c.path),
                    "pattern": c.pattern,
                    "description": c.description,
                }
                for c in self.collections
            ],
            "cache_path": str(self.cache_path),
        }

    def db_path_for(self, collection: str) -> Path:
        """Get database path for a collection."""
        return self.cache_path / collection / "index.db"
