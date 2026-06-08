# 生态二期 D-前半：溢出落地 —— 橡子松塔 + 池采收 + 鸟巢

**From**: deepseek-v4
**To**: cursor
**Date**: 2026-06-06
**Priority**: HIGH
**设计依据**: `docs/design/diversity-overflow-design.md`

---

C 阶段把容器建好了——树里有栎树/松树，水潭里有菱角/莲。D-前半让它们产出东西。

---

## D1：橡子 + 松塔自然掉落

### 1.1 新卡

`card_db.gd`：

```
_reg("acorn", "橡", "橡子", "entity", ["food.edible","camp.storable"])
_reg("pineCone", "塔", "松塔", "entity", ["food.edible","camp.storable"])
```

橡子和松塔可被玩家/兔/鼠/雉鸡吃，人也可烤。

### 1.2 生产 tick

`scripts/world/environment_manager.gd`，加 `_tick_contained_producers(real_delta)`，在 `_tick_natural_drops` 同级调用。

逻辑：
- 遍历所有 `in_tree` 的卡
- 有 `nut_producer` 标签 → 生产橡子，周期 `NUT_PRODUCE_INTERVAL`（如 25s）
- 有 `cone_producer` 标签 → 生产松塔，周期 `CONE_PRODUCE_INTERVAL`（如 30s）
- timer 到 → 取宿主树的邻格（空位），spawn 橡子/松塔，`state = "自然掉落"`
- 同一树的邻格已有同类掉落（橡子/松塔各限 2 个邻格），跳过本轮

`autoload/game_state.gd`：

```
const NUT_PRODUCE_INTERVAL: float = 25.0
const CONE_PRODUCE_INTERVAL: float = 30.0
```

### 1.3 CARD_CAPABILITIES

```
"acorn": ["capability.be_carried","capability.be_stored","capability.be_consumed"],
"pineCone": ["capability.be_carried","capability.be_stored","capability.be_consumed"],
```

---

## D2：池采收 —— 菱角 + 莲蓬

### 2.1 交互注册

`interaction_manager.gd`，新增 `_try_harvest_pool`：

- 触发条件：人手持空手（无工具）走到水潭邻格
- 目标格有 `waterCaltrop` → 采收菱角，产出 `acorn`（菱角当食物，复用橡子卡，或新增 `caltropFruit`——见下）
- 目标格有 `lotus` → 采收莲蓬，产出 `lotusSeed`

不收走宿主卡——`waterCaltrop` / `lotus` 留在原位，只是产出物品。

### 2.2 新卡（可选：菱角独立物品）

如果不想复用橡子，独立注册：

```
_reg("caltropFruit", "菱", "菱角", "entity", ["food.edible","camp.storable"])
_reg("lotusSeed", "籽", "莲子", "entity", ["food.edible","camp.storable"])
```

如果嫌卡多，直接用 `acorn` 和新增 `lotusSeed`。Cursor 决定。

### 2.3 再生

采收后，`waterCaltrop` / `lotus` 进入冷却：`state = "采尽"`，`N` 秒后恢复 `state = "漂浮"` / `"挺水"`，可再采。

`game_state.gd`：

```
const POOL_HARVEST_REGEN_SECONDS: float = 40.0
```

---

## D3：鸟巢

### 3.1 新卡

`card_db.gd`：

```
_reg("birdNest", "巢", "鸟巢", "entity", ["nest","organize.locked"], 0, true)
```

### 3.2 容纳

鸟巢 `in_tree = true`，走已有树容纳系统——和栎树/松树同列在树的「容纳」列表里。

`finalize_tree_spawn` 已可处理任意 `in_tree` 卡，无需改。

### 3.3 初始生成

`world_manager.gd`，树 spawn 之后：随机选 1-2 棵树，在树内放鸟巢：

```
var nest = _spawn("birdNest", tree.grid_x, tree.grid_y, {"state": "栖息"})
WorldRules.finalize_tree_spawn(nest)
```

### 3.4 能力

```
"birdNest": ["capability.be_collected"],
```

鸟巢后续可被采集鸟蛋（D-后半或以后）。

---

## 涉及文件

| 文件 | 改动 |
|------|------|
| `scripts/cards/card_db.gd` | acorn / pineCone / birdNest / (可选 caltropFruit lotusSeed) `_reg` |
| `autoload/game_state.gd` | NUT_PRODUCE_INTERVAL / CONE_PRODUCE_INTERVAL / POOL_HARVEST_REGEN_SECONDS |
| `scripts/core/world_rules.gd` | 新卡 CARD_CAPABILITIES |
| `scripts/world/environment_manager.gd` | `_tick_contained_producers` |
| `scripts/core/interaction_manager.gd` | `_try_harvest_pool` 池采收 |
| `scripts/world/world_manager.gd` | 鸟巢初始生成 |
| `scripts/ui/selection_info_panel.gd` | 新卡/标签中文化 |

## 约束

- L0 断言数不降
- 不碰 `AquaticEcologyBehavior`
- 不新增 behavior 文件（生产 tick 放 `environment_manager`，采收放 `interaction_manager`）
- `nut_producer` / `cone_producer` 行为落在此 phase
- 记 fix-log
