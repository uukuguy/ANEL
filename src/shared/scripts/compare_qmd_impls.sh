#!/bin/bash
# QMD 多语言实现对比验证脚本

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# 配置
TEST_DIR="/tmp/qmd-test"

# 各版本路径
RUST_BIN="${HOME}/sandbox/LLM/speechless.ai/Autonomous-Agents/ANEL/src/qmd-rust/target/release/qmd-rust"
GO_BIN="${HOME}/sandbox/LLM/speechless.ai/Autonomous-Agents/ANEL/src/qmd-go/qmd"
PYTHON_BIN="python3 -m qmd_python"
TS_BIN="qmd-ts"

# 打印分隔线
print_separator() {
    echo "=========================================="
}

print_header() {
    print_separator
    echo -e "${BLUE}$1${NC}"
    print_separator
}

# 检查二进制是否存在
check_binary() {
    local name=$1
    local path=$2
    if [ -x "$path" ]; then
        echo -e "${GREEN}✓${NC} $name: $path"
        return 0
    elif command -v $path &> /dev/null; then
        echo -e "${GREEN}✓${NC} $name: $(which $path)"
        return 0
    else
        echo -e "${RED}✗${NC} $name: 不可用 ($path)"
        return 1
    fi
}

print_header "QMD 多语言实现对比验证"

# 步骤 1: 检查各版本
echo -e "\n${YELLOW}[1/6] 检查各版本可执行文件...${NC}"

check_binary "Rust" "$RUST_BIN" || true
check_binary "Go" "$GO_BIN" || true
check_binary "Python" "$PYTHON_BIN" || true
check_binary "TypeScript" "$TS_BIN" || true

# 步骤 2: 准备测试数据
echo -e "\n${YELLOW}[2/6] 准备测试数据...${NC}"

rm -rf "$TEST_DIR"
mkdir -p "$TEST_DIR/notes"

cat > "$TEST_DIR/notes/test1.md" << 'EOF'
# Project Timeline

This project started in January 2024.
The main features include BM25 search and vector embeddings.
We use SQLite FTS5 for full-text search.
EOF

cat > "$TEST_DIR/notes/test2.md" << 'EOF'
# API Reference

The authentication API uses JWT tokens.
Endpoints:
- POST /login
- GET /user/profile
- PUT /user/settings
EOF

cat > "$TEST_DIR/notes/test3.md" << 'EOF'
# Meeting Notes

Q4 planning discussion from November 2024.
Topics:
- Budget allocation
- Team expansion
- Product roadmap
EOF

echo "测试文件已创建:"
ls -la "$TEST_DIR/notes"

# 步骤 3: 对比 --help 输出
print_header "对比 --help 输出"

echo -e "\n${YELLOW}--- Rust 版本 --help (前 30 行) ---${NC}"
if [ -x "$RUST_BIN" ]; then
    $RUST_BIN --help | head -30
fi

echo -e "\n${YELLOW}--- Go 版本 --help (前 30 行) ---${NC}"
if [ -x "$GO_BIN" ]; then
    $GO_BIN --help | head -30
fi

# 步骤 4: 添加集合测试
print_header "添加集合测试"

for name in "Rust" "Go"; do
    case $name in
        Rust) BIN=$RUST_BIN ;;
        Go) BIN=$GO_BIN ;;
    esac

    if [ ! -x "$BIN" ]; then
        continue
    fi

    echo -e "\n${BLUE}--- $name 版本 ---${NC}"

    # 清理索引
    rm -rf ~/.cache/qmd/

    # 添加集合
    echo "$BIN collection add $TEST_DIR/notes --name notes"
    $BIN collection add "$TEST_DIR/notes" --name notes --mask "**/*.md" 2>&1 || echo "添加失败"

    # 列出集合
    echo -e "\n$BIN collection list"
    $BIN collection list 2>&1 || echo "列出失败"
done

# 步骤 5: 搜索测试
print_header "搜索功能测试"

for name in "Rust" "Go"; do
    case $name in
        Rust) BIN=$RUST_BIN ;;
        Go) BIN=$GO_BIN ;;
    esac

    if [ ! -x "$BIN" ]; then
        continue
    fi

    echo -e "\n${BLUE}--- $name 版本搜索测试 ---${NC}"

    # BM25 搜索
    echo -e "\n$BIN search 'authentication' (JSON):"
    $BIN search "authentication" --format json 2>&1 || echo "搜索失败"

    # 带集合参数
    echo -e "\n$BIN search 'planning' -c notes (JSON):"
    $BIN search "planning" -c notes --format json 2>&1 || echo "搜索失败"

    # 带数量限制
    echo -e "\n$BIN search 'API' -n 2 (JSON):"
    $BIN search "API" -n 2 --format json 2>&1 || echo "搜索失败"
done

# 步骤 6: get 命令测试
print_header "get 命令测试"

for name in "Rust" "Go"; do
    case $name in
        Rust) BIN=$RUST_BIN ;;
        Go) BIN=$GO_BIN ;;
    esac

    if [ ! -x "$BIN" ]; then
        continue
    fi

    echo -e "\n${BLUE}--- $name 版本 ---${NC}"
    echo "$BIN get notes/test1.md --limit 10:"
    $BIN get notes/test1.md --limit 10 2>&1 || echo "获取失败"
done

# 步骤 7: status 命令测试
print_header "status 命令测试"

for name in "Rust" "Go"; do
    case $name in
        Rust) BIN=$RUST_BIN ;;
        Go) BIN=$GO_BIN ;;
    esac

    if [ ! -x "$BIN" ]; then
        continue
    fi

    echo -e "\n${BLUE}--- $name 版本 ---${NC}"
    echo "$BIN status:"
    $BIN status 2>&1 || echo "状态获取失败"
done

print_header "验证完成"

echo -e "\n${GREEN}对比验证已完成${NC}"
echo "请检查上述输出，确认各版本的:"
echo "  1. --help 输出格式是否一致"
echo "  2. collection add/list 是否正常工作"
echo "  3. search 输出格式是否一致"
echo "  4. get 输出格式是否一致"
echo "  5. status 输出格式是否一致"
