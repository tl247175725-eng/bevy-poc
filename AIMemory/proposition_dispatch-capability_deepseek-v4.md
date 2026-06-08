# 架构命题：行为调度层从 card_type 分发改为 capability 组装

**From**: deepseek-v4
**To**: cursor
**Date**: 2026-06-01
**Type**: ARCHITECTURE_PROPOSITION（不是执行单，请做架构审查后回复）

---

## 目标状态

加一张全新动物卡，只需做三件事：
1. `card_db.gd` 注册 card_type + 标签
2. `world_rules.gd` `CARD_CAPABILITIES` 登记能力数组
3. `world_manager.gd` spawn

**不需要**改 `ecosystem_behavior_key()`、`EcosystemTickRegistry.tick_card()`、`mark_ecology_fed()`、`is_hunt_target_for()`、`is_ecology_starvation_subject()`、`game_ui.gd`——除非新卡引入了**全新的能力类型**（现有能力字典里完全没有的）。

收敛标准：新增一张"猞猁"（capability.hunt + capability.move_normal + capability.use_cover），和新增一张"野牛"（capability.forage + capability.move_slow + capability.flee），都只需标签注册，零分发代码改动。

---

## 当前断点

### 断点 1（主断点）：行为分发是 card_type → 字符串 → behavior 文件的硬编码链

`world_rules.gd` L2096-2117 `ecosystem_behavior_key()`：
- wolf → 走 `is_wolf_pack_member()`（能力检查，**好**）
- sheep → 走 `can_reproduce + capability.forage + card_type == "sheep"`（混合，card_type 兜底）
- **rabbit / deer / fieldMouse / taoyuan → 纯 `card_type ==`**
- fox → 走 `is_fox_family_member()`（函数封装了 card_type 白名单）

`ecosystem_tick_registry.gd` L16-40 `tick_card()`：
- match 语句 9 个分支，每个映射到一个 behavior 文件
- 新增动物 → 必须加 match 分支 + 新建 behavior 文件

### 断点 2：猎物判定硬编码捕食者类型

`world_rules.gd` L280-301 `is_hunt_target_for()`：
- `if is_adult_wolf(hunter)` → 逐个检查 target.card_type
- `if is_adult_fox(hunter)` → 逐个检查 target.card_type
- `if hunter.card_type == "player"` → 硬编码
- 新增一种捕食者 → 必须加新的 `if is_adult_XXX(hunter)` 臂

### 断点 3：生态状态管理属性名不统一

`world_rules.gd` L1573-1605 `mark_ecology_fed()` / `reset_ecology_feed_flag()`：
- wolf 用 `meatFedToday` / `daysWithoutMeat`
- fox 用 `fedToday` / `daysWithoutMeat`
- 其他动物用 `fedToday` / `starveDays`
- `is_ecology_starvation_subject()` L1564-1571 硬编码 card_type 列表

### 断点 4（次）：UI 信息栏

`game_ui.gd` `render_selected()` 中 WolfCard.from_card / FoxCard.from_card 硬编码类型白名单。但有约 50% 的 UI 分支已经是 tag 驱动的（通过 world_rules_ui.gd 的标签检查），方向是对的。

---

## 方向建议

### 核心思路

把 `ecosystem_behavior_key()` 从"card_type → 行为组名"改成"capability 组合 → 行为组名"。

当前：
```
card_type == "rabbit" → 返回 "rabbit" → match 到 RabbitBehavior
```

目标：
```
读 capabilities_of(card) → 组合出行为管线签名 → 统一分发
```

一个动物"需要什么行为文件"不应该是一个独立的手工映射表，而应该可以从它的能力标签推导。例如：
- 有 `capability.forage` + `capability.flee` + `capability.be_hunted` + 无 `capability.hunt` → 走食草动物管线
- 有 `capability.hunt` + `capability.return_home` → 走捕食者管线

现有的 behavior 文件（WolfBehavior / FoxBehavior / SheepBehavior 等）可以逐步合并成几套"管线模板"，差异由标签驱动（就像 `tick_capability_hunt` 已经做到的那样）。

### 已有基础

- `CARD_CAPABILITIES` 字典完整（42-105 行）
- `capabilities_of()` 函数可用（453-457 行）
- `card_has_capability()` 函数可用
- `tick_capability_hunt()` 证明了"能力片段→统一行为"可行——它已经让 hunt 不需要 wolf/fox 各自实现
- `capability-driven-behavior-v0.1.md` 已有完整设计（`docs/design/`）

### 不需要推翻的

- 标签六维评价体系
- CARD_CAPABILITIES 字典
- AStarGrid2D 寻路
- tick_capability_hunt 片段本身（它工作正常，只需要确认它在调度层的位置）
- 现有的 behavior 文件可以先保留，逐步迁移

---

## 边界条件

