# 耦合风险化解方案 v0.1

> **v0.2 已完成方案内 100%** — 见 [`coupling-risk-mitigation-v0.2.md`](coupling-risk-mitigation-v0.2.md)  
> 生成日期：2026-05-20  
> 前提：runtime 决策层已完成 v2.60–v2.66 规则化，v2.67a/b 已完成配方 / service / state 子域拆分起步，v2.68 已完成 pathfinding passable 规则化第一刀，v2.69a 已完成材料域拆分起步；CardDB `.tres` 化、AStarGrid2D、UI 规则化 **暂不纳入本方案**（见 §8）。  
> 主文档：`world-rule-tag-machine-v0.1.md`、`CODEX_HANDOFF.md` §12

---

## 1. 目标

把「改 A 坏 B」从 **靠经验避坑** 变成 **靠边界 + 守卫 + 小切片可验收**。

本方案不追求零耦合（卡牌模拟不可能），而是把剩余耦合 **收敛到少数枢纽**，并在枢纽处加 **可测试的不变量**。

---

## 2. 现状快照（v2.69a 后）

### 2.1 已收敛的枢纽（保持，勿再散落）

| 枢纽 | 职责 | 消费者 |
|------|------|--------|
| `WorldRules` | tag/capability/domain/service 查询；`card_matches_kind`；`ecosystem_behavior_key` | 任务、生态、needs、helpers |
| `CraftTaskManifest` | 任务 type → need/domain/phase 元数据 | `CraftTaskSystem`、L0 manifest 测试 |
| `NeedEvaluator` → `MissionResolver` → `ActionRunner` | eval 选 need，exec 推 chain/craft | 玩家主链 |
| `CardRuleAudit` + L0 | `CARD_CAPABILITIES` 与 audit 表同步 | 防数据漂移 |
| `NeedContract` + L0 | P0 need → chain / exec entry / can-start fixture | 防 eval/exec 错位 |
| `WorldRules` 状态谓词 + L0 扫描 | 核心 runtime state 判断集中到规则层 | 防状态字符串漂移 |
| `InteractionManager` + `CardDB` | 右键关系、敲击配方、spawn/remove | 手动 + AI 共用 |
| `WorldRules.can_craft_*` | Task 层配方询问入口，内部委托 `CardDB` | fire / shelter 已接入 |
| `world_rules/world_rules_craft.gd` | 配方域子模块，`WorldRules` facade 转发 | v2.67a 已开始 |
| `world_rules/world_rules_service.gd` | service provider 查询子模块，`WorldRules` facade 转发 | v2.67b 已开始 |
| `world_rules/world_rules_state.gd` | 核心 runtime state 谓词子模块，`WorldRules` facade 转发 | v2.67b 已开始 |
| `world_rules/world_rules_material.gd` | 材料 / 木材 / 容器 / 来源纯查询子模块，`WorldRules` facade 转发 | v2.69a 已开始 |
| `WorldRules.is_pathfinding_passable_occupant` / `is_pathfinding_blocking_occupant` | 寻路 passable / blocking 统一入口 | v2.68 已开始 |

### 2.2 仍存在的耦合类型

| 类型 | 严重度 | 典型症状 |
|------|--------|----------|
| **R1 数据双源** | 高 | CardDB tags 与 WorldRules 语义不一致；L0 未覆盖的 relation |
| **R2 eval/exec 错位** | 高 | need 激活但 handler 开不了工 → 发呆、stall、L2 flaky |
| **R3 WorldRules 膨胀** | 中 | 单文件 150+ API；改规则需懂全局 |
| **R4 状态字符串** | 中 | `"眩晕"` / `"离开"` / `"扎根"` 散落；同义 state 写法不一 |
| **R5 双 FSM 叠加** | 中 | `execution_stack` 与 `craft_task.phase` 抢占/恢复边界不清 |
| **R6 配方层 type 耦合** | 中 | `can_impact(a,b)`、`find_relation` 只认 type 字符串 |
| **R7 漏网硬编码** | 低–中 | 个别文件仍 `card_type ==`（如 `mushroom_helpers` 水桶） |
| **R8 UI/展示分叉** | 低 | `game_ui` / `world_manager` type 列表与规则层不同步 |
| **R9 Pathfinding 白名单** | 低–中 | 新 tag 卡能进规则但走不过去；v2.68 已清掉 passable type 白名单第一层 |

