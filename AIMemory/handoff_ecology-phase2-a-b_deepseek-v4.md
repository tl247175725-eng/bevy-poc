# 生态二期 A+B：标签补齐 + 雉鸡竹鼠

**From**: deepseek-v4
**To**: cursor
**Date**: 2026-06-06
**Priority**: HIGH

---

## A1：aquatic 水格移动约束

**文件**：`scripts/world/behaviors/aquatic_ecology_behavior.gd`

`_tick_water_bug_card` 和 `_tick_fish_card` 中的 `random_step` / `move_one_step_toward` 替换为水格限定版：

- 在文件内加两个 static helper：`_random_pool_step(card)` 和 `_move_toward_in_pool(card, target)`
- 目标格必须 `_TerrainRules.is_pool_cell(x, y)`，否则取不移动
- 水虫/鱼不在水格时：`state = "搁浅"`，`goal = "等水"`，skip 移动

不碰藻和贝。

---

## A2：wildPrey 逃跑半径

**文件 1**：`autoload/game_state.gd`，在生态常量区加：

```
const WILDPREY_FEAR_RANGE: int = 5
```

**文件 2**：`scripts/world/behaviors/herbivore_grazer_behavior.gd`，`_tick_deer` 中：

- 第 155 行 `host.nearest_wolf(deer, GameState.SHEEP_FEAR_RANGE)` → 改用 `GameState.WILDPREY_FEAR_RANGE`
- 第 145 行 `near_camp_fire(deer, GameState.FIRE_FEAR_RANGE)` → 改 `FIRE_FEAR_RANGE + 1`

不碰 `_tick_sheep`。

---

## B1：雉鸡 pheasant

### card_db.gd

```
_reg("pheasant", "雉", "雉鸡", "entity", ["being","animal","smallPrey","flocking","omnivore.small","body.small"], 1, false, false)
_reg("pheasantChick", "雏", "雉鸡雏", "entity", ["being","animal","smallPrey","juvenile","body.tiny"], 1, false, false)
```

### world_rules.gd — CARD_CAPABILITIES

```
"pheasant": ["capability.move","capability.forage","capability.escape_small","capability.flee","capability.reproduce","capability.be_hunted","capability.care_child"],
"pheasantChick": ["capability.move","capability.escape_small","capability.follow","capability.grow","capability.be_cared_for","capability.be_hunted"],
```

幼体照搬 fieldMousePup 的能力模式，用 `capability.grow` + `capability.be_cared_for`。

### world_rules.gd — herbivore_grazer_profile

雉鸡 `capability.escape_small` → 已被 `herbivore_grazer_profile` 命中 `"rabbit"` 分支。走 `_tick_rabbit`。

`_tick_rabbit` 里有两处 card_type 硬编码要泛化：

1. `rabbit_forage_priority` — 雉鸡和兔觅食优先级一致（灌丛 > 草）
2. `_rabbit_may_eat_grass` — 雉鸡可以吃草
3. `_rabbit_record_grass_bite` — 雉鸡同兔，咬草记录

这三处把 `card_type == "rabbit"` 改成 `card_has_tag(card, "omnivore.small")` 或新增一个 `_is_small_forager(card)` 判定。

雉鸡不进入 `hiddenInGrass`（兔专属），在 `_tick_rabbit` 入口加守卫：`if card_type == "rabbit"` 才设隐藏。

### world_rules.gd — ecosystem_behavior_key

雉鸡能力签名包含 `forage + flee + be_hunted` → 匹配 `_SIG_HERBIVORE_GRAZER`，无需改。

### population_manager.gd — 繁殖

在 `_update_sheep_lifecycle` 同级的繁殖循环里加 `_update_pheasant_lifecycle`：

- 公母配对
- `flocking_blocks_reproduction(adults)` 门槛（已有，复用）
- 每胎 1-2 只 `pheasantChick`
- 繁殖周期：`POPULATION_REPRO_CYCLE_SECONDS`（同羊）

### world_manager.gd — 初始生成

在 `_spawn_initial_cards` 里，`_spawn_fox_family_near` 之前加：

