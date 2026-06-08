# 世界运转共识 — 摘录（v0.6 §19）

> 完整正文在桌面：`E:\桌面\方寸商国_世界运转共识文档_v0.1.md`（文件名 v0.1，正文 **v0.6**）  
> 与 Layer 0 完成态、增量迭代基线对齐。

---

## 增量迭代工程基线（v2.70）

### WorldRules 架构

主入口：`scripts/core/world_rules.gd`（facade，对外 API 不变）

| 子模块 | 职责 |
|--------|------|
| `world_rules_craft.gd` | 配方 / 敲击查询 |
| `world_rules_service.gd` | service provider |
| `world_rules_state.gd` | 眩晕 / 旅人 / 树木状态 |
| `world_rules_material.gd` | 材料 / 木材 / 容器 / 来源 |
| `world_rules_camp.gd` | 营地域 / 篝火 / hut / table / 收纳 |
| `world_rules_commerce.gd` | 商品 / 经营 demand / 桌边 staging |
| `world_rules_ui.gd` | 展示 / 拖拽 / badge 刷新 |

寻路：`Pathfinding` → `is_pathfinding_passable_occupant` / `is_pathfinding_blocking_occupant`

### 增量迭代守则

1. 新卡 → `card-rule-audit.md` checklist  
2. 规则查询 → 只增 `WorldRules`；禁止 runtime 新增 `card_type ==`  
3. 改 need → `need_contract.gd` + L0 契约测试  
4. 切片验收 → L0；主链影响 → L2b；CHANGELOG + CODEX_HANDOFF §12  

### 验收基线

L0 **625** · L2b **3× PASS**（tick=1494 / 1309 / 1339）

### 索引

- `coupling-risk-mitigation-v0.2.md` · `CODEX_HANDOFF.md` §0 · 主文档末尾 v2.64–v2.70

*摘录 | 2026-05-27*
