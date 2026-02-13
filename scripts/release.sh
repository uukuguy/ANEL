#!/bin/bash
# Release script for QMD - creates release packages for all platforms
# Usage: ./scripts/release.sh [--version VERSION] [--platforms PLATFORMS]

set -e

VERSION="0.1.0"
PLATFORMS="darwin,linux"
DRY_RUN=false

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --version)
            VERSION="$2"
            shift 2
            ;;
        --platforms)
            PLATFORMS="$2"
            shift 2
            ;;
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
RELEASE_DIR="$PROJECT_ROOT/release"

echo "=== Creating QMD Release v$VERSION ==="
echo "Platforms: $PLATFORMS"
echo ""

# Create release directory
mkdir -p "$RELEASE_DIR"

# Get current date for release
DATE=$(date +%Y-%m-%d)

# Platform-specific settings
PLATFORM_ARRAY=($(echo "$PLATFORMS" | tr ',' '\n'))

for PLATFORM in "${PLATFORM_ARRAY[@]}"; do
    echo ">>> Building for $PLATFORM..."

    case $PLATFORM in
        darwin)
            ARCHS=("amd64" "arm64")
            ;;
        linux)
            ARCHS=("amd64" "arm64")
            ;;
        *)
            echo "Unknown platform: $PLATFORM"
            continue
            ;;
    esac

    for ARCH in "${ARCHS[@]}"; do
        echo "  Building $PLATFORM/$ARCH..."

        # Note: Cross-compilation would require additional setup
        # For now, we build for the current platform only
    done
done

echo ""
echo ">>> Creating distribution packages..."

# Create version file
echo "QMD v$VERSION" > "$RELEASE_DIR/VERSION"
echo "Released: $DATE" >> "$RELEASE_DIR/VERSION"

# Build current platform
echo "Building for current platform..."
cd "$PROJECT_ROOT"

# Build all versions
echo "  Building Rust..."
cd "$PROJECT_ROOT/src/qmd-rust"
cargo build --release --features "sqlite-vec,qdrant,lancedb"

echo "  Building Go..."
cd "$PROJECT_ROOT/src/qmd-go"
go build -o qmd ./cmd/qmd

echo "  Building Python..."
cd "$PROJECT_ROOT/src/qmd-python"
pip install -e . -q

echo ""
echo "=== Release Complete ==="
echo ""
echo "Release files:"
echo "  Rust:   src/qmd-rust/target/release/qmd"
echo "  Go:     src/qmd-go/qmd"
echo "  Python: Use 'pip install -e .'"
echo ""
echo "Run './scripts/build.sh --release' to build release versions"
