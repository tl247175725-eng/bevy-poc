#!/bin/bash
# ============================================================
# 桃花源项目初始化 — 新机器/新人一键配置
# 用法: bash scripts/setup.sh
# ============================================================
set -euo pipefail

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo "=================================="
echo "  桃花源项目初始化"
echo "=================================="

# ── 1. Git hooks ──
echo ""
echo "[1/3] 配置 Git hooks..."
git config core.hooksPath .githooks
echo -e "  ${GREEN}✅${NC} hooksPath = .githooks"

# ── 2. 脚本权限 ──
echo ""
echo "[2/3] 设置脚本权限..."
chmod +x .githooks/* scripts/*.sh 2>/dev/null || true
echo -e "  ${GREEN}✅${NC} 权限已设置"

# ── 3. 验证 ──
echo ""
echo "[3/3] 运行铁律检查..."
if bash scripts/check-iron-law.sh; then
    echo ""
    echo -e "${GREEN}=================================="
    echo "  初始化完成 ✅"
    echo "=================================="
    echo ""
    echo "  每次 commit 前自动运行铁律检查"
    echo "  每次 push 前自动运行完整验证"
    echo "  手动检查: bash scripts/check-iron-law.sh${NC}"
else
    echo ""
    echo -e "${YELLOW}⚠️  铁律检查有警告（不阻塞），初始化完成${NC}"
fi
