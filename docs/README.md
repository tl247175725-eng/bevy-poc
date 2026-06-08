# 方寸商国：桃花源记 — 文档入口

> **唯一正本：** 本仓库 `docs/`。桌面仅保留 [`方寸商国_文档入口.md`](file:///E:/桌面/方寸商国_文档入口.md) 快捷入口。

---

## 我该读哪个？

| 你是谁 | 先读 | 再读 |
|--------|------|------|
| **策划 / 定玩法** | [`design/core/设计规则圣经_v0.3.txt`](design/core/设计规则圣经_v0.3.txt) 末尾最新节 | [`design/core/构建路线图_v3.x.md`](design/core/构建路线图_v3.x.md) · [`design/core/意向系统与世界演化_v0.3.md`](design/core/意向系统与世界演化_v0.3.md) |
| **改 AI / 需求 / 主链** | [`design/core/世界运转共识_v0.6.md`](design/core/世界运转共识_v0.6.md) | [`CODEX_HANDOFF.md`](CODEX_HANDOFF.md) §0 |
| **接工程 / Agent** | [`CODEX_HANDOFF.md`](CODEX_HANDOFF.md) | [`design/CHANGELOG.md`](design/CHANGELOG.md) · [`design/card-rule-audit.md`](design/card-rule-audit.md) |
| **加新卡** | [`design/card-rule-audit.md`](design/card-rule-audit.md) 顶部 checklist | `tools/run_card_factory_scaffold.ps1` → [`design/CHANGELOG.md`](design/CHANGELOG.md) |
| **做增量切片** | [`design/incremental-iteration-workflow-v0.1.md`](design/incremental-iteration-workflow-v0.1.md) | [`design/v3.0-a-river-valley-ecology-taoyuan-archetypes.md`](design/v3.0-a-river-valley-ecology-taoyuan-archetypes.md) |

---

## 目录结构

```
docs/
├── README.md                 ← 你在这里
├── CODEX_HANDOFF.md          工程交接、验收基线、§12 协作留痕
└── design/
    ├── README.md             设计文档索引 + 工作流
    ├── CHANGELOG.md          设计/工程变更日志
    │
    ├── core/                 ★ 策划正本（完整正文，不再分桌面/摘录）
    │   ├── 设计规则圣经_v0.3.txt
    │   ├── 构建路线图_v3.x.md
    │   ├── 世界运转共识_v0.6.md
    │   └── 意向系统与世界演化_v0.3.md
    │
    ├── engineering/          （与代码同级的工程向设计 — 见 design/README）
    │   └── …                 实际文件仍在 design/ 根目录，见下方索引
    │
    ├── generated/            自动导出（勿手改）
    │   ├── card-rule-facts.md
    │   └── rule-dimension-facts.md
    │
    └── archive/              已被 core 取代的旧版/摘录
```

---

## 策划正本（`design/core/`）

| 文档 | 回答什么问题 |
|------|----------------|
| [设计规则圣经_v0.3.txt](design/core/设计规则圣经_v0.3.txt) | 玩法、卡牌关系、版本节（**以文件末尾最新 v2.xx 节为准**） |
| [构建路线图_v3.x.md](design/core/构建路线图_v3.x.md) | Layer 0～7+ 分期：现在在哪层、下一层验收句 |
| [世界运转共识_v0.6.md](design/core/世界运转共识_v0.6.md) | 世界/行为规则、改代码前对齐什么 |
| [意向系统与世界演化_v0.3.md](design/core/意向系统与世界演化_v0.3.md) | 非任务驱动、意向冲突、社会演化方向 |

---

## 工程向设计（`design/` 根目录）

| 文档 | 用途 |
|------|------|
| [card-rule-audit.md](design/card-rule-audit.md) | 全卡规则审计 + 新卡 checklist |
| [incremental-iteration-workflow-v0.1.md](design/incremental-iteration-workflow-v0.1.md) | Codex 定方案、Cursor 实现、Codex 复盘的固定工作流 |
| [v3.0-a-river-valley-ecology-taoyuan-archetypes.md](design/v3.0-a-river-valley-ecology-taoyuan-archetypes.md) | 第一轮正式增量总案：四倍河谷生态 + 桃源三性格远观；工程按 A1/A2/A3 分三次合入 |
| [coupling-risk-mitigation-v0.2.md](design/coupling-risk-mitigation-v0.2.md) | v2.64–v2.70 耦合收口完成态 |
| [world-rule-tag-machine-v0.1.md](design/world-rule-tag-machine-v0.1.md) | 标签机 / WorldRules 目标 |
| [rule-dimensions-v0.1.md](design/rule-dimensions-v0.1.md) | 标签六维约束 |
| [capability-action-taxonomy-v0.1.md](design/capability-action-taxonomy-v0.1.md) | Capability / Action 分类 |
| [capability-migration-slices-v0.1.md](design/capability-migration-slices-v0.1.md) | 能力迁移切片 |
| [v2.26-mushroom-system.md](design/v2.26-mushroom-system.md) | 蘑菇系统落地摘录 |
| [layer0-intent-vocabulary.md](design/layer0-intent-vocabulary.md) | Layer 0 意向词汇 |
| [layer0-intent-backlog.md](design/layer0-intent-backlog.md) | Layer 0 意向 backlog |

---

## 常用命令

```powershell
# 改 CardDB 后刷新审计表
powershell -File tools/export_card_rule_facts.ps1

# 新卡脚手架（不生成玩法）
.\tools\run_card_factory_scaffold.ps1 <type> --tags=... --name=... --write-dir=generated/scaffold
```

---

## 改文档工作流

1. **只改** `docs/design/core/` 或 `docs/design/`  
2. 玩法变更 → 圣经末尾版本节 + [`CHANGELOG.md`](design/CHANGELOG.md)  
3. 工程切片 → [`CODEX_HANDOFF.md`](CODEX_HANDOFF.md) §12  
4. 桌面快捷入口：[`方寸商国_文档入口.md`](file:///E:/桌面/方寸商国_文档入口.md)

*文档整理：2026-05-20 | 正本在 `docs/design/core/`*
