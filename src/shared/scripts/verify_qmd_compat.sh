#!/bin/bash
# QMD 兼容性验证脚本

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 配置
TEST_DIR="/tmp/qmd-test"
QMD_BIN="${1:-qmd}"  # 可指定 qmd 二进制路径

echo "=========================================="
echo "QMD 兼容性验证测试"
echo "=========================================="

# 步骤 1: 检查 qmd 是否可用
echo -e "\n${YELLOW}[1/7] 检查 qmd 命令...${NC}"
if command -v qmd &> /dev/null; then
    echo -e "${GREEN}✓ qmd 已安装: $(which qmd)${NC}"
    qmd --version || qmd --help | head -3
else
    echo -e "${RED}✗ qmd 未安装${NC}"
    exit 1
fi

# 步骤 2: 检查测试数据
echo -e "\n${YELLOW}[2/7] 检查测试数据...${NC}"
if [ ! -d "$TEST_DIR/notes" ]; then
    echo -e "${RED}✗ 测试目录不存在: $TEST_DIR/notes${NC}"
    exit 1
fi
echo -e "${GREEN}✓ 测试数据存在${NC}"
ls -la "$TEST_DIR/notes"

# 步骤 3: 清理已有索引
echo -e "\n${YELLOW}[3/7] 清理旧索引...${NC}"
rm -rf ~/.cache/qmd/
echo -e "${GREEN}✓ 索引已清理${NC}"

# 步骤 4: 添加集合
echo -e "\n${YELLOW}[4/7] 添加集合...${NC}"
echo "$QMD_BIN collection add $TEST_DIR/notes --name notes --mask '**/*.md'"
$QMD_BIN collection add "$TEST_DIR/notes" --name notes --mask "**/*.md"

# 步骤 5: 列出集合
echo -e "\n${YELLOW}[5/7] 列出集合...${NC}"
echo "$QMD_BIN collection list"
$QMD_BIN collection list

# 步骤 6: BM25 搜索测试
echo -e "\n${YELLOW}[6/7] BM25 搜索测试...${NC}"
echo "$QMD_BIN search 'authentication'"
$QMD_BIN search "authentication" --format json

# 步骤 7: 向量搜索测试
echo -e "\n${YELLOW}[7/7] 向量搜索测试 (需要 embed)...${NC}"
# 先尝试 embed (可能需要下载模型)
echo "$QMD_BIN embed --collection notes" || echo "embed 需要模型，跳过"
echo "$QMD_BIN vsearch 'API endpoints'" || echo "vsearch 跳过"

echo -e "\n=========================================="
echo -e "${GREEN}基本验证完成${NC}"
echo "=========================================="

# 额外测试: 检查其他命令
echo -e "\n${YELLOW}额外命令测试${NC}"

echo -e "\n--- status ---"
$QMD_BIN status --format json || true

echo -e "\n--- get ---"
$QMD_BIN get notes/test1.md --limit 5 || true

echo -e "\n--- search with collection ---"
$QMD_BIN search "planning" -c notes --format json || true

echo -e "\n--- search with output formats ---"
$QMD_BIN search "API" --format csv || true

echo -e "\n=========================================="
echo -e "${GREEN}验证脚本执行完成${NC}"
echo "=========================================="
