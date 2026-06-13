# Session 启动规则

```
每次接收任务后，第一步读取 AIMemory/current.md、AIMemory/shortcuts.md、memory/FACT.md。不跳过，不凭记忆。

工作流按性质分文件存放于 AIMemory/workflows/。执行对应性质的任务前，必须先读取对应工作流文件。

项目代码通过 handoff 执行。本地验证（cargo check + cargo test，限 4 核）通过后再 push。GitHub 作为代码备份和可选 CI。

设计讨论收束时向用户确认"铁律/暂定/放着"。意图对齐后自主执行，用户只负责跑游戏看效果。
```

---

## 工作流文件速查

| 任务性质 | 先读 |
|---|---|
| 写代码、改文件、执行 handoff、push、查 CI | `AIMemory/workflows/handoff-execution.md` |
| 设计讨论、标签设计、架构决策 | `AIMemory/workflows/design-discussion.md` |
| 记知识、存事实、写日志 | `AIMemory/workflows/knowledge-management.md` |
