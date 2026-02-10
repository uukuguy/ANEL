"""Agent mode for QMD."""

from typing import Optional


def run_agent(
    interactive: bool = False,
    mcp: bool = False,
    transport: str = "stdio",
) -> None:
    """Run QMD in agent mode."""
    print("QMD Agent Mode")

    if interactive:
        import sys

        print("Type 'exit' to quit")
        while True:
            try:
                query = input("qmd> ").strip()
                if query.lower() == "exit":
                    break
                if not query:
                    continue

                # TODO: Implement intelligent query processing
                # - keyword queries -> BM25
                # - semantic queries -> vector search
                # - complex queries -> hybrid search
                print(f"Processing: {query}")

            except EOFError:
                break

    else:
        # Single query mode
        print("Agent mode ready")
