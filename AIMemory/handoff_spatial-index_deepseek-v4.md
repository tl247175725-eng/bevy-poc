# 架构改造：标签索引空间网格

**From**: deepseek-v4
**To**: cursor
**Date**: 2026-06-06
**Priority**: P0（架构地基，先于所有新功能）
**Profiler 数据**: ecosystem_manager 537ms/tick，find_path 19s 累计

---

## 问题

262 处 `for card in GameState.cards` 全量扫描。每 tick 对 500 张卡做 100+ 次全扫 ≈ 5-10 万次迭代。Profiler 实测：_game_tick 642ms/次，83% 在生态 tick。

## 解

标签索引空间网格。标签 = 索引键，空间 = 索引维度。卡增删移时增量更新索引，查询时 O(1) 读取。

**吻合设计铁律**：标签从 `_reg` 那一刻就是索引键——不需要新分类。WorldRules 仍是唯一中介——索引是它的内部数据结构。

---

## 一、新建 SpatialIndex

`scripts/core/spatial_index.gd`：

```
class_name SpatialIndex
extends RefCounted

# 全局：tag → [有该标签的卡]
var by_tag: Dictionary = {}

# 空间：Vector2i → {tag → [该格具有该标签的卡]}
var by_cell: Dictionary = {}
```

### API

| 方法 | 用途 |
|------|------|
| `on_spawn(card)` | 卡生成时，按 card.card_def.tags 注册到 by_tag 和 by_cell |
| `on_remove(card)` | 卡移除时，从两个索引中擦除 |
| `on_move(card, from_x, from_y)` | 卡移动时，更新 by_cell（by_tag 不变） |
| `query_tag(tag)` → Array | 全局：所有带此标签的卡 |
| `query_at(x, y, tag)` → Array | 空间：该格带此标签的卡 |
| `query_near(x, y, tag, radius)` → Array | 空间：范围内带此标签的卡 |
| `count_tag(tag)` → int | 计数 |
| `rebuild()` | 遍历 GameState.cards 全量重建索引（仅开局调用一次） |

### 实现细节

- `on_spawn`：遍历 `card.card_def.tags`，每个 tag 作为键，card 追加到 `by_tag[tag]`。同时调用内部 `_add_to_cell(card)`
- `_add_to_cell`：读 `card.grid_x, card.grid_y` 得到 `Vector2i` 键。取 `by_cell[key]`（不存在则建空字典）。遍历 `card.card_def.tags`，card 追加到 `by_cell[key][tag]`
- `_remove_from_cell`：同上但 erase
- `query_near`：遍历 `(x ± radius, y ± radius)` 矩形内所有格，聚合每个格的 `tag` 数组返回
- `rebuild`：先清空两个字典，遍历所有 `GameState.cards`，对每张有效卡调 `on_spawn`

---

## 二、接入 GameState 卡生命周期

### 2.1 autoload/game_state.gd

在文件顶部加：

```
const SpatialIndexClass = preload("res://scripts/core/spatial_index.gd")
static var spatial_index: SpatialIndex = null
```

在 `_ready()` 或初始化处：

```
spatial_index = SpatialIndexClass.new()
spatial_index.rebuild()
```

### 2.2 scripts/core/interaction_manager.gd

- `spawn_card_at`：spawn 成功后调 `GameState.spatial_index.on_spawn(card)`
- `remove_card`：remove 前调 `GameState.spatial_index.on_remove(card)`

### 2.3 scripts/core/world_helpers.gd

- `move_to_cell(card, x, y)`：移动前记录 `(card.grid_x, card.grid_y)`，移动后调 `GameState.spatial_index.on_move(card, old_x, old_y)`
- `move_one_step_toward`、`move_one_step_near`、`move_one_step_toward_xy`：同上一一嵌钩子

---

## 三、迁移 WorldRules 查询函数

以下函数从"遍历全卡"改为读索引。**不改函数签名**——调用方不受影响。

| 函数 | 当前 | 改为 |
|------|------|------|
| `living_grasses()` | `for card in cards: if tag=="grass"` | `spatial_index.query_tag("grass")` |
| `grass_covers()` | 同上 + cover 过滤 | `query_tag("grass")` + filter |
| `living_algae()` | 全扫 + tag=="algae" | `query_tag("algae")` |
| `bushes()` | 全扫 + tag=="bush" | `query_tag("bush")` |
| `corpses_in_world()` | 全扫 + tag=="corpse" | `query_tag("corpse")` |
| `land_bugs_in_world()` | 全扫 + type=="landBug" | `query_tag("volant")` + filter type |
| `count_living_grasses()` | 全扫计数 | `spatial_index.count_tag("grass")` |
| `pool_occupants_at(x,y)` | 全扫 + in_pool 过滤 | `query_at(x, y, "aquatic")` |
| `tree_occupants_at(x,y)` | 全扫 + in_tree 过滤 | `query_at(x, y, "nut_producer")` + `query_at(x, y, "cone_producer")` + `query_at(x, y, "nest")` |
| `underground_occupants_at(x,y)` | 全扫 + in_ground | `query_at(x, y, "tuber")` |
| `nearest_wolf(actor, range)` | 全扫 + tag=="predator" + 距离比较 | `query_near(x, y, "predator", range)` + 距离排序 |
| `nearest_threat(actor, range)` | 全扫 | `query_near(x, y, "predator", range)` + `query_near(x, y, "mesopredator", range)` 合并 |
| `count_type(type_name)` | 全扫 | `query_tag(tag)` + filter by type |
| `hunt_threat_near(card, range)` | 全扫 | `query_near(x, y, "predator", range)` |
| `pack_hunter_near(card, range)` | 全扫 | `query_near(x, y, "pack_hunter", range)` |
| `nearest_pack_hunter(card, range)` | 全扫 | 同上 + 距离排序 |

