# 标签本质性审计 — 执行 handoff

**From**: deepseek-v4
**To**: cursor
**Date**: 2026-06-03

---

## 一、狐狸猎杀上限

**问题**：狐狸一天杀 13 只兔子。`mark_ecology_fed` 只标记"吃过了"，不阻止继续猎。

**改为**：

1. `world_rules.gd` 新增常量 `MESOPREDATOR_MAX_KILLS_PER_DAY := 2`
2. `tick_capability_hunt` 或 `execute_capability_hunt_attack` 中，mesopredator 类型检查当日猎杀数，达到上限后跳过捕猎（改状态为"饱腹"，不进入 hunt 目标选择）
3. 计数值存在卡上：`fox.scavengeToday` 类似机制，新增 `dailyKills` 字段，日重置归零
4. `reset_ecology_feed_flag` 中一并重置

---

## 二、HP 标签化

**问题**：羊 HP=2、鹿 HP=3——同为 largePrey + herbivore，HP 差异无标签依据。

**改为**：

1. 新增三个标签：
   - `body.tiny` → HP=1（田鼠、兔、所有幼体）
   - `body.small` → HP=2（羊、水牛犊）
   - `body.medium` → HP=3（鹿、狐）
   - `body.large` → HP=4（狼、水牛）
   - 玩家 HP=6 保留硬编码（actor 特殊）

2. `card_db.gd` 给每张动物卡补 body.* 标签
3. HP 值保持现有数值不变——只是现在有了标签依据

---

## 三、孤立标签清理

以下标签删除（无逻辑引用，纯装饰）：
- `plant.patch`、`berry.source`、`microfauna.host`、`regenerates`、`natural`（灌木相关，走函数判断）
- `consumable`、`food`、`food.raw`、`raw`（食物，不走 tag）
- `soil`（humus，`fertile` 已够）
- `toolHead`、`craftPart`、`novelty`（死标签）
- `stoneSource`、`woodSource`、`mushroomSource`（走 capability）
- `wet`（湿木头）
- `organic`（尸体走 is_corpse）
- `naturalDrop`（树枝装饰）

以下标签保留但暂不补逻辑（后续需求）：
- `burrower`、`wildPrey`、`omnivore.small`、`scavenger`

`selection_info_panel.gd` 的 `TAG_ZH` 和 `_SKIP_TAGS` 同步清理。

---

## 四、关键常量标签化

在 `game_state.gd` 中不改数值，只补注释——每个常量注明"依据标签"：

```gdscript
# 依据：predator + body.large → 日食量 2
const WOLF_MEAT_PER_DAY: int = 2

# 依据：mesopredator + body.medium → 日食量 2（同狼但猎物更小）
# 当前无对应常量，由 MESOPREDATOR_MAX_KILLS_PER_DAY 间接约束

# 依据：smallPrey + body.tiny → 种群上限 8
const FIELD_MOUSE_POP_CAP: int = 8

# 依据：largePrey + herbivore → 种群上限 5
const DEER_POP_CAP: int = 5

# 依据：mesopredator + body.medium → 窝数上限与家庭成员
const FOX_DEN_CAP: int = 3
const FOX_DEN_FAMILY_CAPACITY: int = 3
```

只改注释，不改代码逻辑。

---

## 涉及文件

| 文件 | 改什么 |
|------|--------|
| `scripts/core/world_rules.gd` | MESOPREDATOR_MAX_KILLS_PER_DAY + tick_capability_hunt 上限检查 |
| `scripts/cards/card_db.gd` | body.* 标签补全 + 孤立标签删除 |
| `scripts/ui/selection_info_panel.gd` | TAG_ZH / _SKIP_TAGS 同步清理 |
| `scripts/core/game_state.gd` | 常量加标签依据注释 |

## 约束

- L0 断言数不降
- 不碰管线顺序 / ecosystem_behavior_key / 封闭模式
- 记 fix-log