---

## 3. 化解原则（全局）

1. **单语义单入口**：同一种游戏语义（「是不是篝火」「旅人 demand 是否匹配」）只允许在 `WorldRules`（或明确登记的子模块）定义一次。
2. **eval 必须可 exec**：新增/修改 need 时，同步改 `MissionResolver.chain_for` + handler + `*_can_execute` 守卫；L0 加「need 激活 ⇒ 能开工」断言。
3. **小切片可验收**：每次只动一条链（如 commerce demand、wolf spawn、GH 备木）；必跑 L0 + L2b；更新 `CHANGELOG.md` + `CODEX_HANDOFF.md` §12。
4. **专名规则显式登记**：必须保留 `card_type` 的地方（如「长矛只要 twig」）在 `card-rule-audit.md` 写原因，并在 `WorldRules` 用命名函数封装（如 `is_twig_lumber`），禁止 task 内联。
5. **刻意保留清单**：测试 bootstrap、`pathfinding` 白名单、CardDB impact 映射、UI 展示 — 未列入切片前 **禁止顺手改**。

---

## 4. 分层化解方案

### 4.1 R1 数据双源 — 「一张事实表，两处只读」

**问题：** `CardDB`（注册时 tags）、`WorldRules.CARD_CAPABILITIES`（审计表）、`card-rule-audit.md`（人工）可能 drift。

**方案：**

```
CardDB.init() tags
        ↓ 导出（已有 card-rule-facts）
card-rule-audit.md（人工结论 + 待定）
        ↓ L0 断言
WorldRules.CARD_CAPABILITIES（必须与 audit 一致）
        ↓ 运行时只读
任务 / 生态 / needs（禁止再写 type 列表）
```

**动作：**

| 步骤 | 内容 | 验收 |
|------|------|------|
| D1 | 新增卡/改 tag：**先**改 `CardDB._reg`，**再**改 audit 表，**再**改 `CARD_CAPABILITIES` | L0 `_test_card_rule_audit_integrity` PASS |
| D2 | 新增 L0：`type_has_tag("fire","camp.anchor")` 与 `is_camp_fire_anchor` 等价类抽样 | 防 tag 改名漏改 WorldRules |
| D3 | 每周或每个 v2.x 切片：跑 `card_rule_facts_export_runner`  diff audit | 无静默 drift |

**禁止：** 在 task 脚本里复制 CardDB 的 tag 数组。

---

### 4.2 R2 eval/exec 错位 — 「need 契约表」

**问题：** `NeedEvaluator` 返回 need，但 `ActionRunner` / `CampHelpers.storage_can_execute` 等可能不满足开工条件。

**方案：** 维护 **Need 契约表**（当前落地为 `scripts/player/execution/need_contract.gd`）：

| need | 激活条件（eval） | 开工条件（exec） | 失败时 note_blocked 文案 |
|------|------------------|------------------|---------------------------|
| `build_greenhouse` | `should_player_build_greenhouse` | GH 未建成 + 可 start buildGreenhouse | 建棚推不动 |
| `commerce` | 旅人有 demand + 有 table | `commerce_can_execute` / 有货或可 cook | 经营暂无进展 |
| `storage_idle` | camp 有待存 + `storage_can_execute` | 同左 | 归置暂无目标 |
| … | … | … | … |

**动作：**

| 步骤 | 内容 | 验收 |
|------|------|------|
| E1 | 为 P0 need（fire, hunger, build_greenhouse, commerce, cleanliness, storage_idle）补全契约表文档 + 代码注释 | **v2.64 已完成** |
| E2 | L0：对每个 need 构造最小 fixture，`evaluate` 激活时对应 `*_can_execute` 为 true | **v2.64 已完成**：`_test_need_exec_contract` |
| E3 | L2b flaky 时 **先查契约表**，再改逻辑（避免只改 eval 或只改 exec） | L2b 连续 3 次 PASS 记 CHANGELOG |

**核心不变量（已有，强化执行）：**

> eval 必须与 exec 对齐 — need 只有在对应 handler **真能开工** 时才应激活。

---

### 4.3 R3 WorldRules 膨胀 — 「按域拆文件，对外一个入口」

