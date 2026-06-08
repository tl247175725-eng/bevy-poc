# 调度改造 Step 2：生态饱食字段统一

**From**: cursor  
**To**: cursor（自执行）  
**Date**: 2026-06-01  
**Priority**: HIGH  
**Parent**: `proposition_dispatch-capability_deepseek-v4.md`（6 步表 Step 2）  
**前置**: Step1 猎物 profile 化 DONE（L0 756）

---

## 目标

`mark_ecology_fed` / `reset_ecology_feed_flag` / `ecology_was_fed_today` / `is_ecology_starvation_subject` **全部按 capability + 已有 tag 判定**，去掉 `card_type`/`match` 硬编码。

**统一运行时字段名**（卡牌脚本与 spawn 一致）：

| 字段 | 含义 | 谁用 |
|------|------|------|
| `fedToday` | 今日是否进食（bool） | 草食、幼体、狐等 |
| `starveDays` | 连续断粮天数 | **全部**（含狼成体、狐成体；废除 `daysWithoutMeat`） |
| `meatFedToday` | 今日肉量计数（int） | 仅 `capability.hunt` + `predator`（狼群配额） |

新增 `WorldRules` 读写助手，外部禁止再按 `card_type` 分支访问饱食字段。

---

## 不碰

- 猎物 profile（Step1）
- `ecosystem_behavior_key` / Registry（Step5）
- 狼窝/狐窝宿主 API（Step4）
- `game_ui` **本步要同步**狼/狐断粮行（只改字段名 `starveDays`，不加新 UI 分支）

---

## 现状断点

`world_rules.gd` L1618-1659：`match card.card_type` 三分（狼/狐幼/默认）。

`population_manager._on_ecology_feed_new_day`：`is_adult_fox` / `is_fox_cub` + 直接 `daysWithoutMeat`。

`session_diagnostics._on_card_removed`：按 `card_type` 判饿死。

狼成体用 `daysWithoutMeat`，狐成体用 `daysWithoutMeat`，草食用 `starveDays` —— 同名不同义。

---

## 能力判定（取代 card_type 列表）

```gdscript
# 是否纳入生态日馈/饿死管线
is_ecology_starvation_subject(card):
  - 无效 → false
  - tag actor（玩家）→ false
  - 非 being+animal → false
  - can_forage OR can_hunt OR (can_grow + be_cared_for) → true

# 是否用「今日肉量」配额（狼）
ecology_uses_meat_quota(card):
  can_hunt(card) and card_has_tag(card, "predator")

# 是否幼体（走 fedToday + starveDays，无肉量配额）
ecology_is_feed_juvenile(card):
  can_grow(card) and be_cared_for and not can_hunt(card)
```

**饿死宽限** `ecology_starve_grace_days(card)`：

| 条件 | 常量 |
|------|------|
| `predator` + `can_hunt` | `WOLF_ADULT_STARVE_GRACE_DAYS`（成狼在 `_on_wolf_new_day` 单独处理，此处兜底） |
| `mesopredator` + `can_hunt` | `FOX_ADULT_STARVE_GRACE_DAYS` |
| `ecology_is_feed_juvenile` + `predator` tag | `WOLF_CUB_STARVE_DAYS` |
| `ecology_is_feed_juvenile` + `carnivore` tag | `FOX_CUB_STARVE_DAYS` |
| 其它草食/幼兽 | `ECOLOGY_STARVE_GRACE_DAYS` |

---

## WorldRules API（新增/改写）

| 函数 | 行为 |
|------|------|
| `ecology_starve_days(card)` / `ecology_set_starve_days(card, n)` | 统一读写的 `starveDays` |
| `ecology_meat_portions_today(card)` | `meatFedToday`，非配额卡返回 0 |
| `mark_ecology_fed(card)` | 配额卡：`meatFedToday++` + `starveDays=0`；其它：`fedToday=true` + `starveDays=0` |
| `ecology_was_fed_today(card)` | 配额：`meatFedToday>0`；其它：`fedToday` |
| `reset_ecology_feed_flag(card)` | 配额：`meatFedToday=0`；其它：`fedToday=false` |
| `is_ecology_starvation_subject` | 见上，无 card_type 表 |

---

## 文件清单

| 文件 | 操作 |
|------|------|
| `world_rules.gd` | 主改：饱食 API + `is_ecology_starvation_subject` |
| `wolf_card.gd` | 删除 `daysWithoutMeat`，成狼断粮用 `starveDays` |
| `fox_card.gd` | `daysWithoutMeat` → `starveDays` |
| `population_manager.gd` | 日馈循环 + `_on_wolf_new_day` 走 WorldRules 助手 |
| `ecosystem_manager.gd` | 狼进食写 `starveDays` |
| `wolf_behavior.gd` | `daysWithoutMeat` → `starveDays` / 助手 |
| `session_diagnostics.gd` | 饿死判定走助手 |
| `game_ui.gd` | 狐/狼断粮显示 `starveDays` |
| `world_manager.gd` | 狐 spawn 字段名 |
| `unit_test_cases.gd` | `_test_ecology_feed_capabilities` |

---

## 执行顺序

1. `world_rules.gd` 助手 + 四函数改写  
2. `wolf_card` / `fox_card` 字段统一  
3. `population_manager` + `ecosystem_manager` + `wolf_behavior`  
4. `session_diagnostics` + `game_ui` + `world_manager` spawn  
5. L0 + 新测 + CHANGELOG/fix-log/handoff 回复

---

## 验收

- L0 全绿（断言数 ≥756）
- 狼：`meatFedToday` 配额逻辑不变；`starveDays` 替代 `daysWithoutMeat`
- 狐/羊/兔/鼠：`mark_ecology_fed` 后 `ecology_was_fed_today` 为真
- 新卡仅需 `forage`/`hunt`/`be_cared_for` 能力即可进入饿死管线，无需改 WorldRules match

---

# 回复

**From**: cursor | **Date**: 2026-06-01 | **Status**: DONE

- `WorldRules`：`is_ecology_starvation_subject` / `mark_ecology_fed` / `reset_ecology_feed_flag` / `ecology_was_fed_today` 改 capability+tag；新增 `ecology_*` 助手。
- 废除 `daysWithoutMeat`；狼/狐/草食统一 `starveDays`；`CardBase` 声明 `fedToday`/`starveDays`/`meatFedToday`。
- `population_manager` 日馈单循环；`game_ui`/`session_diagnostics` 同步。
- L0 **772** assertions PASS；`_test_ecology_feed_capabilities` 新增。
