# 人卡大脑重构：标签驱动化

**From**: deepseek-v4
**To**: cursor
**Date**: 2026-06-04
**Priority**: HIGH
**设计依据**: `docs/design/human-card-intelligence.md`

---

## 铁律（搁在文件头部）

**每行决策从标签推导，禁用任何不经过标签的硬编码条件。**
禁用：`if hunger > 72`、`if has_camp_domain_anchor()`、`if "spear" in tools`、任意魔法数字。
允许：`if card_has_tag("hungry")`、`if card_has_capability("capability.hunt")`、`if WorldRules.nearest_threat(actor, "threat") != null`。
标签是决策的唯一依据。数值从标签推导，不存在没有标签依据的数值。

---

## 一、player_affordance.gd — 可供性从标签×世界推导

**现状**: 每条可供性是一个独立函数。新增可供性 = 新增函数。`_afford_hunt` 硬编码检查"有矛吗"。

**改为**: 可供性从两个维度交叉匹配：
1. 玩家标签 → 玩家的能力边界（能做什么）
2. 世界状态 → 世界的物体标签（提供了什么）

### 1.1 可供性注册表

```gdscript
const AFFORDANCE_TABLE := {
    "hunt_armed": {
        "require_caps": ["capability.hunt"],
        "require_tags": ["has_weapon"],  # 由 _eval_condition 处理
        "require_world": ["prey_available"],  # 有可猎猎物
        "need": "competence",
        "score": 7,
    },
    "hunt_bare": {
        "require_caps": ["capability.hunt"],
        "require_not_tags": ["has_weapon"],
        "require_world": ["prey_available"],
        "need": "competence",
        "score": 2,
    },
    "collect_fuel": {
        "require_tags": ["fire_bond"],
        "require_world": ["camp_fire_exists", "wood_nearby"],
        "need": "relatedness",
        "score": 6,
    },
    "craft_knife": {
        "require_caps": ["capability.craft"],
        "require_world": ["stones_available", "no_knife_owned"],
        "need": "competence",
        "score": 8,
    },
    "craft_spear": {
        "require_caps": ["capability.craft"],
        "require_world": ["stone_available", "wood_nearby", "no_spear_owned"],
        "need": "competence",
        "score": 7,
    },
    "build_hut": {
        "require_tags": ["tool_dependent"],
        "require_world": ["no_shelter_exists"],
        "need": "relatedness",
        "score": 9,
    },
    "relight_fire": {
        "require_tags": ["fire_bond"],
        "require_world": ["no_camp_fire"],
        "need": "relatedness",
        "score": 12,
    },
    "forage": {
        "require_caps": ["capability.forage"],
        "require_world": ["berry_bush_nearby"],
        "need": "hunger",
        "score": 4,
    },
}
```

### 1.2 世界条件检查函数（从标签推导，不硬编码物体类型）

```gdscript
static func _check_world_condition(player: CardBase, condition: String) -> bool:
    match condition:
        "prey_available":
            return _best_player_prey(player) != null
        "camp_fire_exists":
            return _WorldRules.has_camp_domain_anchor()
        "wood_nearby":
            return _card_with_tag_in_range(player, "material.lumber", 8)
        "stones_available":
            return _count_cards_with_tag(player, "material.stone", 6) >= 2
        "stone_available":
            return _count_cards_with_tag(player, "material.stone", 6) >= 1
        "no_knife_owned":
            return not _player_has_tool_tag(player, "sharp")
        "no_spear_owned":
            return not _player_has_tool_tag(player, "sharp") or "spear" not in player.tools
        "berry_bush_nearby":
            return _card_with_tag_in_range(player, "berry.source", 8)
        "no_shelter_exists":
            return WorldHelpers.find_camp() == null
        "no_camp_fire":
            return not _WorldRules.has_camp_domain_anchor()
    return false
```

### 1.3 detect() 改为遍历注册表

```gdscript
static func detect(player: CardBase) -> Dictionary:
    if not _WorldRules.is_player_card(player):
        return {}
    var caps := _WorldRules.capabilities_of(player)
    var affordances := {}
    for key in AFFORDANCE_TABLE:
        var rule := AFFORDANCE_TABLE[key]
        if not _check_caps(caps, rule.get("require_caps", [])):
            continue
        if not _check_tags(player, rule.get("require_tags", [])):
            continue
        if not _check_not_tags(player, rule.get("require_not_tags", [])):
            continue
        if not _check_world_conditions(player, rule.get("require_world", [])):
            continue
        affordances[key] = {"key": key, "need": rule["need"], "score": rule["score"]}
    return affordances
```

新增可供性 = 在 `AFFORDANCE_TABLE` 加一条，不改任何函数。**这是标签驱动的核心。**

---

## 二、player_needs.gd — 需求从标签推导

