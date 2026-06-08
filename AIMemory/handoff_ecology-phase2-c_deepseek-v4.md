# 生态二期 C：容纳扩展 —— 树 + 水潭

**From**: deepseek-v4
**To**: cursor
**Date**: 2026-06-06
**Priority**: HIGH
**设计依据**: `docs/design/diversity-overflow-design.md`

---

水潭容纳已有地基——`in_pool` 字段 + `ui_pool_containment_entries_at` 在 Cursor 之前就建好了，藻/鱼/虫/贝已经走 `in_pool` 隐藏和容纳显示。

C 做两件事：**树容纳栎树/松树**（全新）+ **池容纳菱角/莲**（对齐已有池模式）。

---

## 一、树容纳系统（全新）

### 1.1 card_base.gd — 加 `in_tree` 字段

照搬 `in_pool` 模式：

```
var in_tree: bool = false

func set_in_tree(value: bool) -> void:
    in_tree = value
    # in_tree 卡在地表不可见（同 in_pool）
    visible = not value
```

### 1.2 world_rules.gd — 加判定函数

```
static func is_tree_host(card: CardBase) -> bool:
    return is_instance_valid(card) and card_has_tag(card, "source.lumber") and card_has_tag(card, "rooted")

static func tree_occupants_at(gx: int, gy: int) -> Array:
    # 返回该格所有 in_tree 的卡
```

### 1.3 world_rules_ui.gd — 加树容纳条目

```
static func _tree_containment_entries(tree: CardBase) -> Array:
```

对同格且 `in_tree` 的卡，用已有 `_containment_entry` 格式化显示。

### 1.4 world_rules_ui.gd — `ui_containment_entries` 加树分支

```
if is_tree_host(host):
    return _tree_containment_entries(host)
```

### 1.5 game_ui.gd — 树点击时调用树容纳渲染

当选中卡为树时，`cell_entries` 走树容纳（类似已有 `ui_pool_containment_entries_at` 调用模式）。

---

## 二、四张新卡

### 2.1 card_db.gd

```
# 树容纳卡（in_tree，地表不可见）
_reg("oak", "橡", "栎树", "entity", ["rooted","nut_producer","organize.locked"], 0, true)
_reg("pine", "松", "松树", "entity", ["rooted","cone_producer","organize.locked"], 0, true)

# 水潭容纳卡（in_pool，同现有水生卡）
_reg("waterCaltrop", "菱", "菱角", "entity", ["floating","food.edible","organize.locked"], 0, true)
_reg("lotus", "莲", "莲", "entity", ["floating","food.edible","organize.locked"], 0, true)
```

### 2.2 world_rules.gd — CARD_CAPABILITIES

```
"oak": ["capability.be_collected"],
"pine": ["capability.be_collected"],
"waterCaltrop": ["capability.be_collected"],
"lotus": ["capability.be_collected"],
```

`be_collected` 表示可采收——行为在 D 阶段落地，C 只注册能力。

### 2.3 world_rules.gd — 树宿主 spawn 辅助

```
static func finalize_tree_spawn(card: CardBase) -> void:
    if not is_instance_valid(card):
        return
    card.set_in_tree(true)
```

### 2.4 新标签注册

`card_rule_audit.gd` 维度归类：

```
"nut_producer": DIM_CAPABILITY,      # 栎树产橡子
"cone_producer": DIM_CAPABILITY,     # 松树产松塔
"floating": DIM_IDENTITY,            # 水面附着
```

---

## 三、初始生成

### 3.1 world_manager.gd `_spawn_initial_cards`

树容纳卡：在已有 `tree` 卡 spawn 之后，随机选几棵树，在其格上 spawn oak/pine：

```
# 遍历已有 tree 卡，随机 3-4 棵放 oak，3-4 棵放 pine
for tree in all_trees:
    if rng < threshold:
        var oak = _spawn("oak", tree.grid_x, tree.grid_y, {"state": "生长"})
        WorldRules.finalize_tree_spawn(oak)
```

池容纳卡：在水潭格随机 spawn 菱角和莲：

```
for pool_cell in pool_cells:
    if rng < 0.15:
        var caltrop = _spawn("waterCaltrop", cell.x, cell.y, {"state": "漂浮"})
        caltrop.set_in_pool(true)
    if rng < 0.10:
        var lotus = _spawn("lotus", cell.x, cell.y, {"state": "挺水"})
        lotus.set_in_pool(true)
```

---

## 四、涉及文件

| 文件 | 改动 |
|------|------|
| `scripts/cards/card_base.gd` | `in_tree` 字段 + `set_in_tree()` |
| `scripts/cards/card_db.gd` | 四条 `_reg` |
| `scripts/core/world_rules.gd` | CARD_CAPABILITIES 四条 + `is_tree_host` + `tree_occupants_at` + `finalize_tree_spawn` |
| `scripts/core/world_rules/world_rules_ui.gd` | `_tree_containment_entries` + `ui_containment_entries` 加树分支 |
| `scripts/ui/game_ui.gd` | 树选中时调树容纳渲染 |
| `scripts/ui/selection_info_panel.gd` | `nut_producer`/`cone_producer`/`floating` 中文化 |
| `scripts/world/world_manager.gd` | 树内 spawn oak/pine + 池内 spawn 菱角/莲 |
| `scripts/test/card_rule_audit.gd` | 新标签维度归类 + 新卡 audit 条目 |

## 约束

- L0 断言数不降
- 不碰 `AquaticEcologyBehavior`（池容纳仅 UI 叠加，不改行为管线）
- 不新增 behavior 文件
- `nut_producer` / `cone_producer` / `floating` 仅注册标签，不落行为
- `be_collected` 仅注册能力，不落采收逻辑
- 记 fix-log
