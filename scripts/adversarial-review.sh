#!/bin/bash
# ============================================================
# 对抗性审查脚本 — 调用 DeepSeek API 审查最近改动
# 用法: bash scripts/adversarial-review.sh [commits=5]
# ============================================================
set -uo pipefail

COMMITS=${1:-5}
CONFIG=".deepseek/config.toml"
OUTPUT_DIR="AIMemory/reviews"
DATE=$(date +%Y-%m-%d)
OUTPUT_FILE="$OUTPUT_DIR/deepseek-review-$DATE.md"

echo "=================================="
echo "  对抗性审查（DeepSeek）"
echo "=================================="

# ── 1. 读取 API Key ──
if [ ! -f "$CONFIG" ]; then
    echo "❌ $CONFIG 不存在"
    exit 1
fi
API_KEY=$(grep "api_key" "$CONFIG" | sed 's/.*= *"//;s/".*//')
BASE_URL=$(grep "base_url" "$CONFIG" | sed 's/.*= *"//;s/".*//')
if [ -z "$API_KEY" ]; then
    echo "❌ API key 为空"
    exit 1
fi
echo "✅ API Key 已读取"

# ── 2. 收集最近改动 ──
echo "📋 收集最近 $COMMITS 个 commit 的 diff..."
DIFF=$(git log -$COMMITS --oneline 2>/dev/null)
DIFF_STAT=$(git diff HEAD~$COMMITS --stat 2>/dev/null | tail -5)
DIFF_CONTENT=$(git diff HEAD~$COMMITS -- src/ --no-color 2>/dev/null | head -300)

# ── 3. 读取设计原则 ──
PHILOSOPHY=""
if [ -f "AIMemory/library/02-philosophy.md" ]; then
    PHILOSOPHY=$(head -50 AIMemory/library/02-philosophy.md)
fi
RULES=""
if [ -f "AIMemory/library/10-rules.md" ]; then
    RULES=$(head -80 AIMemory/library/10-rules.md)
fi

# ── 4. 组装 prompt ──
PROMPT="你是一个对抗性代码审查员。你的任务是找出 AI 助手写的代码中的语义违规——不是语法错误，是设计意图层面的问题。

## 设计原则摘要
$PHILOSOPHY

## 实现规则摘要
$RULES

## 最近 $COMMITS 个 commit
$DIFF

## 改动统计
$DIFF_STAT

## 代码 diff（前300行）
$DIFF_CONTENT

## 你的任务

请找出以下问题：
1. **改名绕过**：有没有通过重命名常量/变量来满足检查脚本，但实际 bug 没修的情况？
2. **语义违规**：代码是否真的遵循了标签驱动原则？有没有隐藏的硬编码？
3. **设计哲学不一致**：有没有违反'标签即存在''同构'原则的地方？
4. **测试盲区**：有没有功能改动但没有对应的测试覆盖？
5. **技术债**：有没有临时方案变成了永久方案？

请用中文回答。每个问题给出：文件位置 + 具体违规 + 修复建议。如果没发现问题就说'未发现'。"

# ── 5. 调用 API ──
echo "🔍 调用 DeepSeek API..."
mkdir -p "$OUTPUT_DIR"

# JSON 转义
ESCAPED_PROMPT=$(echo "$PROMPT" | python3 -c "import sys,json; print(json.dumps(sys.stdin.read()))" 2>/dev/null || echo "$PROMPT" | sed 's/\\/\\\\/g;s/"/\\"/g;s/\t/\\t/g' | tr '\n' ' ')

RESPONSE=$(curl -s -X POST "${BASE_URL}/chat/completions" \
    -H "Authorization: Bearer $API_KEY" \
    -H "Content-Type: application/json" \
    -d "{
        \"model\": \"deepseek-chat\",
        \"messages\": [{\"role\": \"user\", \"content\": $ESCAPED_PROMPT}],
        \"temperature\": 0.3,
        \"max_tokens\": 4000
    }" 2>/dev/null)

# ── 6. 解析输出 ──
if echo "$RESPONSE" | grep -q '"content"'; then
    REVIEW=$(echo "$RESPONSE" | python3 -c "import sys,json; r=json.load(sys.stdin); print(r['choices'][0]['message']['content'])" 2>/dev/null)
    if [ -z "$REVIEW" ]; then
        REVIEW=$(echo "$RESPONSE" | grep -o '"content":"[^"]*"' | sed 's/"content":"//;s/"$//')
    fi

    # 写入报告
    cat > "$OUTPUT_FILE" << REPORT
# DeepSeek 对抗性审查报告

**日期**: $DATE
**审查范围**: 最近 $COMMITS 个 commit
**审查员**: DeepSeek API (deepseek-chat)

---

## 审查结果

$REVIEW

---

> 此报告由 scripts/adversarial-review.sh 自动生成。
REPORT

    echo ""
    echo "✅ 审查报告已生成: $OUTPUT_FILE"
    echo ""
    echo "--- 审查摘要 ---"
    echo "$REVIEW" | head -30
else
    echo "❌ API 调用失败"
    echo "$RESPONSE" | head -10
    exit 1
fi