**问题：** `world_rules.gd` 过大，认知负担高。

**方案：** 物理拆分、逻辑不分叉：

```
scripts/core/world_rules/
  world_rules.gd          # 薄 facade：static func 转发
  world_rules_tags.gd     # tag/capability 基础
  world_rules_camp.gd     # domain.camp / fire / hut / table
  world_rules_hunt.gd     # hunt / prey / wolf pack
  world_rules_material.gd # lumber / stone / container
  world_rules_commerce.gd # product / demand / traveler
  world_rules_ecosystem.gd# behavior_key / spawn avoid
```

**动作：**

| 步骤 | 内容 | 验收 |
|------|------|------|
| W1 | 先 **复制转发**（facade 保留原 API 名），零行为变更 | **v2.67a/b 已完成第一批**：配方 / service / state 子域 L0 / L1 / greenhouse / live PASS |
| W2 | 新 API 只加在对应子文件；facade 加一行转发 | 无 task 直接 preload 子文件 |
| W3 | `CARD_CAPABILITIES` 留在单文件或 `world_rules_capabilities.gd` | audit 测试仍 PASS |

**禁止：** 拆分同时改语义；一次 PR 只做搬家。

---

### 4.4 R4 状态字符串 — 「状态 → 规则查询」

**问题：** `card.state == "眩晕"`、`c.state != "离开"` 等与规则混杂。

**方案：** 在 `WorldRules` 增加 **状态谓词**（内部仍读 `state`，对外不暴露字符串）：

| 谓词 | 含义 |
|------|------|
| `is_stunned_being(card)` | 可击晕生物处于眩晕 |
| `is_traveler_departing(card)` | 旅人离开流程 |
| `is_tree_depleted(card)` | 树枯竭/受损等 |

**动作：**

| 步骤 | 内容 | 验收 |
|------|------|------|
| S1 | 审计全项目 `card.state` / `str(card.state)` 出现点 | **v2.65 已完成**，见附录 A |
| S2 | 每次替换 1–2 个高频 state（眩晕、离开、扎根） | **v2.65 已完成**：眩晕 / 离开 / 枯竭 / 受损 / 扎根 |
| S3 | audit 表增加「运行时 state 枚举」列 | 与谓词一一对应 |

---

### 4.5 R5 双 FSM — 「明确优先级表」

**问题：** `PlayerExecutionStack`（chain/step）与 `craft_task`（type/phase）谁优先、何时 pop/push 不清晰。

**方案：** 写死 **优先级表**（代码 + 文档）：

```
优先级（高 → 低）：
  survival tier need（hunger/fire/wolf_react）
  → craft_task 进行中（非 lumber 子链）
  → execution_stack chain
  → lumber 子链（可嵌 GH）
  → idle_scan
```

**动作：**

| 步骤 | 内容 | 验收 |
|------|------|------|
| F1 | 在 `action_runner.gd` 顶部注释或 `chain_registry.gd` 登记优先级 | 与 `_sync_stack` 行为一致 |
| F2 | L0：atomic carry move（moveTable/moveHut）不被低优先级 need 误清 | 已有 `_is_atomic_carry_move` 扩展测试 |
| F3 | stall/abandon 只走 `PlayerExecutionRecovery` + `ActionRecovery` | 无第三处 reset path |

---

### 4.6 R6 配方层 type 耦合 — 「WorldRules 问，CardDB 答」

**问题：** AI 用 `WorldRules.is_lumber_material`，合成仍用 `can_craft("twig","grass","hut")`。

**方案：** 短期不重写 CardDB；加 **薄封装**：

```gdscript
# WorldRules 或 CraftRules
static func can_craft_cards(a: CardBase, b: CardBase, expect_result: String = "") -> bool:
    if not is_instance_valid(a) or not is_instance_valid(b):
        return false
    return can_craft_types(a.card_type, b.card_type, expect_result)
```

Task 层逐步改为传 `CardBase` 不传 type 字符串。

**动作：**

| 步骤 | 内容 | 验收 |
|------|------|------|
| C1 | `shelter_tasks` / `fire_tasks` 内 `can_craft(x.card_type, …)` → `can_craft_cards(x, …)` | **v2.66 已完成**：L0 craft rule card query PASS |
| C2 | 新增 L0：同 tag 不同 type 的 negative case（若有专名规则） | 文档登记原因 |
| C3 | `tool_tasks` 长矛 / `survival_tasks` 熟肉继续改 `can_craft_cards` | 后续 R6 小切片 |

