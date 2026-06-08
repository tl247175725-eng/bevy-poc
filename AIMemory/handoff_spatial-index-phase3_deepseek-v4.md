# 空间索引 Phase 3：域查询缓存 + type 级 tag 预建 + get_card_at 改索引

**From**: deepseek-v4
**To**: cursor
**Date**: 2026-06-06
**Priority**: P0
**Profiler 复测**: Phase 2 后 _game_tick 717ms/次。寻路已优化（18.6s→7.6s）。新瓶颈：域查询链 ~170s 累计 + type_has_tag ~55s。

---

## 根因

`best_feed_source_for` / `best_hunt_target` 调用链：
```
best_feed_source_for(actor)
  → is_feed_source → in_camp_home_territory
    → camp_domain_anchor()          ← 每调一次就重新找篝火（490 次）
      → defines_domain → type_domains → type_has_tag  ← 每次查 CardDB
```

篝火位置一帧内不变。`type_has_tag` 的答案从 CardDB init 那一刻就确定了。两件事都在被反复重算。

---

## 一、P0：`type_has_tag` 静态预建（最大单点收益）

### card_db.gd `init()`

在 `_reg` 循环之后加：

```
static var type_tag_sets: Dictionary = {}  # type_name → {tag: true}

# init() 末尾：
for type_name in defs:
    var d = defs[type_name]
    var tag_set = {}
    for tag in d.tags:
        tag_set[tag] = true
    type_tag_sets[type_name] = tag_set
```

### world_rules.gd `type_has_tag`

当前：遍历 `CARD_CAPABILITIES` 或读 CardDB
改为：
```
static func type_has_tag(type_name: String, tag: String) -> bool:
    var ts = CardDB.type_tag_sets.get(type_name, {})
    return ts.get(tag, false)
```

含前缀匹配（`tag.begins_with`）逻辑也在此做——tag_set 已展平，直接 O(1) 查。

---

## 二、P0：营地锚点缓存

### world_rules.gd

加类变量：

```
static var _cached_camp_anchor: CardBase = null
static var _cached_camp_anchor_tick: int = 0
```

`camp_domain_anchor()` 改为：

```
static func camp_domain_anchor() -> CardBase:
    if _cached_camp_anchor_tick == GameState.current_tick:
        return _cached_camp_anchor
    _cached_camp_anchor = _find_camp_anchor()  # 原来的查找逻辑
    _cached_camp_anchor_tick = GameState.current_tick
    return _cached_camp_anchor
```

### game_state.gd

加 `static var current_tick: int = 0`，在 `_game_tick` 入口 `current_tick += 1`。

---

## 三、P1：`get_card_at` 改 SpatialIndex.by_cell

### game_state.gd `get_card_at`

当前：
```
for card in cards:
    if card.grid_x == x and card.grid_y == y:
        return card
```

改为从 `SpatialIndex.by_cell` 读：

```
var cell_data = spatial_index.by_cell.get(Vector2i(x, y), {})
# 取地面可见卡（非 in_pool / in_tree / in_ground / cell.overlay 卡）
for tag in cell_data:
    for card in cell_data[tag]:
        if card.visible_on_ground():  # 或用 WorldRules 判定
            return card
return null
```

注意 `get_card_at` 返回的是"地表可见卡"——`in_pool`/`in_tree`/`in_ground` 的卡不返。`SpatialIndex.by_cell` 是全部卡，需过滤。

如果过滤逻辑太重，另一个方案：`SpatialIndex` 加一个 `surface_cards_at(x, y)` 方法——在 `on_spawn` / `set_in_pool` 等状态变化时维护一个单独的 `by_cell_surface`。

---

## 四、P1：`best_feed_source_for` 节流

每 actor 每 tick 最多算一次。`CapabilityBehaviorPipeline` 和 `PredatorDenBehavior` 可能同 tick 重复调。

### capability_behavior_pipeline.gd

在 `_tick_wolf_den_work` 调 `find_meat_source` 前，先查 `actor` 上是否有 `_last_feed_source_tick == GameState.current_tick`，有则跳过。

### CardBase

加字段：
```
var _last_feed_source_tick: int = -1
var _cached_feed_source: CardBase = null
```

---

## 五、涉及文件

| 文件 | 改动 |
|------|------|
| `scripts/cards/card_db.gd` | `type_tag_sets` 预建字典 |
| `scripts/core/world_rules.gd` | `type_has_tag` 改集合读取 + `camp_domain_anchor` 缓存 |
| `autoload/game_state.gd` | `current_tick` 计数器 + `get_card_at` 改 by_cell |
| `scripts/core/spatial_index.gd` | 可选：`surface_cards_at` 方法 |
| `scripts/core/capability_behavior_pipeline.gd` | `best_feed_source_for` 节流 |
| `scripts/cards/card_base.gd` | feed source 缓存字段 |

## 六、验收标准

- L0 993 断言不降
- Profiler 复测：`_game_tick` < 300ms
- `type_has_tag` 累计 < 1s
- `defines_domain` + `type_domains` + `in_camp_home_territory` 合计 < 5s
- F5 进游戏：生态行为一致

## 约束

- 不动行为逻辑
- 不动标签/能力定义
- 记 fix-log