**现状**: `有狼 → 减 80`、`没火 → 减 60`、`饿 → 减 40`。魔法数字，无标签依据。

**改为**: 需求标签定义贡献值。

### 2.1 需求标签注册表

```gdscript
const NEED_TAG_TABLE := {
    # 自主：什么在强迫我行动？
    "fire_bond": {"need": "autonomy", "unsatisfied": -60, "satisfied": 0},
    "predator": {"need": "autonomy", "nearby": -80, "distant": 0},  # 任何有 predator 标签的生物
    "hungry": {"need": "autonomy", "active": -40, "inactive": 0},
    # 胜任：我能有效完成吗？
    "has_weapon": {"need": "competence", "true": 0, "false": -30},
    "has_tools": {"need": "competence", "true": 0, "false": -20},
    "recent_hunt_success": {"need": "competence", "true": 15, "false": 0},
    # 关联：我和营地/火/他人有连接吗？
    "fire_bond": {"need": "relatedness", "unsatisfied": -70, "satisfied": 0},  # 火=关联锚点
    "has_shelter": {"need": "relatedness", "true": 0, "false": -30},
    "near_camp": {"need": "relatedness", "true": 20, "false": 0},
}
```

### 2.2 evaluate() 改为遍历标签计算贡献

```gdscript
static func evaluate(player: CardBase) -> Dictionary:
    var needs := {"autonomy": 100, "competence": 100, "relatedness": 100}
    if not _WorldRules.is_player_card(player):
        return needs
    for tag_key in NEED_TAG_TABLE:
        var rule := NEED_TAG_TABLE[tag_key]
        var need_name := rule["need"]
        var contribution := _eval_need_tag(player, rule)
        needs[need_name] += contribution
    for key in needs.keys():
        needs[key] = clampi(needs[key], 0, 100)
    return needs
```

### 2.3 标签条件的检查用标签，不用魔法数字

`hungry` 标签的判断不是 `player.hunger > 72`。是检查玩家卡有没有 `hungry` 标签——而这个标签应该在生理状态变化时自动挂上/取下（由 `_update_hunger_tag` 负责）。**需求系统不直接读 `player.hunger` 数值。**

`predator` 标签的"附近"检查不是 `if is_adult_wolf and manhattan <= 3`。是 `WorldRules.nearest_threat(player, "threat", radius)` 返回非 null。

---

## 三、player_intention.gd — 意图从可供性+需求推导

**现状**: `LONG_TERM_KEYS` 硬编码 `build_greenhouse`。`_prerequisites_met` 硬编码检查蘑菇棚。

**改为**: 长期意图注册表。

```gdscript
const LONG_TERM_TABLE := {
    "build_greenhouse": {
        "require_caps": ["capability.craft"],
        "require_world": ["wood_available_x3", "craft_station_available"],
        "require_needs_satisfied": true,  # 只有生存盈余时才激活
    },
}

static func _prerequisites_met(player: CardBase, desire: Dictionary) -> bool:
    var key := desire.get("key", "")
    var rule := LONG_TERM_TABLE.get(key, {})
    if rule.is_empty():
        return false
    if rule.get("require_needs_satisfied", false):
        if not _survival_surplus(player):
            return false
    return _check_world_conditions(player, rule.get("require_world", []))
```

`_intention_fulfilled` 和 `_intention_impossible` 同理——从注册表查询完成条件和放弃条件。

---

## 四、player_behavior.gd — 大脑 tick 保持，清理硬编码辅助

`_wolf_near` → 改用 `WorldRules.nearest_threat(player, "threat", radius)`。
`_fire_emergency` → 读取 `fire_bond` 标签的满足状态。
`_should_idle_rest` → 读取必需标签的满足状态，不直接读 `player.hunger` 数值。

---

## 五、玩家卡标签补充

`card_db.gd` 的 player 标签更新。当前 `actor` 标签不参与决策。需要以下标签驱动需求系统：

```
player: ["being","actor","omnivore","tool_dependent","fire_bond","opportunistic","body.large"]
```

其中：
- `fire_bond` → 驱动"火不能灭"的需求
- `tool_dependent` → 驱动"无工具时胜负感低"
- `opportunistic` → 驱动"看到材料想捡"的可供性
- `hungry` → 不在 CardDB 静态注册——是运行时标签，由生理 tick 挂上/取下

---

## 验收

- `player_affordance.gd` 新增可供性只需在 `AFFORDANCE_TABLE` 加条目，不改函数
- `player_needs.gd` 零魔法数字，全从 NEED_TAG_TABLE 推导
- `player_intention.gd` 零硬编码键名，全从 LONG_TERM_TABLE 推导
- 第二张人卡（不同初始标签）加入时，五层自动适配，不改任何逻辑代码
- L0 断言数不降
- 记 fix-log
