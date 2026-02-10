"""QMD - AI-powered search with hybrid BM25 and vector search."""

import sys
from pathlib import Path
from .cli.app import app

# Add src to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent))


def main() -> None:
    """Entry point for QMD CLI."""
    app()


if __name__ == "__main__":
    main()