```
_spawn("pheasant", px - 13, py + 2, {"state": "找食", "sex": "公", "fedToday": true, "starveDays": 0})
_spawn("pheasant", px - 12, py + 2, {"state": "找食", "sex": "公", "fedToday": true, "starveDays": 0})
_spawn("pheasant", px - 11, py + 2, {"state": "找食", "sex": "母", "fedToday": true, "starveDays": 0})
_spawn("pheasant", px - 10, py + 2, {"state": "找食", "sex": "母", "fedToday": true, "starveDays": 0})
_spawn("pheasant", px - 9, py + 2, {"state": "找食", "sex": "母", "fedToday": true, "starveDays": 0})
```

---

## B2：竹鼠 bambooRat

### card_db.gd

```
_reg("bambooRat", "竹", "竹鼠", "entity", ["being","animal","smallPrey","burrower","omnivore.small","body.tiny"], 1, false, false)
```

无独立幼体卡——和兔一样，幼体直接用 `bambooRat` spawn，`state = "新生"`。

### world_rules.gd — CARD_CAPABILITIES

```
"bambooRat": ["capability.move","capability.forage","capability.use_cover","capability.escape_cover","capability.flee","capability.reproduce","capability.be_hunted"],
```

同 fieldMouse 的能力签名 → `ecosystem_behavior_key` 自动命中 `BEHAVIOR_COVER_FORAGER`。

### field_mouse_behavior.gd — 泛化

`FieldMouseBehavior` 里 `is_adult_field_mouse` 的 card_type 检查：改成 `card_has_tag(card, "burrower") && !card_has_tag(card, "juvenile")`。

田鼠的灌丛依赖逻辑（`_nearest_food_cover`、`enter_cover` 等）在 `card_type == "fieldMouse"` 守卫下保留。竹鼠不触发灌丛逻辑，走林地路径：

- 竹鼠觅食目标：`nearest_tree(card)` 附近的草皮（树周有虫/根）
- 掘穴：`enter_burrow` 已有，直接复用

### population_manager.gd — 繁殖

在 `_update_field_mouse_lifecycle` 同级加 `_update_bamboo_rat_lifecycle`：

- 公母配对，同田鼠
- 每胎 2-3 只
- 繁殖周期：`POPULATION_REPRO_CYCLE_SECONDS`

### world_manager.gd — 初始生成

```
_spawn("bambooRat", px + 5, py + 4, {"state": "林下觅食", "sex": "公", "fedToday": true, "starveDays": 0})
_spawn("bambooRat", px + 6, py + 4, {"state": "林下觅食", "sex": "公", "fedToday": true, "starveDays": 0})
_spawn("bambooRat", px + 5, py + 5, {"state": "林下觅食", "sex": "母", "fedToday": true, "starveDays": 0})
_spawn("bambooRat", px + 6, py + 5, {"state": "林下觅食", "sex": "母", "fedToday": true, "starveDays": 0})
```

位置靠近树林（`tree` 卡）。

---

## 全部涉及文件

| 文件 | 改了什么 |
|------|---------|
| `autoload/game_state.gd` | A2: WILDPREY_FEAR_RANGE |
| `scripts/world/behaviors/aquatic_ecology_behavior.gd` | A1: 水格约束 helper + 替换移动调用 |
| `scripts/world/behaviors/herbivore_grazer_behavior.gd` | A2: deer 探测距离 + B1: _tick_rabbit 泛化 |
| `scripts/cards/card_db.gd` | B1/B2: 三条 _reg |
| `scripts/core/world_rules.gd` | B1/B2: CARD_CAPABILITIES 三条 |
| `scripts/world/world_manager.gd` | B1/B2: 初始 spawn |
| `scripts/world/population_manager.gd` | B1: 雉鸡繁殖 + B2: 竹鼠繁殖 |
| `scripts/world/behaviors/field_mouse_behavior.gd` | B2: 泛化 card_type → tag + 竹鼠觅食 |

## 约束

- L0 断言数不降
- 不新增 behavior 文件
- 不新增系统
- 不碰管线/封闭模式/containment/natural_drop
- 记 fix-log
