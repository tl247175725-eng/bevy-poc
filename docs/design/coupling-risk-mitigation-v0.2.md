# 耦合风险化解方案 v0.2（方案内 100% 完成）

> 更新日期：2026-05-27  
> 前版：`coupling-risk-mitigation-v0.1.md`  
> 主文档：`world-rule-tag-machine-v0.1.md`、`CODEX_HANDOFF.md` §12

---

## 完成度总览

| 范围 | 状态 | 说明 |
|------|------|------|
| runtime 决策层 `card_type` 硬编码 | **100%** | L0 `CardRuleAudit.runtime_type_coupling_errors()` 扫描 player/world/helpers/ui/input |
| runtime 核心 state 硬编码 | **100%** | 谓词在 `world_rules_state.gd`；扫描守卫 + 任务层清零 |
| eval/exec need 契约 | **100%** | `need_contract.gd` + `_test_need_exec_contract` |
| pathfinding passable | **100%** | v2.68 `WorldRules.is_pathfinding_*` + L0 |
| 新卡 checklist | **100%** | `card-rule-audit.md` 顶部表 |
| WorldRules 子域拆分 | **100%（方案内）** | 7 子模块 + facade |
| UI/展示条件分支 | **100%** | `world_rules_ui.gd` + `game_ui` / `world_manager` / `drag_manager` |
| L2b 稳定性 | **100%（方案指标）** | 连续 3× PASS：1494 / 1309 / 1339 |
| CardDB 配方 type 匹配 | **刻意保留** | §8 范围外 |
| CardDB `.tres` / AStarGrid2D | **刻意保留** | §8 范围外 |

**方案内耦合风险化解：100%。** 剩余耦合仅存在于 §8 刻意保留层与 `world_rules.gd` facade 内的专名 fallback（狩猎/种类匹配/尸体类型等），均有 audit 登记或 L0 守卫。

---

## WorldRules 子模块（facade 对外 API 不变）

| 子模块 | 职责 |
|--------|------|
| `world_rules_craft.gd` | 配方查询 |
| `world_rules_service.gd` | service provider |
| `world_rules_state.gd` | 眩晕/旅人离开/树木状态 |
| `world_rules_material.gd` | 材料/木材/容器/来源 |
| `world_rules_camp.gd` | 营地域/篝火/ hut/table/收纳/整理 |
| `world_rules_commerce.gd` | 商品/经营 demand/桌边 staging |
| `world_rules_ui.gd` | 展示/拖拽/ badge 刷新规则 |

主文件 `world_rules.gd` 保留：能力表、域查询入口、生态/狩猎/种类匹配、pathfinding、facade 转发。

---

## 守卫（持续有效）

1. **L0** `_test_card_rule_audit_integrity` — 能力表同步 + runtime type/state 扫描  
2. **L0** `_test_need_exec_contract` — P0 need 可 exec  
3. **L0** `_test_pathfinding_passable_rules` / `_test_ui_display_rule_queries`  
4. **新卡** — 先填 `card-rule-audit.md` checklist 再写代码  
5. **每切片** — L0 +（影响主线时）L2b；更新 CHANGELOG + CODEX_HANDOFF §12  

---

## 刻意不在方案内（长期 tech debt，非「未完成」）

- CardDB → `.tres` 资源化  
- `AStarGrid2D` 替换 BFS  
- CardDB relation 纯 tag 匹配（大重构）  
- 完整第三方 StateMachine / ECS  

---

## v2.64–v2.70 切片清单（均已落地）

| 版本 | 内容 |
|------|------|
| v2.64 | 漏网清扫 + need 契约 L0 |
| v2.65–67b | 状态谓词 / craft / service / state 子域 |
| v2.68 | pathfinding + 新卡 checklist |
| v2.69a | material 子域 |
| v2.70 | camp + commerce + ui 子域；UI 规则化；L2b 3× |

---

## 验收记录（v2.70）

- L0：**625** assertions PASS  
- L2b：**1494 / 1309 / 1339** tick PASS，均建成蘑菇棚  

---

*后续若扩展玩法，仍按 checklist + WorldRules 单入口 + L0/L2b 流程接入；勿在 task/UI 内联 `card_type`。*
