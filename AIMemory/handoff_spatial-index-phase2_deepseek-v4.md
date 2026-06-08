# 空间索引 Phase 2：生态入口改桶 + 寻路缓存 + 热路径减负

**From**: deepseek-v4
**To**: cursor
**Date**: 2026-06-06
**Priority**: P0
**Profiler 复测**: _game_tick 805ms/次，索引已上线但主循环仍是全量扫卡

---

## 根因

SpatialIndex 只改了 WorldRules 查询和部分行为文件的找卡方式。但最关键的 `ecosystem_manager.update()` 仍然是 `for card in GameState.cards` 遍历全部 500 张卡。索引的好处被主循环的全扫吞掉了。

---

## 一、P0：生态入口改 tag 桶迭代

### 1.1 ecosystem_manager.gd `update()`

当前：
```
for card in GameState.cards:
    EcosystemTickRegistry.tick_card(self, card, ...)
```

改为：
```
var active = SpatialIndex.query_tag("being") + SpatialIndex.query_tag("autonomous")
for card in active:
    EcosystemTickRegistry.tick_card(self, card, ...)
```

`being` 标签覆盖所有动物，`autonomous` 覆盖桃源人/旅人。排除草皮、石头、树枝、尸体等不需要逐卡 tick 的静态卡。这些静态卡的逻辑由 `environment_manager` 统一 tick——不动。

### 1.2 aquatic_ecology_behavior.gd `tick()`

当前：
```
for card in GameState.cards:
    match card.card_type:
        "waterBug": ...
        "fish": ...
        "shellfish": ...
```

改为：
```
for card in SpatialIndex.query_tag("aquatic"):
    match card.card_type:
        ...
```

不再扫全卡——直接从 `by_tag["aquatic"]` 取。

### 1.3 environment_manager 全扫检查

`environment_manager` 中 `_tick_natural_drops`、`_tick_contained_producers`、`_tick_land_bugs`、`_tick_underground_spread`、`_tick_cliff_weathering` 等已在 Phase 1 改用索引。再次确认是否有遗漏的全卡扫描——有则全部改 `SpatialIndex.query_tag()`。

---

## 二、P0：寻路缓存

### 2.1 问题

Profiler 显示 `Pathfinding.find_path` 169 次 / 18.6s。平均每个 actor 每 tick 都重算一次路径——即使目标和位置没变。

### 2.2 card_base.gd 加缓存字段

```
var cached_path_target_id: int = 0       # 上次寻路的目标卡 instance_id
var cached_path_target_x: int = -1       # 目标坐标
var cached_path_target_y: int = -1
var cached_path_valid_tick: int = 0      # 上次计算的 tick 序号
var cached_path: Array = []              # 坐标数组
```

### 2.3 world_helpers.gd 改寻路调用

`move_one_step_toward` / `move_one_step_near` / `move_one_step_toward_xy` 中，调 `find_path` 前先查缓存：

- 目标卡 instance_id 相同 + 目标坐标不变 → 复用 `cached_path`
- 目标坐标变了 → 重算，更新缓存
- 缓存超过 N tick（如 5）→ 强制重算
- `move_to_cell` 走到一步后从路径中移除该步

### 2.4 寻路节流

同一 actor 每 tick 最多算 1 次 `find_path`。如果当前 tick 已算过（被不同行为重复调用），直接复用。

### 2.5 pathfinding.gd `_sync_obstacles`

当前 `find_path` 每调一次就 `_sync_obstacles` 全扫卡建阻挡表（18.5s/169 次）。改为：SpatialIndex 维护一个 `blocked_cells` 集合（卡占用的格子），`_sync_obstacles` 直接读这个集合而不扫全卡。

在 `SpatialIndex` 中加：

```
var blocked_cells: Dictionary = {}  # Vector2i → bool

func _add_to_cell(card):
    ...
    blocked_cells[Vector2i(card.grid_x, card.grid_y)] = true

func _remove_from_cell(card, x, y):
    ...
    # 该格如果没有其他卡，移除 blocked
    var cell_data = by_cell.get(Vector2i(x, y))
    if cell_data == null or cell_data.is_empty():
        blocked_cells.erase(Vector2i(x, y))
```

`pathfinding._sync_obstacles` 改为从 `SpatialIndex.blocked_cells` 读取。

---

## 三、P2：索引热路径减负

### 3.1 `card_has_tag` / `type_has_tag`

Profiler 显示这两个函数共 1726 次调用 / 23.3s。大部分是同一张卡的不同 tag 被反复检查。

在 `CardBase` 上加一个 `_tag_set: Dictionary = {}`（String → bool），`_ready` 或 `set_card_def` 时从 `card_def.tags` 一次性构建。`card_has_tag` 改为 `return card._tag_set.has(tag)`。

### 3.2 `SpatialIndex.query_near` 大半径优化

当前 `query_near` 遍历矩形区域内所有格。对于 `wildPrey` 的 `WILDPREY_FEAR_RANGE = 5`，矩形 11×11 = 121 格。每次威胁检查都走一遍。

优化：半径 ≤ 2 用展开的 25 格直接读取（不建循环），半径 > 2 时才走矩形遍历。或者 `query_near` 内部缓存"距离环"——每个半径预先计算相对坐标列表。

### 3.3 `card_has_capability` 同 `card_has_tag`

`CARD_CAPABILITIES` 查询也改成从 card 上的预建集合读取。`CardBase` 加 `_cap_set: Dictionary = {}`。

---

## 四、涉及文件

| 文件 | 改动 |
|------|------|
| `scripts/world/ecosystem_manager.gd` | `update()` 入口改 tag 桶迭代 |
| `scripts/world/behaviors/aquatic_ecology_behavior.gd` | tick 入口改 `query_tag("aquatic")` |
| `scripts/world/environment_manager.gd` | 确认并消除剩余全卡扫描 |
| `scripts/cards/card_base.gd` | 寻路缓存字段 + `_tag_set` + `_cap_set` 预建 |
| `scripts/core/world_helpers.gd` | 寻路调用前查缓存 + 节流 |
| `scripts/core/pathfinding.gd` | `_sync_obstacles` 改从 `SpatialIndex.blocked_cells` 读 |
| `scripts/core/spatial_index.gd` | 加 `blocked_cells` 维护 |
| `scripts/core/world_rules.gd` | `card_has_tag` / `type_has_tag` / `card_has_capability` 改集合读取 |
| `scripts/world/ecosystem_tick_registry.gd` | 无改动（入口已改，分发逻辑不变） |

## 五、验收标准

- L0 993 断言不降
- Profiler 复测：`_game_tick` 目标 < 200ms（首次冲刺，不设 100ms）
- `find_path` 调用次数 < 60 次/10s
- `card_has_tag` / `type_has_tag` 累计 < 3s/10s
- F5 进游戏：生态行为与改前一致

## 约束

- 不动标签/能力定义
- 不动行为逻辑
- 不动 UI / 渲染
- 记 fix-log
