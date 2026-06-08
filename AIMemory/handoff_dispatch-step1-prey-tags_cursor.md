# 调度改造 Step 1：猎物体系标签化

**From**: cursor  
**To**: cursor（自执行）  
**Date**: 2026-06-01  
**Priority**: HIGH  
**Parent**: `proposition_dispatch-capability_deepseek-v4.md`（6 步表 Step 1）  
**对齐依据**: Cursor 架构审查 + DeepSeek 对齐点（步骤 A 已完成；**本步只做断点 2 猎物**，不碰 Registry / `ecosystem_behavior_key` / 生态饱食 / `game_ui`）

---

## 目标

把 `WorldRules` 中猎物 **合法性**、**排序偏好**、**草丛可见性** 从 `is_adult_wolf` / `is_adult_fox` / `target.card_type` 改为 **CardDB 已有 tag + `CARD_CAPABILITIES` 查询**。

改完后：新增带 `capability.hunt` 的捕食者，只需在 `CARD_CAPABILITIES` 登记能力，并在（本步新增的）`HUNT_PROFILE` 表里挂一行或靠 tag 自动归类——**不必**再改 `is_hunt_target_for` 的 `if card_type ==` 臂。

行为不变量：现有 L0 猎物相关断言全绿；狼仍偏好兔/幼体；狐仍只猎鼠/兔；玩家仍不自动猎羔。

---

## 不碰（本 handoff 硬边界）

| 排除项 | 原因 |
|--------|------|
| `card_db.gd` 标签增删 | 命题边界：标签已够用 |
| `tick_capability_hunt` 主体逻辑 | 已验证 |
| `ecosystem_behavior_key` / `EcosystemTickRegistry` | Step 5 |
| `mark_ecology_fed` / 饥饿字段 | Step 2 |
| `ecosystem_manager` 狼狐宿主 API | Step 4 |
| `game_ui.gd` | 命题排除 |
| `is_adult_wolf` / `is_adult_fox` **其它调用点** | 筑窝/种群/诊断仍可用；本步只清 hunt 三函数内的物种分支 |

---

## 现状快照（改前）

| 函数 | 硬编码点 |
|------|----------|
| `is_hunt_target_for` L287-301 | `is_adult_wolf` + `target.card_type` player/fox；`is_adult_fox` + card_type 列表；`hunter.card_type == "player"` |
| `hunt_target_score` L345-360 | 同上 + 多组 `target.card_type in [...]` |
| `hunt_target_visible_to` L366 | `hunter.card_type == "wolf"` + `hiddenInGrass` |

**已知瑕疵（本步顺带修）**：狼分支 `target.card_type == "fox"` 要求 `can_be_hunted`，但狐卡 **无** `capability.be_hunted` → 该臂实为死代码。迁移时用目标 tag `mesopredator` 表达「可猎的中型食肉者」，与数值设计一致。

---

## 执行方案

### 1. 引入 `HUNT_PROFILE`（`world_rules.gd` 顶部常量区）

用 **猎手 profile** 集中描述 diet，避免在三个函数里各写一遍。

**Profile 解析**（`static func hunt_profile_for(hunter: CardBase) -> String`）：

| profile | 识别条件（无 card_type） |
|---------|-------------------------|
| `pack_predator` | `can_hunt(hunter)` 且 `card_has_tag(hunter, "predator")` |
| `mesopredator` | `can_hunt(hunter)` 且 `card_has_tag(hunter, "mesopredator")` |
| `tool_hunter` | `can_hunt(hunter)` 且 `card_has_tag(hunter, "actor")`（玩家） |
| `""` | 其它 |

> 狼：`predator` tag（card_db）。狐：`mesopredator`。玩家：`actor`。与现有 CardDB 一致，**不加新 tag**。

**Profile 数据**（字典常量 `HUNT_PROFILE_RULES`）建议字段：

