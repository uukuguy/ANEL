"""Output formatting for search results"""

import csv
import json
import io
from enum import Enum
from typing import Optional

from qmd.store.store import SearchResult


class Format(str, Enum):
    """Output format types"""
    CLI = "cli"
    JSON = "json"
    MARKDOWN = "markdown"
    CSV = "csv"
    FILES = "files"
    XML = "xml"


class Formatter:
    """Formats search results for output"""

    def __init__(self, format_str: str = "cli", limit: int = 20):
        self.format = Format(format_str) if format_str else Format.CLI
        self.limit = limit

    def format_results(self, results: list[SearchResult], query: str = "") -> str:
        """Format search results"""
        if len(results) > self.limit:
            results = results[:self.limit]

        match self.format:
            case Format.JSON:
                return self._format_json(results, query)
            case Format.MARKDOWN:
                return self._format_markdown(results)
            case Format.CSV:
                return self._format_csv(results)
            case Format.FILES:
                return self._format_files(results)
            case Format.XML:
                return self._format_xml(results)
            case _:
                return self._format_cli(results)

    def _format_cli(self, results: list[SearchResult]) -> str:
        """Format as CLI table"""
        if not results:
            return "No results found.\n"

        lines = []
        lines.append(f"{'Score':<10} {'Lines':<8} {'DocID':<6} {'Path'}")
        lines.append("-" * 60)

        for r in results:
            lines.append(f"{r.score:<10.4f} {r.lines:<8} {r.doc_id:<6} {r.path}")

        lines.append(f"\nTotal: {len(results)} results\n")
        return "\n".join(lines)

    def _format_json(self, results: list[SearchResult], query: str) -> str:
        """Format as JSON"""
        output = {
            "query": query,
            "total": len(results),
            "results": [
                {
                    "doc_id": r.doc_id,
                    "path": r.path,
                    "collection": r.collection,
                    "score": r.score,
                    "lines": r.lines,
                    "title": r.title,
                    "hash": r.hash,
                }
                for r in results
            ],
        }
        return json.dumps(output, indent=2) + "\n"

    def _format_markdown(self, results: list[SearchResult]) -> str:
        """Format as Markdown"""
        lines = ["# Search Results\n", f"Total: {len(results)} results\n"]

        for i, r in enumerate(results):
            lines.append(f"## {i + 1}. {r.path}\n")
            lines.append(f"- **Score**: {r.score:.4f}")
            lines.append(f"- **Lines**: {r.lines}")
            lines.append(f"- **Collection**: {r.collection}")
            lines.append(f"- **DocID**: {r.doc_id}\n")

        return "\n".join(lines)

    def _format_csv(self, results: list[SearchResult]) -> str:
        """Format as CSV"""
        output = io.StringIO()
        writer = csv.writer(output)
        writer.writerow(["score", "lines", "docid", "path", "collection", "title", "hash"])

        for r in results:
            writer.writerow([
                f"{r.score:.4f}",
                r.lines,
                r.doc_id,
                r.path,
                r.collection,
                r.title,
                r.hash,
            ])

        return output.getvalue()

    def _format_files(self, results: list[SearchResult]) -> str:
        """Format as file list"""
        return "\n".join(r.path for r in results) + "\n"

    def _format_xml(self, results: list[SearchResult]) -> str:
        """Format as XML"""
        lines = ['<?xml version="1.0" encoding="UTF-8"?>', "<results>"]
        lines.append(f"  <total>{len(results)}</total>")

        for r in results:
            lines.append("  <result>")
            lines.append(f"    <score>{r.score:.4f}</score>")
            lines.append(f"    <lines>{r.lines}</lines>")
            lines.append(f"    <docid>{self._escape_xml(r.doc_id)}</docid>")
            lines.append(f"    <path>{self._escape_xml(r.path)}</path>")
            lines.append(f"    <collection>{self._escape_xml(r.collection)}</collection>")
            lines.append(f"    <title>{self._escape_xml(r.title)}</title>")
            lines.append(f"    <hash>{r.hash}</hash>")
            lines.append("  </result>")

        lines.append("</results>\n")
        return "\n".join(lines)

    @staticmethod
    def _escape_xml(s: str) -> str:
        """Escape XML special characters"""
        return (
            s.replace("&", "&amp;")
            .replace("<", "&lt;")
            .replace(">", "&gt;")
            .replace('"', "&quot;")
            .replace("'", "&apos;")
        )
