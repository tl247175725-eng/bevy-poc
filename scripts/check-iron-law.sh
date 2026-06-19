#!/bin/bash
# ============================================================
# 铁律强制检查脚本 — 每次 commit 前自动运行
# 用法: bash scripts/check-iron-law.sh [--strict]
#   --strict: 新增文件也检查（用于 CI）
# ============================================================
set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

FAILURES=0
WARNINGS=0
SRC_DIR="src"
STRICT=false

if [[ "${1:-}" == "--strict" ]]; then
    STRICT=true
fi

echo "=================================="
echo "  铁律强制检查"
echo "=================================="

# ── 辅助函数 ──
check_pass() { echo -e "  ${GREEN}✅${NC} $1"; }
check_fail() { echo -e "  ${RED}❌${NC} $1"; FAILURES=$((FAILURES + 1)); }
check_warn() { echo -e "  ${YELLOW}⚠️${NC}  $1"; WARNINGS=$((WARNINGS + 1)); }

# ── 规则 1: 禁止裸 EntityId(0) ──
echo ""
echo "【规则1】禁止裸 EntityId(0)"

ENTITYID_ZERO=$(grep -rn "EntityId(0)" "$SRC_DIR/" --include="*.rs" | grep -v "NONE_ID\|//.*EntityId(0)\|cfg(test)" || true)
if [ -z "$ENTITYID_ZERO" ]; then
    check_pass "无裸 EntityId(0)"
else
    check_fail "发现裸 EntityId(0):"
    echo "$ENTITYID_ZERO" | while read line; do echo "       $line"; done
fi

# ── 规则 1b: 禁止将 NONE_ID 作为动作目标 ──
echo ""
echo "【规则1b】禁止 Strike/Consume/Combine 以 NONE_ID 为目标"

NONEID_TARGET=$(grep -rn "Strike\|Consume\|Combine" "$SRC_DIR/" --include="*.rs" -A2 | grep "target:\s*NONE_ID" | grep -v "cfg(test)" || true)
if [ -z "$NONEID_TARGET" ]; then
    check_pass "动作无 NONE_ID 目标"
else
    check_fail "发现动作以 NONE_ID 为目标（语义绕过——NONE_ID 就是 EntityId(0)）:"
    echo "$NONEID_TARGET" | while read line; do echo "       $line"; done
fi

# ── 规则 2: 禁止 type_name 字符串硬编码 ──
echo ""
echo "【规则2】禁止 type_name 字符串硬编码"

# 排除: 测试代码、同物种判断、clone、recipes(旧配方系统-已知例外)、已知测试文件
TYPE_NAME_RAW=$(grep -rn 'type_name\s*==\s*"' "$SRC_DIR/" --include="*.rs" \
    | grep -v "cfg(test)\|#\[test\]\|male_def.type_name == female\|\.type_name.clone()" \
    | grep -v "card_def.rs:1[0-9][0-9]:\|interaction/recipes.rs\|iron-law:allow" \
    || true)
if [ -z "$TYPE_NAME_RAW" ]; then
    check_pass "无 type_name 字符串硬编码"
else
    check_fail "发现 type_name 字符串硬编码:"
    echo "$TYPE_NAME_RAW" | while read line; do echo "       $line"; done
fi

# ── 规则 3: 禁止按标签 if-else 链（公理函数除外） ──
echo ""
echo "【规则3】禁止按标签 if-else 链"

# 检测模式: card_has_tag(...) { ... } 后紧跟 else if card_has_tag
TAG_IFELSE=$(grep -rn "if card_has_tag.*{.*}" "$SRC_DIR/" --include="*.rs" -A3 | grep "else if card_has_tag\|if card_has_tag" | grep -v "can_digest\|consume\|cfg(test)" || true)
if [ -z "$TAG_IFELSE" ]; then
    check_pass "无按标签 if-else 链"
else
    check_warn "发现疑似按标签分支（请人工审查）:"
    echo "$TAG_IFELSE" | while read line; do echo "       $line"; done
fi

# ── 规则 4: TICK_SECONDS 和 TICKS_PER_DAY 只在 meta_values.rs 定义 ──
echo ""
echo "【规则4】meta_values.rs 是 TICK_SECONDS/TICKS_PER_DAY 唯一定义源"