长期（可选）：impact/relation 改 tag 匹配 — **单独大切片，不在 v0.1 做**。

---

### 4.7 R7 漏网硬编码 — 「grep 门禁 + 最后一轮清扫」

**方案：** CI/本地脚本（或 L0 前脚本）：

```powershell
# 允许路径：world_rules.gd, card_db.gd, card_base.gd, game_ui.gd,
#           world_manager.gd, pathfinding.gd, test/*, interaction_manager.gd
rg "card_type\s*==|card_type\s+in\s+\[" scripts/player scripts/world scripts/core/world_helpers.gd
# → 出现即 fail，除非行尾有 # coupling:allow 注释并在 audit 登记
```

**v2.64 已清：**

| 文件 | 片段 |
|------|------|
| `mushroom_helpers.gd` | `bucket.card_type != "waterbucket"` → `is_water_bucket` |
| `world_helpers.gd` | `card.card_type == "player"` → `card is PlayerCard` |
| `manual_control.gd` | `card_type == "player"` → `card is PlayerCard` |

**仍刻意保留：**

| 文件 | 原因 |
|------|------|
| `world_manager.gd` | UI/展示层，暂列 runtime coupling scan 白名单；v2.65+ 单独收口 |
| `camp_helpers.gd` | storage key 仍用 `card.card_type` → `camp_storage_kind(card)` |
| `player_needs_manager.gd` | 非 equality/list 分支为任务参数与配方 type，后续随 R6 处理 |

---

### 4.8 R8 UI 展示分叉 — 「只读 WorldRules，不改模拟」

**方案：** UI 层 **允许** 继续用 type 做显示名；**条件分支**（是否画热量条、是否可点）改调 WorldRules：

| UI 条件 | 替换为 |
|---------|--------|
| `card_type in ["waterbucket","halfbucket"]` | `is_heated_water_container(card)` |
| `card_type == "tree"` | `is_lumber_source_tree(card)` |
| 可点击生物列表 | `is_being_card(card)` 或更细 predicate |

**动作：** 独立切片 v2.65+；**不影响** L2b 模拟验收（可选 eyeball）。

---

### 4.9 R9 Pathfinding 白名单 — 「tag 驱动 passable」

**问题：** 新卡 tags 正确但不在 `pathfinding.gd` passable 列表 → 寻路失败。

**方案：**

```gdscript
static func is_passable_for(card: CardBase, actor) -> bool:
    if _WorldRules.is_living_grass(card): return true
    if _WorldRules.is_forage_food(card) and card.card_type == "berry": return true
    # … 逐步从白名单 migrate 到 WorldRules
```

**动作：**

| 步骤 | 内容 | 验收 |
|------|------|------|
| P1 | 新卡 checklist：audit + WorldRules + **pathfinding passable** 三项齐 | **v2.68 已完成**：checklist 入 `card-rule-audit.md` 模板 |
| P2 | L0：berry/grass/bucket 对 player passable；sheep/tree/fire 阻挡 | **v2.68 已完成**：L0 pathfinding passable rules |

**v2.68 当前策略：**

- `Pathfinding.blocks_actor` 不再维护独立 passable type 白名单，先询问 `WorldRules.is_pathfinding_passable_occupant`。
- 通过项：草 / 干草覆盖、灌木、可采食物、水容器、非锁定营地可收纳物、散落可搬物。
- 阻挡项：rooted、businessUnit、shelter、生物、尸体、营地域定义者。
- 默认项：未知未登记类别默认阻挡；新增卡需要在审计 checklist 里明确寻路规则。

---

## 5. 推荐切片顺序（v2.64–v2.69a）

| 版本 | 主题 | 化解风险 | 预估 |
|------|------|----------|------|
| **v2.64** | 漏网清扫 + need 契约 L0 | R7, R2 | **已完成** |
| **v2.65** | 状态谓词（眩晕/离开/树木） | R4 | **已完成** |
| **v2.66** | `can_craft_cards` + shelter/fire | R6 | **已完成** |
| **v2.67** | WorldRules 按域拆文件（只搬家） | R3 | **进行中**：v2.67a/b 配方、service、state 子域已拆并验收 |
| **v2.68** | pathfinding tag 化 + 新卡 checklist | R9 | **已完成本轮目标**：passable/blocking 第一刀 + 新卡 checklist + live trace 摘要输出 |
| **v2.69a** | WorldRules 材料域拆分起步 | R3 | **已完成**：material/container/source 查询搬入子模块 |

