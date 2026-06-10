# 三位一体治理体系

> 2026-06-10 建立。

---

## 三权分立

| 角色 | 职责 | 边界 |
|---|---|---|
| **制作人（你）** | 策划讨论、最终拍板 | 不碰执行、不碰命令行 |
| **DeepSeek（我）** | 主动讨论、写 handoff、接洽 Cursor、维护工作体系 | 不直接改 `.rs`、不直接编译 |
| **Cursor** | 执行 handoff、被动解释代码 | 不主动设计、不改 AIMemory/ |

---

## 三位一体

### 1. 工作流程

```
策划讨论（你+我）
    │
    ▼
我写 handoff → 更新 AIMemory/current.md → push
    │
    ▼
你告诉 Cursor → "读 AIMemory/current.md，执行"
    │
    ▼
Cursor 本地执行 → 编译 → 你启动游戏验证
    │
    ▼
确认 OK → Cursor commit + push → GitHub CI 自动测试
```

### 2. 合作体系

- **我每次接收任务第一步**：读 `AIMemory/current.md` + `AIMemory/shortcuts.md`
- **每个 handoff 必含**：架构计划 + 架构反馈
- **设计结论**: 写入 `AIMemory/design_*.md`
- **一次事件**: 写入 `memory/JOURNAL.jsonl`
- **持久知识**: 写入 `memory/FACT.md`
- **我不直接改 `.rs`**，所有代码改动通过 handoff
- **我负责编译**（`cargo run`），用户只点快捷方式进游戏
- **GitHub Actions** 负责 CI（test + smoke），不是用户电脑

### 3. 工作资料

| 文件 | 用途 | 维护者 |
|---|---|---|
| `AIMemory/current.md` | 当前执行单 | DeepSeek |
| `AIMemory/shortcuts.md` | 工作流参考 | DeepSeek |
| `AIMemory/governance.md` | 本文件 | DeepSeek |
| `AIMemory/design_*.md` | 设计文档 | DeepSeek |
| `AIMemory/handoff_*.md` | 执行单 | DeepSeek |
| `.cursorrules` | Cursor 代码规范 | DeepSeek |
| `.github/workflows/ci.yml` | CI 配置 | DeepSeek |
| `memory/FACT.md` | 持久知识 | DeepSeek |
| `memory/JOURNAL.jsonl` | 事件日志 | DeepSeek |