# 检查是否有其他文件定义这两个常量
TICK_DUP=$(grep -rn "pub const TICK_SECONDS\|pub const TICKS_PER_DAY" "$SRC_DIR/" --include="*.rs" | grep -v "meta_values.rs" || true)
if [ -z "$TICK_DUP" ]; then
    check_pass "TICK_SECONDS/TICKS_PER_DAY 仅在 meta_values.rs 定义"
else
    check_fail "TICK_SECONDS/TICKS_PER_DAY 在其他文件重复定义:"
    echo "$TICK_DUP" | while read line; do echo "       $line"; done
fi

# ── 规则 5: game_constants.rs 不得与 meta_values.rs 冲突 ──
echo ""
echo "【规则5】game_constants.rs 与 meta_values.rs 无冲突"

GC_TICK=$(grep -c "TICK_SECONDS\|TICKS_PER_DAY" "$SRC_DIR/game_constants.rs" 2>/dev/null || true)
GC_TICK="${GC_TICK:-0}"
if [ "${GC_TICK// /}" = "0" ]; then
    check_pass "game_constants.rs 无冲突常量"
else
    check_fail "game_constants.rs 仍包含 TICK_SECONDS/TICKS_PER_DAY"
fi

# ── 规则 6: 裸数字检测（宽松模式——只检查高频模式） ──
echo ""
echo "【规则6】高频裸数字检测"

# 检查 hp -= 1（应为 meta_values 派生）
HP_MINUS_ONE=$(grep -rn "hp.*-= 1\|hp.*saturating_sub(1)" "$SRC_DIR/" --include="*.rs" | grep -v "cfg(test)\|meta_values" || true)
if [ -z "$HP_MINUS_ONE" ]; then
    check_pass "无 hp -= 1 裸数字"
else
    check_warn "发现 hp -= 1 裸数字（应改为元数值公式）:"
    echo "$HP_MINUS_ONE" | while read line; do echo "       $line"; done
fi

# 检查 decay_rate 裸数字（不在 meta_values 引用也不在注释中）
DECAY_BARE=$(grep -rn "decay_rate.*[0-9]\.[0-9]" "$SRC_DIR/" --include="*.rs" | grep -v "meta_values\|cfg(test)\|//\|default_decay_rate\|NUTRITION_DECAY\|SOCIAL_DECAY\|CURIOSITY_DECAY\|MEMORY_DECAY" || true)
if [ -z "$DECAY_BARE" ]; then
    check_pass "decay_rate 值均引用 meta_values 常量"
else
    check_warn "发现 decay_rate 裸数字:"
    echo "$DECAY_BARE" | while read line; do echo "       $line"; done
fi

# ── 规则 7: 核心标签注册检查 ──
echo ""
echo "【规则7】核心标签是否注册"

MISSING_CORE=""
for tag in "animal" "plant" "terrain" "tree" "fish" "state:dead" "diet:carnivore" "diet:herbivore" "body_plan:quadruped" "habitat:aquatic"; do
    if ! grep -q "\"$tag\"" src/tags.rs 2>/dev/null; then
        MISSING_CORE="$MISSING_CORE $tag"
    fi
done

if [ -z "$MISSING_CORE" ]; then
    check_pass "核心标签已注册"
else
    check_fail "核心标签未在 tags.rs 注册:$MISSING_CORE"
fi

# ── 规则 8: FACT.md 三柱断点表是否过期 ──
echo ""
echo "【规则8】FACT.md 三柱断点表是否过期"

# 检查是否有全 ❌ 行（表示有已知断点未修）
BROKEN=$(grep -c "❌.*❌.*❌.*❌" memory/FACT.md 2>/dev/null || true)
BROKEN="${BROKEN:-0}"
if [ "${BROKEN// /}" = "0" ]; then
    check_pass "FACT.md 三柱断点表无全❌行"
else
    check_warn "FACT.md 三柱表有 ${BROKEN// /} 行全❌——是否遗忘更新？"
fi

# ── 规则 8b: 热路径禁 stub/TODO/死函数 ──
echo ""
echo "【规则8b】热路径禁 stub/TODO/死函数"

HOT_STUBS=$(grep -rn "Vec::new();\s*//\s*TODO\|todo!()" "$SRC_DIR/systems/main_tick.rs" "$SRC_DIR/need_match/" "$SRC_DIR/execution.rs" --include="*.rs" 2>/dev/null || true)
if [ -z "$HOT_STUBS" ]; then
    check_pass "热路径无 stub/TODO"
else
    check_fail "热路径发现 stub/TODO:"
    echo "$HOT_STUBS" | while read line; do echo "       $line"; done
