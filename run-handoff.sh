#!/bin/bash
# ============================================================
# Cline 全自动流水线 — 执行 Handoff（最强配置）
# 用法: bash run-handoff.sh
# ============================================================
set -e

PROJECT_DIR="E:/桌面/bevy-poc"
HANDOFF_FILE="$PROJECT_DIR/AIMemory/current.md"

echo "========================================="
echo "  Cline 全自动流水线 — Handoff 执行"
echo "  $(date)"
echo "========================================="

STATUS=$(grep "status:" "$HANDOFF_FILE" | head -1 | sed 's/.*status: //')
echo "Handoff 状态: $STATUS"

if [ "$STATUS" = "completed" ]; then
    echo "Handoff 已完成，跳过。"
    exit 0
fi

echo ""
echo "正在执行..."
echo ""

# ============================================================
# Cline 最强配置
# ============================================================
# --yolo          全自动，零交互，所有操作自动确认
# --thinking xhigh 最强推理（DeepSeek 1024 token thinking budget）
# --compaction agentic LLM 驱动的智能上下文压缩（非简单截断）
# --retries 6      最多 6 轮错误自修复
# --worktree       Git worktree 隔离——改动在独立分支，失败不影响主分支
# --cwd            工作目录
# -p openai        DeepSeek API（OpenAI 兼容）
# -m deepseek-chat DeepSeek 最新模型
# ============================================================

cline \
    -c "$PROJECT_DIR" \
    -p openai \
    -m "deepseek-chat" \
    --thinking xhigh \
    --compaction agentic \
    --retries 6 \
    --worktree \
    -y "你是方寸商国：桃花源记 项目的 AI 开发助手。

请按照以下流程执行当前 Handoff：

## 第一步：读取任务
读 AIMemory/current.md，提取 handoff 文件路径（file: 字段）。

## 第二步：读取 Handoff
读 handoff 文件。Handoff 包含三段：
- 架构计划：要改什么、为什么
- 架构反馈：注意事项
- 智能验收：改完后必须满足的条件（写成可执行断言）

## 第三步：读取相关代码和设计文档
读所有涉改文件。如果 handoff 引用了设计文档（AIMemory/design-*.md），也要读取。

## 第四步：实现改动
按架构计划逐个文件修改。规则：
- 只改 handoff 要求的，不动别的
- 所有改动遵守项目铁律（见 .cline/rules/bevy-poc.md）
- 修改最小化，不重构无关代码

## 第五步：验证
1. cargo test -- --nocapture
2. 如果失败 → 读报错 → 分析原因 → 修复 → 重试
3. cargo build
4. 全部通过才能继续

## 第六步：提交
git add <具体改动的文件>
git commit -m \"handoff: <改动摘要>\"
git push

## 第七步：报告
输出改动摘要清单：
- 改了哪些文件
- 每个文件的改动内容
- cargo test 结果: PASS/FAIL
- cargo build 结果: PASS/FAIL
- commit hash"

echo ""
echo "========================================="
echo "  执行完成 — 请检查结果"
echo "========================================="
