# 设计文档索引

> **入口：** 先看 [`../README.md`](../README.md)。策划正本在 **`core/`**，工程向文档在本目录根下。

---

## 策划正本（`core/` — 完整正文）

| 文件 | 说明 |
|------|------|
| [`core/设计规则圣经_v0.3.txt`](core/设计规则圣经_v0.3.txt) | 主文档；文件名 v0.3，**以末尾最新 v2.xx 节为准** |
| [`core/构建路线图_v3.x.md`](core/构建路线图_v3.x.md) | Layer 0～7+ 分期与验收句 |
| [`core/世界运转共识_v0.6.md`](core/世界运转共识_v0.6.md) | 世界/行为规则、改代码前对齐 |
| [`core/意向系统与世界演化_v0.3.md`](core/意向系统与世界演化_v0.3.md) | 意向驱动、社会演化 |

旧版摘录与 v0.2 意向文档已移入 [`archive/`](archive/)，勿再编辑。

---

## 规则圣经

正本：`docs/design/core/设计规则圣经_v0.3.txt`（入口 `docs/README.md`）

```powershell
# 改 CardDB 后刷新审计表
powershell -File tools/export_card_rule_facts.ps1
```

## 工程向文档

| 主题 | 文件 |
|------|------|
| 变更日志 | [`CHANGELOG.md`](CHANGELOG.md) |
| 增量迭代工作流 | [`incremental-iteration-workflow-v0.1.md`](incremental-iteration-workflow-v0.1.md) |
| v3.0-A 第一切片总案 | [`v3.0-a-river-valley-ecology-taoyuan-archetypes.md`](v3.0-a-river-valley-ecology-taoyuan-archetypes.md)（A1/A2/A3 分三次合入） |
| 卡牌审计 / 新卡 checklist | [`card-rule-audit.md`](card-rule-audit.md) |
| 耦合收口完成态 | [`coupling-risk-mitigation-v0.2.md`](coupling-risk-mitigation-v0.2.md) |
| 标签机目标 | [`world-rule-tag-machine-v0.1.md`](world-rule-tag-machine-v0.1.md) |
| 规则维度 | [`rule-dimensions-v0.1.md`](rule-dimensions-v0.1.md) |
| 能力 / 动作分类 | [`capability-action-taxonomy-v0.1.md`](capability-action-taxonomy-v0.1.md) |
| 能力迁移切片 | [`capability-migration-slices-v0.1.md`](capability-migration-slices-v0.1.md) |
| 蘑菇系统 | [`v2.26-mushroom-system.md`](v2.26-mushroom-system.md) |
| 意向词汇 / backlog | [`layer0-intent-vocabulary.md`](layer0-intent-vocabulary.md) · [`layer0-intent-backlog.md`](layer0-intent-backlog.md) |

### 自动导出（勿手改）

| 文件 | 来源 |
|------|------|
| [`generated/card-rule-facts.md`](generated/card-rule-facts.md) | `CardDB` |
| [`generated/rule-dimension-facts.md`](generated/rule-dimension-facts.md) | 维度事实 |
| [`card-rule-audit.md`](card-rule-audit.md) | 人工审计表（含 checklist） |

```powershell
powershell -File tools/export_card_rule_facts.ps1
```

---

## 版本速查

| 版本 | 主题 | 位置 |
|------|------|------|
| v2.64–v2.70 | 耦合收口 + 增量迭代基线 | 圣经末尾 · `coupling-risk-mitigation-v0.2.md` · `CODEX_HANDOFF.md` §0 |
| v2.72 | 卡牌工厂 Phase 1 | `card-rule-audit.md` · `CHANGELOG.md` |
| v2.73 | 卡牌工厂 Phase 2 脚手架 | `card_factory_scaffold.gd` · `CHANGELOG.md` |
| v3.x | 构建路线图 | `core/构建路线图_v3.x.md` |
| v3.0-A | 四倍河谷生态 + 桃源三性格远观；工程按 A1/A2/A3 分片 | `v3.0-a-river-valley-ecology-taoyuan-archetypes.md` |
| v0.3 | 意向系统 | `core/意向系统与世界演化_v0.3.md` |
| v0.1 | 增量迭代工作流 | `incremental-iteration-workflow-v0.1.md` |

---

## 工作流

**改玩法后：**

1. 更新 `core/设计规则圣经_v0.3.txt` 对应版本节  
2. [`CHANGELOG.md`](CHANGELOG.md) 追加摘要  
3. 大功能可更新专题 md（如 `v2.26-mushroom-system.md`）

**加新卡：**

1. `.\tools\run_card_factory_scaffold.ps1 <type> ... --write-dir=generated/scaffold`  
2. 人工粘贴片段 + 设计玩法（脚手架不生成 craft/need）  
3. L0 + L1.5；动主链 → L2b  
4. `export_card_rule_facts.ps1` + `CHANGELOG` + `CODEX_HANDOFF` §12

**改代码前：** 读圣经相关节 + `core/世界运转共识_v0.6.md` + `CODEX_HANDOFF.md` §0。
