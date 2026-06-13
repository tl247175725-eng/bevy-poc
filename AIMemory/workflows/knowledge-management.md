# 知识管理工作流

## 两个存储

| 文件 | 存什么 | 工具 |
|---|---|---|
| `memory/FACT.md` | 持久知识（6 个月后仍然有效） | `mcp__agent-memory__memory` action=update |
| `memory/JOURNAL.jsonl` | 一次性事件、任务完成记录 | `mcp__agent-memory__memory` action=append |

## 写入规则

- **FACT.md**：项目凭证、工作流规则、设计铁律
- **JOURNAL.jsonl**：handoff 完成、设计讨论、CI 修复、工具链变更
- 存档前问自己：这个 6 个月后还有用吗？是 → FACT.md，否 → JOURNAL.jsonl
- 永不用文件编辑工具直接写 `memory/` 目录——必须走 `mcp__agent-memory__memory` 工具