每切片：**L0 + L2b**；更新 CHANGELOG + CODEX_HANDOFF §12。

---

## 6. 守卫清单（每个 PR 自检）

- [ ] 是否在 task/behavior 里新增 `card_type ==`？（若是 → 改 WorldRules）
- [ ] 是否新增/修改 need？（若是 → 更新 need 契约表 + L0 exec 测试）
- [ ] 是否改 CardDB tag？（若是 → audit + CARD_CAPABILITIES + L0 audit 测试）
- [ ] 是否改 preempt/stall/stack？（若是 → 对照 FSM 优先级表）
- [ ] 是否动 pathfinding？（若是 → passable 与 WorldRules 一致）
- [ ] L0 断言数是否增加或保持？L2b 是否至少跑 1 次？

---

## 7. 成功指标

| 指标 | 当前（v2.69a） | 目标（v2.69 后） |
|------|---------------|------------------|
| runtime `card_type ==`（不含 world_rules/card_db/test/ui） | **0**（由 L0 扫描守卫） | **0** |
| runtime 核心 state 直接比较（不含 world_rules/test/ui） | **0**（由 L0 扫描守卫） | **0** |
| L0 断言 | **617** | ≥580（契约+path+state） |
| L2b | v2.69a PASS tick=1334 / wall=31.4s；卡滞段 0 | 连续 3 次 PASS |
| 新卡接入步骤 | `card-rule-audit.md` checklist 已落地；pathfinding 规则入口已接入 | audit checklist 可执行并逐步加 L0 |
| WorldRules 单文件行数 | ~1500+ | facade + 6 子模块 |

---

## 8. 刻意不在本方案内（用户已决定暂缓）

- CardDB → `.tres` 资源化  
- `AStarGrid2D` 替换 BFS  
- 完整 ECS / 第三方 StateMachine 插件  
- CardDB relation 改为纯 tag 匹配（大重构）

---

## 9. 附录 A — state 字符串扫描（v2.65）

> 状态赋值仍允许分布在状态机输出处；核心状态判断必须走 `WorldRules` 谓词。`CardRuleAudit.validate()` 已加入扫描守卫。

| state 值 | 出现模块 | 建议谓词 |
|----------|----------|----------|
| `"眩晕"` | hunt / butcher / stun tick | `is_stunned_being` |
| `"离开"` | traveler behavior / commerce demand | `is_traveler_departing` |
| `"枯竭"` | tree regen / chop guard | `is_tree_depleted` / `is_tree_depleted_state` |
| `"受损"` | tree regen after chop pause | `is_tree_damaged` |
| `"扎根"` | tree drop eligibility | `is_tree_active` |
| `"死亡"` | player lifecycle | 待后续切片，当前未纳入核心谓词 |
| `"消费中"` / `"逃跑"` / 伏击状态 | 局部行为 FSM | 待后续切片，暂不扩大范围 |

---

## 10. 附录 B — 相关文件索引

| 文件 | 耦合角色 |
|------|----------|
| `scripts/core/world_rules.gd` | 规则枢纽 |
| `scripts/player/execution/need_evaluator.gd` | eval 入口 |
| `scripts/player/execution/mission_resolver.gd` | need → chain |
| `scripts/player/execution/action_runner.gd` | exec + stack |
| `scripts/player/craft_task_system.gd` | craft FSM 生命周期 |
| `scripts/player/craft_task_manifest.gd` | 任务元数据 |
| `scripts/test/card_rule_audit.gd` | 数据同步守卫 |
| `scripts/core/pathfinding.gd` | 寻路 passable |
| `scripts/core/interaction_manager.gd` | 配方执行 |
| `scripts/cards/card_db.gd` | 类型与 relation 源 |

---

*本文档随切片推进更新；下一步建议继续拆 `WorldRules` 的 commerce 小域，或先补 L2b 连续 3 次 PASS 统计。*
