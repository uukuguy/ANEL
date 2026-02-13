#!/bin/bash
# Build script for QMD - builds all language implementations
# Usage: ./scripts/build.sh [--release] [--target TARGET]

set -e

RELEASE=false
TARGET=""
FEATURES=""

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --release)
            RELEASE=true
            shift
            ;;
        --target)
            TARGET="$2"
            shift 2
            ;;
        --features)
            FEATURES="$2"
            shift 2
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Default features for Rust
RUST_FEATURES="${FEATURES:-sqlite-vec,qdrant,lancedb}"
if [ -z "$FEATURES" ]; then
    echo "Building Rust with default features: $RUST_FEATURES"
fi

echo "=== Building QMD ==="
echo ""

# Build Rust
echo ">>> Building Rust..."
cd "$PROJECT_ROOT/src/qmd-rust"

BUILD_CMD="cargo build"
if [ "$RELEASE" = true ]; then
    BUILD_CMD="$BUILD_CMD --release"
fi
if [ -n "$RUST_FEATURES" ]; then
    BUILD_CMD="$BUILD_CMD --features $RUST_FEATURES"
fi

echo "  Running: $BUILD_CMD"
eval $BUILD_CMD
echo "  Rust build complete"
echo ""

# Build Go
echo ">>> Building Go..."
cd "$PROJECT_ROOT/src/qmd-go"

GO_BUILD_CMD="go build -o qmd ./cmd/qmd"
if [ "$RELEASE" = true ]; then
    GO_BUILD_CMD="$GO_BUILD_CMD -ldflags '-s -w'"
fi

echo "  Running: $GO_BUILD_CMD"
eval $GO_BUILD_CMD
echo "  Go build complete"
echo ""

# Build Python
echo ">>> Building Python..."
cd "$PROJECT_ROOT/src/qmd-python"

# Install dependencies
echo "  Installing Python dependencies..."
pip install -e . --quiet

echo "  Python build complete"
echo ""

echo "=== Build Complete ==="
echo ""
echo "Binaries:"
echo "  Rust:   $PROJECT_ROOT/src/qmd-rust/target/debug/qmd (or target/release/qmd)"
echo "  Go:     $PROJECT_ROOT/src/qmd-go/qmd"
echo "  Python: Use 'python -m qmd_python' or install and run 'qmd'"
