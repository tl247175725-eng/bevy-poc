# 能力驱动的行为调度 — 原子标签吸附

> 状态：执行正本  
> 目标：让标签从"描述"变成"引擎"——Registry 不再按 card_type 分发，而是读能力标签组合行为

---

## 1. 现状 vs 目标

```
现状：
card_type == "wolf"  → ecosystem_behavior_key → "wolf_pack" → WolfBehavior.tick
card_type == "fox"   → ecosystem_behavior_key → "fox_family" → FoxBehavior.tick

目标：
card 有 capability.hunt + capability.move → 自动接入捕猎行为链
card 有 capability.scavenge               → 自动接入清道夫行为链
card 有 capability.use_cover              → 自动接入掩体移动链
```

## 2. 原理

每张自动卡的行为 = 它的所有能力标签各自贡献的行为片段之和。

```
狐狸 = hunt（基础捕猎）
     + scavenge（偷尸体肉）
     + use_cover（灌木加速+隐藏）
     + reproduce（繁殖）
     + den_build（筑窝，且窝来自灌木转化）

狼   = hunt（基础捕猎）
     + pack（群居密度判定）
     + reproduce（繁殖）
     + den_build（筑窝，且窝来自叼干草建窝）
     + care_child（喂幼狼）
```

共享能力（hunt、reproduce、den_build）的逻辑完全一致。差异部分（scavenge vs pack vs care_child）来自各自独有的标签。

## 3. 实现骨架

### 3.1 行为片段注册

WorldRules 中新增一个字典，将每个能力标签映射为一个行为片段：

```gdscript
static var CAPABILITY_BEHAVIOR: Dictionary = {
    "capability.hunt":         _tick_hunt,
    "capability.scavenge":     _tick_scavenge,
    "capability.use_cover":    _tick_cover_movement,
    "capability.reproduce":    _tick_reproduce,
    "capability.care_child":   _tick_care_child,
    "capability.flee":         _tick_flee,
    "capability.return_home":  _tick_return_home,
}
```

每个片段是一个小函数，签名统一：`func _tick_xxx(actor: CardBase, real_delta: float) -> void`

### 3.2 行为组装

EcosystemTickRegistry 不再按 card_type 查表。改为：

```gdscript
func tick_card(actor: CardBase, real_delta: float) -> void:
    var caps := _WorldRules.capabilities_of(actor)
    for cap in caps:
        var behavior := _WorldRules.CAPABILITY_BEHAVIOR.get(cap)
        if behavior:
            behavior.call(actor, real_delta)
```

### 3.3 行为片段内部的共同依赖

所有片段通过 WorldRules 消费通用规则查询：
- `best_hunt_target(actor)` — 谁是可猎目标（已有）
- `hunt_success_chance(actor, target)` — 攻击是否成功（已有）
- `move_interval_for(actor)` — 移动速度（已有）
- `cover_move_multiplier(actor)` — 掩体加速（已有）
- `home_for_actor(actor)` — 归属窝（已有）
- `flee_from(actor, threat)` — 逃跑方向（已有）

片段本身不实现这些——它们已经在 WorldRules 里。

## 4. 迁移路径（三步）

### 步骤 A：迁移 `capability.hunt`（第一个样板）

将 WolfBehavior 和 FoxBehavior 中的捕猎逻辑提取为一个共享的 `_tick_hunt` 片段。

范围：
- 选猎物（`best_hunt_target`）
- 靠近猎物（`move_one_step_near`）
- 攻击判定（`try_hunt_attack` / `hunt_success_chance`）
- 消化冷却（`huntCooldown`）

验收：狼的捕猎行为不变，狐狸取消 spawn 注释后捕猎行为自动激活。

### 步骤 B：迁移其余能力

逐一迁移 `flee`、`reproduce`、`use_cover`、`return_home`、`scavenge`、`care_child`。

### 步骤 C：废除 card_type 分发

`ecosystem_behavior_key` 的 card_type 分支可以全部删除。`FoxBehavior.gd` 和 `WolfBehavior.gd` 可以删除（行为片段已在 WorldRules 中）。

## 5. 不做

- 不一次性迁移全部能力（逐个切，每刀可验收）
- 不废除 card_type 本身（它仍是 card_db 的 identity 标签）
- 不碰非自动卡（grass、tree、bush、工具、建筑等没有 tick 的卡）