fi

# ── 规则 9: handoff 模板合规 ──
echo ""
echo "【规则9】handoff 文件合规"

# 检查最近新增的 handoff 是否有三柱声明段
RECENT_HANDOFFS=$(find AIMemory/handoffs/ -name "*.md" ! -name "_TEMPLATE.md" -newer AIMemory/handoffs/_TEMPLATE.md 2>/dev/null)
if [ -n "$RECENT_HANDOFFS" ]; then
    while IFS= read -r hf; do
        if ! grep -q "三柱强制检查\|三柱对照\|三柱归属" "$hf" 2>/dev/null; then
            check_warn "$hf 缺少三柱声明段"
        fi
    done <<< "$RECENT_HANDOFFS"
    check_pass "handoff 三柱声明检查完成"
else
    check_pass "无新增 handoff 文件"
fi

# ── 规则 10: 本体一致性 ──
echo ""
echo "【规则10】本体(library/)一致性"

# 检查设计图书馆目录是否存在
if [ ! -f "AIMemory/library/_INDEX.md" ]; then
    check_fail "AIMemory/library/_INDEX.md 缺失"
else
    check_pass "library/_INDEX.md 存在"
fi

# 检查 tags.ron 中的维度是否列在 04-abstractions.md 中
if [ -f "AIMemory/library/04-abstractions.md" ] && [ -f "assets/tags.ron" ]; then
    ONTO_DIMS=$(grep -oE '[a-z_]+:' AIMemory/library/04-abstractions.md | grep -vE "ontology|A层|B层|depth|labels|note|desc|derives|affects|status|file|inputs|output|validates|used_by|invariant|value|params|id:|time:|space:|materials:|thermal:|senses:|life:|mind:|social:|ecology:|派生|cross_references|tag_to|abstraction|depth_levels|change_process|invariants" | tr -d ' :' | sort -u)
    TAGS_DIMS=$(grep -oE '^    [a-z_]+:' assets/tags.ron | tr -d ' :' | sort -u)
    MISSING_IN_ONTO=""
    for dim in $TAGS_DIMS; do
        if ! echo "$ONTO_DIMS" | grep -q "$dim"; then
            MISSING_IN_ONTO="$MISSING_IN_ONTO $dim"
        fi
    done
    if [ -z "$MISSING_IN_ONTO" ]; then
        check_pass "tags.ron 所有维度在本体中有记录"
    else
        check_warn "tags.ron 中有维度未在本体记录:$MISSING_IN_ONTO"
    fi
fi

# ── 规则 11: 禁止 unwrap() / expect() ──
echo ""
echo "【规则11】禁止 unwrap() / expect()"

UNWRAP_COUNT=$(grep -rn "\.unwrap()" "$SRC_DIR/" --include="*.rs" | grep -v "cfg(test)" | wc -l)
EXPECT_COUNT=$(grep -rn "\.expect(" "$SRC_DIR/" --include="*.rs" | grep -v "cfg(test)" | wc -l)

if [ "$UNWRAP_COUNT" -eq 0 ] && [ "$EXPECT_COUNT" -eq 0 ]; then
    check_pass "无 unwrap()/expect()"
else
    if [ "$UNWRAP_COUNT" -gt 0 ]; then
        check_fail "发现 $UNWRAP_COUNT 处 .unwrap()（非测试代码禁止）"
    fi
    if [ "$EXPECT_COUNT" -gt 0 ]; then
        check_fail "发现 $EXPECT_COUNT 处 .expect()（非测试代码禁止）"
    fi
fi

# ── 汇总 ──
echo ""
echo "=================================="
if [ "$FAILURES" -eq 0 ] && [ "$WARNINGS" -eq 0 ]; then
    echo -e "  ${GREEN}全部通过 ✅${NC}"
elif [ "$FAILURES" -eq 0 ]; then
    echo -e "  ${YELLOW}$WARNINGS 个警告（不阻塞）${NC}"
    echo -e "  ${GREEN}0 个致命错误${NC}"
else
    echo -e "  ${RED}$FAILURES 个致命错误${NC}"
    if [ "$WARNINGS" -gt 0 ]; then
        echo -e "  ${YELLOW}$WARNINGS 个警告${NC}"
    fi
    echo ""
    echo "  请修复致命错误后再提交。"
fi
echo "=================================="

if [ "$FAILURES" -gt 0 ]; then
    exit 1
fi
exit 0