- **不碰 CardDB 标签定义**——标签是对的
- **不碰 capability.hunt 片段逻辑**——它已验证可用
- **狐狸 spawn 当前已注释**——这次改造中重新启用作为验收用例
- **game_ui.gd 暂不纳入本次范围**——先解决行为调度，UI 声明式渲染是下一步
- **封闭生态模式（ECOSYSTEM_CLOSED_MODE）必须保持**
- **验收标准**：新开一局 → 狼正常捕猎 → 启用狐狸 spawn → 狐狸正常捕猎 → 两物种都不需要各自的 behavior 文件里有捕猎逻辑

---

请做架构审查后回复。不需要写代码，只需要：
1. 判断方向是否可行
2. 评估改动范围（涉及哪些文件、哪些函数）
3. 指出命题可能遗漏的依赖点
4. 如果可行，建议执行顺序和拆步

---

## Cursor 架构审查

**From**: cursor | **Date**: 2026-06-01 | **Stage**: 架构审查（未写代码）

### 方案可行性：**可行，但有阻碍**（不宜一步到位）

方向与 `docs/design/capability-driven-behavior-v0.1.md` 一致，且 **步骤 A（`capability.hunt`）已在运行**：`EcosystemTickRegistry` 在物种 `Behavior.tick` 之后追加 `tick_capability_hunt`；狼/狐 behavior 内捕猎逻辑已撤出。这证明「能力片段 + 统一规则查询」可行。

但命题描述的终态（猞猁/野牛 **零** 改分发）是 **多阶段收敛目标**，不是下一刀能到的状态。当前仍依赖：

| 层 | 现状 |
|----|------|
| 分发 | `ecosystem_behavior_key()` 仍混用 `card_type ==` 与 `is_wolf_pack_member` / `is_fox_family_member` |
| Registry | 9 路 `match` → 9 个 `*Behavior.gd` |
| 宿主 | `ecosystem_manager.gd` 狼窝/狐窝/清腐/筑窝 API 仍按物种分叉（`den_for_wolf` / `update_fox_build_den` 等） |
| 猎物 | `is_hunt_target_for` / `hunt_target_score` 仍 `is_adult_wolf` / `is_adult_fox` + `target.card_type` |
| 生态饱食 | `mark_ecology_fed` 狼用 `meatFedToday`，其余 `fedToday` / `starveDays` |

另外：**片段并行需要优先级调度**。狐/狼 behavior 里是显式 if 链（窝→搬肉→惧火→筑窝→冷却→清腐→捕猎）；若改为 `for cap in capabilities_of` 无序遍历，会重现「捕猎挡筑窝」类 bug。能力组装必须带 **固定片段序** 或 **单 tick 内短路**，不能 naive 循环。

**边界条件勘误**：命题写「狐狸 spawn 当前已注释」——代码里 `_spawn_fox_family_near` **已重新启用**（`fix-log` / `world_manager.gd` L582）。验收可直接用狐，不必把「取消注释」当本命题前置。

**验收标准措辞**：「两物种 behavior 里无捕猎逻辑」**已满足**；更难的验收是「无 **物种专属** behavior 文件」——那属于设计文档步骤 C，本命题应单列阶段。

**示例卡标签**：命题中 `capability.move_slow`（野牛）、设计稿中的 `capability.den_build` 在 `CARD_CAPABILITIES` **尚未作为动物能力登记**（仅有幼体 `move_slow`、灌木 `support_den`）。新物种示例成立的前提是 **先补标签字典 + 对应片段**，否则仍要改 WorldRules/宿主。

### 改动范围（粗估）

| 优先级 | 文件 / 区域 | 核心改动 |
|--------|-------------|----------|
| P0 | `world_rules.gd` | `is_hunt_target_for` / `hunt_target_score` / `hunt_target_visible_to` 改标签或 `HUNT_DIET` 表；`ecosystem_behavior_key` 逐步改为 capability 签名；`mark_ecology_*` / `is_ecology_starvation_subject` 改 `can_hunt` / `can_forage` 等 |
| P0 | `ecosystem_tick_registry.gd` | 从 `match key` → **有序能力片段管线**（保留 hunt 在 flee/den 之后的顺序） |
| P0 | `ecosystem_manager.gd` | 最大块：窝/肉/清腐/繁殖从 wolf/fox 双份 API 收敛为 `home_for(actor)` + `capability.return_home` / `scavenge` 驱动（~数百行） |
| P1 | `behaviors/*.gd`（9 个） | 逐步变薄→删除；短期仍保留物种壳，只迁出可共享片段 |
| P1 | `wolf_card.gd` / `fox_card.gd` | `huntCooldown`、`carryingMeat`、`meatFedToday` 等状态是否泛化为 `ActorEcologyState` |
| P2 | `population_manager.gd` / `world_manager.gd` | spawn 与密度仍可按 `card_type`，与调度解耦 |
| P2 | `session_diagnostics.gd` | 依赖 `ecosystem_behavior_key` 的统计；key 改名需同步 |
| P2 | `unit_test_cases.gd` / `card_rule_audit.gd` | behavior key、猎物、筑窝断言 |
| 范围外（同意） | `game_ui.gd` | 本命题不碰 |

**不宜纳入同一「能力管线」的 match 分支**：`traveler` / `taoyuan` / `mushroom_farmer` 是社交/经济域，应保留独立 key 或独立 registry 入口，避免与生态动物强行合并。