**迁移优先级**：先改 `living_grasses`、`bushes`、`corpses_in_world`、`nearest_wolf`——这 4 个被调频次最高。

---

## 四、迁移行为文件中的直接全扫

这些文件中还有独立的全卡扫描循环，改掉。

### 4.1 herbivore_grazer_behavior.gd

- `_tick_rabbit` L87 全扫找 forage → 改用 `SpatialIndex.query_tag("foodSource")` + `query_tag("food.edible")` 合并
- `_tick_deer` L164 全扫找草 → `SpatialIndex.query_tag("grass")`
- `_tick_sheep` L238 同 → `host.all_grass()` 改为 `SpatialIndex.query_tag("grass")`
- `_tick_slow_grazer` L293 同

### 4.2 field_mouse_behavior.gd

- 田鼠找虫 → `SpatialIndex.query_near(x, y, "berry.source", 3)` 或 `query_tag("landBug")`
- 竹鼠找树周草 → `SpatialIndex.query_near(x, y, "grass", 2)` + filter near tree

### 4.3 ecosystem_manager.gd

- `nearest_wolf` 已通过 WorldRules 间接改
- `find_meat_source` → `SpatialIndex.query_near(x, y, "camp.storable", range)` + filter by meat type
- `all_grass()` → `SpatialIndex.query_tag("grass")`

### 4.4 environment_manager.gd

- `_tick_natural_drops` L173 全扫找树 → `SpatialIndex.query_tag("source.lumber")`
- `_tick_contained_producers` → 已通过 WorldRules 间接改
- `_tick_land_bugs` → `SpatialIndex.query_tag("corpse")` + `SpatialIndex.query_tag("volant")`
- `_tick_bush_ecology` → `SpatialIndex.query_tag("bush")`

### 4.5 aquatic_ecology_behavior.gd

- `_count_type` → `SpatialIndex.count_tag("aquatic")` + filter
- `_nearest_algae` → `SpatialIndex.query_near(x, y, "primary_producer", radius)`
- `_nearest_type` → `SpatialIndex.query_near(x, y, "aquatic", radius)` + filter type

### 4.6 population_manager.gd

- 各 lifecycle 函数中的计数逻辑 → 改用 `SpatialIndex.count_tag()`

---

## 五、不动的

- L0 断言逻辑（测试独立建环境，不依赖索引）
- `GameState.cards` 数组本身（保留作为主数据源，索引是镜像）
- 卡定义、标签、能力——零改动
- 渲染/UI 层——零改动
- 封闭模式/管线——零改动
- 地形系统——零改动

---

## 六、涉及文件

| 文件 | 改动 |
|------|------|
| `scripts/core/spatial_index.gd` | **新建** |
| `autoload/game_state.gd` | 加 spatial_index 实例 + 初始化 |
| `scripts/core/interaction_manager.gd` | spawn/remove 嵌钩子 |
| `scripts/core/world_helpers.gd` | move 四个函数嵌钩子 |
| `scripts/core/world_rules.gd` | ~20 个查询函数改索引读取 |
| `scripts/world/behaviors/herbivore_grazer_behavior.gd` | 4 处全扫改索引 |
| `scripts/world/behaviors/field_mouse_behavior.gd` | 2 处全扫改索引 |
| `scripts/world/ecosystem_manager.gd` | 3 处全扫改索引 |
| `scripts/world/environment_manager.gd` | 4 处全扫改索引 |
| `scripts/world/behaviors/aquatic_ecology_behavior.gd` | 3 处全扫改索引 |
| `scripts/world/population_manager.gd` | 计数逻辑改索引 |

## 七、验收标准

- L0 全量 PASS（断言数不降）
- Profiler 复测：_game_tick 从 642ms → 目标 < 100ms
- find_path 调用次数不因索引改造而增加
- F5 进游戏：生态行为与改前一致（羊吃草、狼猎鹿、狐清腐均正常）

## 八、约束

- 不新增卡
- 不动任何标签定义
- 不动任何行为逻辑（只改"怎么找到相关卡"）
- 不动 UI / 渲染
- 记 fix-log
