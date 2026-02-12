"""Main entry point for QMD CLI"""

import sys

from qmd.cli.commands import cli


def main():
    sys.exit(cli())


if __name__ == "__main__":
    main()