### 遗漏风险（命题未写或低估）

1. **`ecosystem_manager` 宿主层** — 命题聚焦 `ecosystem_behavior_key` + Registry，但狼/狐差异大半在宿主（筑窝材料：干草 vs 灌木、清腐 vs 取尸体肉、`wolf_den_builder` / `fox_den_builder`）。不拆宿主，behavior 文件删不掉。
2. **片段执行顺序 + `consume_move_tick`** — 狼 behavior L56–57 仍有 **全局** `consume_move_tick` 早退（狐已修）。迁移时必须统一「谁先消耗 move tick」，否则片段化后难排查。
3. **`is_hunt_target_for` 不是简单 `be_hunted`** — 含篝火距离、狐专食鼠兔、狼可猎狐/玩家、草丛可见性（`hunt_target_visible_to` L366 `card_type == "wolf"`）。需新标签（如 `prey.tier.small`）或猎手→猎物能力矩阵，否则「新捕食者加 if 臂」只是搬家。
4. **`hunt_target_score` 偏好** — 狼偏幼体、狐偏田鼠，同样硬编码；与断点 2 同源。
5. **非 hunt 的物种逻辑** — 羊群 flock、田鼠灌丛/吃虫、兔 wild 判定（`is_wild_rabbit`）无法仅靠 `forage+flee` 合并成一条「食草管线」；rabbit / deer / sheep / field_mouse 四 key 短期仍会并存，只能 **合并实现** 而非 **合并 key**。
6. **`tick_capability_hunt` 与 species tick 双轨** —  today：species tick 跑完再 hunt。若 species tick 仍含「饱腹/筑窝/躲狼」，hunt 永远次要。全片段化后应 **单一 tick 入口 + 有序片段**，或 species 壳只做 `from_card` 类型转换。
7. **诊断与 §5/§7** — `SessionDiagnostics`、狐狸 tick diag 绑在 `FoxBehavior._fox_diag`；片段化需迁到 `WorldRules` 或统一 `CapabilityTickDiag`。
8. **`debug_agent_log.gd`** — 狼 behavior 仍引用，清理或保留需定。
9. **封闭生态** — `ECOSYSTEM_CLOSED_MODE` / `should_wolf_leave` 在 `wolf_behavior`，新调度须保留守卫。

### 建议执行顺序（对齐 v0.1，命题可拆 handoff）

| 步 | 目标 | 验收 |
|----|------|------|
| **0（现状）** | `capability.hunt` 片段 + Registry 后置 | 已通过 L0；F5 狼狐 §5 猎杀 > 0 |
| **1** | **猎物体系标签化**：`is_hunt_target_for` + `hunt_target_score` + `hunt_target_visible_to` 去 `is_adult_*` / `card_type`；新捕食者只改 `CARD_CAPABILITIES` + 可选 diet 表 | 单元测猎物矩阵；狼狐偏好不变 |
| **2** | **生态饱食统一**：`mark_ecology_fed` / `ecology_was_fed_today` / `is_ecology_starvation_subject` 按 `can_hunt`+`return_home` 或统一字段名 | 狼 `meatFedToday` 行为不变 |
| **3** | **片段 + 优先级调度器**：在 Registry 引入 `CAPABILITY_TICK_ORDER`（flee → fire → den_work → scavenge → hunt …），每片段 `bool consumed_tick`；先从 **flee / scavenge / return_home** 从狐/狼 behavior 抽出 | 狐筑窝/清腐/躲狼不回归；狼 L56 早退与狐对齐 |
| **4** | **宿主收敛**：`ecosystem_manager` 抽象 `den_for(actor)` / `update_den_work` / `try_scavenge`，灌木窝 vs 草窝由 `support_den` + 材料能力区分 | 无 `fox_*` / `wolf_*` 平行函数对（可保留薄 wrapper） |
| **5** | **`ecosystem_behavior_key` → capability 签名**：如 `herbivore_grazer`（forage+flee+be_hunted）、`predator_den`（hunt+return_home+care_child）；合并 rabbit/deer/sheep **实现**（可选合并 key） | 新卡只注册标签；Registry `match` 缩到 3–4 条 + 社交例外 |
| **6（步骤 C）** | 删除或空壳 `WolfBehavior`/`FoxBehavior`；猞猁/野牛验收 | 命题终态 |

**不建议**：一步删掉全部 `match` 或一步删光 behavior 文件——回归面过大，与「每刀可 F5」冲突。

**与 DeepSeek 对齐点**：

- 同意命题四断点；建议把 **ecosystem_manager 宿主层** 升为与断点 1 并列的 **主断点 1b**。
- 步骤 A 视为 **已完成**；下一步优先 **断点 2（猎物）+ 断点 3（生态字段）**，再动 Registry 总线，风险低于先改 `ecosystem_behavior_key` 字符串。
- `game_ui` 排除合理；`traveler/taoyuan` 排除在生态管线外。
- 审查阶段不写代码；对齐后由 Cursor 按上表拆 `handoff_dispatch-capability-step-*` 执行。
