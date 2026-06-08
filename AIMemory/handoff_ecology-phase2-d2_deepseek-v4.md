# 生态二期 D-后半：岩壁风化 + 尸体引虫 + 野山药

**From**: deepseek-v4
**To**: cursor
**Date**: 2026-06-06
**Priority**: HIGH
**设计依据**: `docs/design/diversity-overflow-design.md`

---

D4+D5+D3 合并，互不依赖。

---

## D4：岩壁风化

### 4.1 地形

`scripts/core/world_rules/world_rules_terrain.gd`：
- 山 `mountain` 卡邻格 → 标记为 `cliff` 格型
- 或在地图边缘高海拔处设 8-12 格 cliff
- `is_cliff_cell(x, y)` 判定函数

### 4.2 风化 tick

`scripts/world/environment_manager.gd`，加 `_tick_cliff_weathering`：

- 对每个 cliff 格，维护风化计时器（meta key `cliff_weather_%d_%d`）
- 周期 `CLIFF_WEATHER_INTERVAL`（60s）
- 到 -> 取该格 4 邻格中随机空位 -> spawn `shard`，`state = "风化掉落"`
- `game_state.gd`：`const CLIFF_WEATHER_INTERVAL: float = 60.0`

### 4.3 可选：主动敲

`interaction_manager.gd`：锤子敲 cliff 格 → 出 `shard`×N。已有 `hammer → stone → shard` 冲击配方，但针对的是 mountain 卡。把 cliff 格也挂上。

---

## D5：尸体引虫

### 5.1 新卡

`card_db.gd`：

```
_reg("landBug", "虫", "虫", "entity", ["tiny","volant","smallPrey","scavenger"], 1, false, false)
```

水虫叫 `waterBug`，陆虫叫 `landBug`，两套不混。

### 5.2 CARD_CAPABILITIES

```
"landBug": ["capability.move","capability.be_hunted"],
```

### 5.3 吸引行为

`scripts/world/environment_manager.gd`，加 `_tick_land_bugs(real_delta)`：

- 遍历所有 `corpse` 卡
- 尸体周围 N 格（`BUG_CARCASS_ATTRACT_RANGE = 4`）没有 `landBug` → 在空位 spawn 一只
- 同尸体最多吸引 1 只虫
- 全局 `landBug` 上限 8 只
- `landBug` 到达尸体格 → 加速分解（腐解倒计时 ×2 速度）
- 无尸体时，虫随机游荡

### 5.4 食物链

- `landBug` 有 `smallPrey` 标签 → 兔/鼠/雉鸡/竹鼠可觅食
- `field_mouse_behavior`：田鼠觅食目标列表加 `landBug`
- `herbivore_grazer_behavior`：`_tick_rabbit` / 雉鸡觅食列表加 `landBug`

### 5.5 常量

`game_state.gd`：

```
const BUG_CARCASS_ATTRACT_RANGE: int = 4
const LAND_BUG_POP_CAP: int = 8
```

---

## D3：野山药

### 3.1 新卡

`card_db.gd`：

```
_reg("wildYam", "药", "野山药", "entity", ["tuber","food.edible","underground","organize.locked"], 0, true)
```

`tuber` 标签 = 地下块茎，`underground` = 藏于地下。

### 3.2 card_base.gd — `in_ground`

照搬 `in_tree` 模式：

```
var in_ground: bool = false

func set_in_ground(value: bool) -> void:
    in_ground = value
    visible = not value
```

### 3.3 world_rules.gd

```
static func is_underground_card(card: CardBase) -> bool:
    return is_instance_valid(card) and card.in_ground

static func underground_occupants_at(gx: int, gy: int) -> Array:
    # 返回该格所有 in_ground 的卡

static func finalize_underground_spawn(card: CardBase) -> void:
    if not is_instance_valid(card):
        return
    card.set_in_ground(true)
```

### 3.4 林下容纳 UI

`world_rules_ui.gd`：
- `_underground_containment_entries(cell_x, cell_y)` — 对格内 `in_ground` 卡建条目
- `ui_containment_entries` 加分支：格子有地下卡时显示

`game_ui.gd`：选中地下有内容的格子时，渲染地下容纳（同水潭模式）。

`selection_info_panel.gd`：格子信息加「地下：野山药」行。

### 3.5 CARD_CAPABILITIES

```
"wildYam": ["capability.be_collected"],
```

### 3.6 采收

`interaction_manager.gd`：人空手或持锄走到有 `wildYam` 的格 → 挖山药：

- 人需在该格（不是邻格，是站上去）
- 消费 `wildYam` → 产出 `wildYamRoot`（山药食物卡）
- 或直接把 `wildYam` 转成可携带食物（用已有卡），Cursor 判断

如果有专用产出卡：

```
_reg("wildYamRoot", "薯", "山药", "entity", ["food.edible","camp.storable","perishable"])
```

`perishable` 已有行为——挖出来的山药和生肉一样会腐坏。

### 3.7 蔓延

`scripts/world/environment_manager.gd`，加 `_tick_underground_spread`：

- 遍历所有 `in_ground` 的 `wildYam`
- 周期 `YAM_SPREAD_INTERVAL`（60s）
- 取邻格（4 向）随机空位 → spawn 新 `wildYam`，`in_ground = true`
- 全局 `wildYam` 上限 12

`game_state.gd`：

```
const YAM_SPREAD_INTERVAL: float = 60.0
const WILDYAM_POP_CAP: int = 12
```

### 3.8 竹鼠联动

`field_mouse_behavior.gd`：竹鼠觅食目标加同格 `in_ground` 的 `wildYam`。竹鼠站上去 → `state = "拱山药"` → 消费 `wildYam` → `mark_ecology_fed`。

### 3.9 初始生成

`world_manager.gd`：选 3-4 个林下格（近树林、非水格），spawn `wildYam` 并 `set_in_ground(true)`。

---

## 涉及文件

| 文件 | 改动 |
|------|------|
| `scripts/cards/card_db.gd` | landBug / wildYam / wildYamRoot `_reg` |
| `scripts/cards/card_base.gd` | `in_ground` + `set_in_ground()` |
| `autoload/game_state.gd` | CLIFF_WEATHER_INTERVAL / BUG_CARCASS_ATTRACT_RANGE / LAND_BUG_POP_CAP / YAM_SPREAD_INTERVAL / WILDYAM_POP_CAP |
| `scripts/core/world_rules.gd` | CARD_CAPABILITIES + `is_cliff_cell` + `is_underground_card` + `underground_occupants_at` + `finalize_underground_spawn` |
| `scripts/core/world_rules/world_rules_terrain.gd` | cliff 格型 |
| `scripts/core/world_rules/world_rules_ui.gd` | `_underground_containment_entries` + `ui_containment_entries` 加地下分支 |
| `scripts/world/environment_manager.gd` | `_tick_cliff_weathering` + `_tick_land_bugs` + `_tick_underground_spread` |
| `scripts/core/interaction_manager.gd` | 挖山药交互 |
| `scripts/world/world_manager.gd` | wildYam 初始生成 |
| `scripts/world/behaviors/field_mouse_behavior.gd` | 竹鼠觅食加 wildYam |
| `scripts/world/behaviors/herbivore_grazer_behavior.gd` | 兔/雉鸡觅食加 landBug |
| `scripts/ui/game_ui.gd` | 地下容纳渲染 |
| `scripts/ui/selection_info_panel.gd` | 新卡/标签中文化 |

## 约束

- L0 断言数不降
- 不新增 behavior 文件
- 不碰管线/封闭模式
- `in_ground` 照搬 `in_tree` 实现，不另起炉灶
- 记 fix-log