```gdscript
# 伪结构 — 实现时用 Dictionary
{
  "pack_predator": {
    "require_be_hunted": true,           # 默认猎物
    "also_allow_tags": ["mesopredator"], # 竞食者（狐）
    "also_allow_actor": true,            # 玩家实体
    "respect_fire": true,                # 目标近火则不可猎
  },
  "mesopredator": {
    "require_be_hunted": true,
    "require_any_target_tags": ["smallPrey", "smallHerbivore"],
    "threat_radius": 3.0,                # hunt_threat_near
  },
  "tool_hunter": {
    "require_be_hunted": true,
    "deny_target_capability": "capability.be_cared_for",
  },
}
```

### 2. 重写 `is_hunt_target_for`

流程：

1. 基础校验（实例、非携带、非眩晕、`can_hunt`）— 保持。
2. `var profile := hunt_profile_for(hunter)`；空则 `false`。
3. 调 `_hunt_target_allowed_by_profile(hunter, target, profile)`：
   - `pack_predator`：`can_be_hunted(target)` **或** `card_has_tag(target,"mesopredator")` **或**（`also_allow_actor` 且目标为玩家卡）；凡 `respect_fire` 的目标须 `not _near_fire(target)`。
   - `mesopredator`：目标须 `can_be_hunted` 且带 `smallPrey` 或 `smallHerbivore`；且 `not hunt_threat_near(hunter, threat_radius)`。
   - `tool_hunter`：`can_be_hunted` 且非 `be_cared_for`。
4. **禁止** 出现 `is_adult_wolf` / `is_adult_fox` / 猎物 `card_type ==` / `card_type in`。

### 3. 重写 `hunt_target_score`

在 `hunter.distance_to(target)` 基础上，按 **profile + 目标 tag** 加减分（复刻现网数值）：

**`pack_predator`**（对齐现狼逻辑）：

| 条件 | Δscore |
|------|--------|
| 目标 `mesopredator` | +2.5 |
| 目标 `juvenile` | -1.5 |
| 目标 `smallPrey` 或 `smallHerbivore`（且非上两行已处理） | 可与 juvenile 重叠，保持 -1.5 |
| 目标 `largePrey` 且 `can_reproduce(target)` | 0 |
| `can_be_hunted` 且非 reproduce 且非 grow | -1.0 |

**`mesopredator`**（对齐现狐逻辑）：

| 条件 | Δscore |
|------|--------|
| `smallPrey` | -2.0 |
| `smallHerbivore` | -1.0 |
| 其它（若误入候选） | `INF` — 防御性；合法候选应已被 `is_hunt_target_for` 滤掉 |

**`tool_hunter`**：仅距离，无偏好加减。

### 4. 重写 `hunt_target_visible_to`

- 将 `hunter.card_type == "wolf"` 改为 `hunt_profile_for(hunter) == "pack_predator"`（或 `card_has_tag(hunter,"predator")`）。
- 规则不变：`hiddenInGrass` 且距离 > 1 → 不可见。
- 狐 **不** 获得草丛免疫（现网如此）；勿扩大 scope。

### 5. 保持下游签名不变

以下 **只调用** 上述三函数，本步若行为等价则不必改：

- `best_hunt_target`
- `is_ambush_prey_for`
- `tick_capability_hunt`
- `need_evaluator.gd` / `player_needs_manager` / `player_hunt_helper`

### 6. 单元测试（`unit_test_cases.gd`）

**保留并通过** 现有块：

- `_test_deer_ecology_chain`（狼猎鹿、兔优先分）
- `_test_hunt_and_care_rule_queries`（狼/玩家目标集、草中兔不可见、`best_hunt_target`）

**新增** `_test_hunt_profile_matrix`（建议 12–16 断言）：

| 断言 |
|------|
| `hunt_profile_for(wolf) == "pack_predator"` |
| `hunt_profile_for(fox) == "mesopredator"` |
| `hunt_profile_for(player) == "tool_hunter"` |
| 狐 `is_hunt_target_for(fox, fieldMouse)` ✓ |
| 狐 `is_hunt_target_for(fox, rabbit)` ✓ |
| 狐 `not is_hunt_target_for(fox, sheep)` |
| 狐 `not is_hunt_target_for(fox, deer)` |
| 狼 `is_hunt_target_for(wolf, fox)` ✓（修死代码后） |
| 狼 `not is_hunt_target_for(wolf, wolf)` |
| 狐 `best_hunt_target` 在鼠+兔同场时选更近或按分（固定坐标） |
| 近火玩家：狼 `not is_hunt_target_for`（若现有 campfire fixture 可复用） |

### 7. 文档（极简）

- `docs/design/capability-migration-slices-v0.1.md` 或 `CHANGELOG.md` 追加 3 行：Step1 猎物 profile 化，指向 `HUNT_PROFILE_RULES`。
- **不写** 超 200 行新文档。

---

## 文件清单

| 文件 | 操作 |
|------|------|
| `scripts/core/world_rules.gd` | 主改：`HUNT_PROFILE_*`、`hunt_profile_for`、三函数 |
| `scripts/test/unit/unit_test_cases.gd` | 新测试块 + 注册到 runner |
| `docs/design/CHANGELOG.md`（或 migration-slices） | 1 段记录 |

---

## 验收

1. **L0**：`unit_test.tscn` 全绿（≥741，新测增加断言数）。
2. **行为回归**（与 Step 0 相同）：
   - 狼 `best_hunt_target` 仍偏兔；
   - 草中远距兔仍不可见；
   - 玩家不猎羔。
3. **F5（建议）**：§5 狼/狐猎杀 > 0；无需改 behavior 文件。
4. **fix-log**：一条 FIX「猎物 profile 化」；`work.log` 一行 `TASK_DONE`。

---

## 本步不做（留给后续）

- `is_active_hunt_threat` 仍用 `is_adult_wolf`（狐躲狼依赖「狼=威胁」）— 可在 Step 1 末尾记 TODO，或 Step 3 改为 `predator` tag。
- `preference.*` 正式维度、`HUNT_DIET` 外置 JSON — 过度设计，本步字典常量即可。
- 猞猁验收卡 — Step 6；本步只保证 **表驱动可扩展**。

---

## 执行顺序（实现时）

1. 加 `HUNT_PROFILE_RULES` + `hunt_profile_for`（无行为变更）
2. 改 `is_hunt_target_for` + 跑 L0
3. 改 `hunt_target_score` + 跑 L0
4. 改 `hunt_target_visible_to` + 跑 L0
5. 加 `_test_hunt_profile_matrix` + 全量 L0
6. CHANGELOG + fix-log + handoff `# 回复` + work.log

---

## 风险与回滚

| 风险 | 缓解 |
|------|------|
| tag 与 card_type 语义漂移 | 用现有 card_db 行对照表自测；不增 tag |
| 狼猎狐从死代码变为真行为 | 有意修复；若策划不要，改 `also_allow_tags` 去掉 `mesopredator` 并加断言 `not` |
| 羊无 smallPrey 导致分数字段变化 | 以 L0 兔>鹿断言为准 |

回滚：还原 `world_rules.gd` 三函数即可，无 Registry 耦合。

---

# 回复

**From**: cursor | **Date**: 2026-06-01 | **Status**: DONE

- `world_rules.gd`：`hunt_profile_for` + `_hunt_target_allowed_by_profile`；猎物三函数改 tag/profile 驱动；`mesopredator` 先于 `predator` 解析 profile。
- 狼可猎 `mesopredator` 目标（狐），`hunt_target_score` +2.5 保持低优先。
- `is_active_hunt_threat` 改 `card_has_tag("predator")`（原 `is_adult_wolf` 会把狐当成威胁导致 `hunt_threat_near` 恒真）。
- L0 **756** assertions PASS；`_test_hunt_profile_matrix` 新增。
