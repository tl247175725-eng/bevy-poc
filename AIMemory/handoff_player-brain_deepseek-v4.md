# 人卡大脑系统 — 全量落地

**From**: deepseek-v4
**To**: cursor
**Date**: 2026-06-04
**设计文档**: `docs/design/human-card-intelligence.md`

---

## 体系概述

当前玩家卡行为是硬编码。目标：五层标签化智能。

```
第五层 BDI意图引擎 ─┐
第四层 执行承诺     ├─ 本次实现
第三层 SDT需求检测  │
第二层 可供性检测   │
第一层 标签层       ─┘ 已有，不动
```

动物跳过二三四五层，直接标签→行为管线。只有人卡走完整五层。

---

## 第一步：可供性检测层

**文件**: `scripts/core/player_affordance.gd`（新建）

**函数**: `static func detect(card: CardBase) -> Dictionary`

**改为**: 读取世界状态 + 玩家标签，返回当前可供性字典。

```gdscript
static func detect(player: CardBase) -> Dictionary:
    var affordances := {}
    var caps := _WorldRules.capabilities_of(player)
    
    # 可制石刀：有 hunt 或 craft 能力 + 手持和地面共 ≥2 块石头
    if (caps.has("capability.hunt") or caps.has("capability.craft")) and _count_stones_near(player) >= 2:
        affordances["craft_knife"] = {"need": "competence", "score": 8}
    
    # 可猎：有 hunt 能力 + 有武器 + 有猎物在范围内
    if caps.has("capability.hunt") and _has_weapon(player):
        var prey := _WorldRules.best_hunt_target(player)
        if prey:
            affordances["hunt"] = {"need": "competence", "target": prey, "score": 7}
    
    # 可徒手猎：有 hunt 但无武器 → 低成功率
    if caps.has("capability.hunt") and not _has_weapon(player):
        var prey := _WorldRules.best_hunt_target(player)
        if prey:
            affordances["hunt_bare"] = {"need": "competence", "target": prey, "score": 2}
    
    # 可采集浆果：有 forage 能力 + 灌木在附近
    if caps.has("capability.forage") and _bush_with_berry_near(player):
        affordances["forage"] = {"need": "hunger", "score": 4}
    
    # 可收集燃料：火在烧 + 燃料不足 + 木头/树枝在附近
    if _fire_needs_fuel() and _wood_near(player):
        affordances["collect_fuel"] = {"need": "relatedness", "score": 6}
    
    # 可制工具链（后续扩展）
    if caps.has("capability.craft"):
        # 检查各种制作配方的前置条件
        ...
    
    return affordances
```

**规则**: 可供性字典不写死在函数内。每个可供性条目 = 条件检查 + 对应的 SDT 需求标签。后续新可供性只需加条目，不改架构。

## 第二步：SDT 需求检测层

**文件**: `scripts/core/player_needs.gd`（新建）

**函数**: `static func evaluate(player: CardBase, affordances: Dictionary) -> Dictionary`

**改为**: 计算三个根需求的当前满足度（0-100）。

```gdscript
static func evaluate(player: CardBase, affordances: Dictionary) -> Dictionary:
    var needs := {"autonomy": 100, "competence": 100, "relatedness": 100}
    
    # 自主：有什么在强迫我行动？
    var wolf_near := _wolf_within(player, _FEAR_RANGE)
    var fire_dead := _fire_is_dead()
    var starving := player.hunger > _STARVE_THRESHOLD
    
    if wolf_near:
        needs["autonomy"] -= 80    # 被狼追 = 严重失去自主
    if fire_dead:
        needs["autonomy"] -= 60    # 火=生存依赖，没了火人被环境主宰
    if starving:
        needs["autonomy"] -= 40    # 饥饿强迫行动
    
    # 胜任：我能有效完成吗？
    var has_weapon := _has_weapon(player)
    var has_tools := _count_tools(player) > 0
    var recent_success := _recent_hunt_success(player)
    
    if not has_weapon:
        needs["competence"] -= 30
    if not has_tools:
        needs["competence"] -= 20
    if recent_success:
        needs["competence"] += 15   # 上次猎杀成功→胜任感上升
    
    # 关联：我和营地/火/他人有连接吗？
    var fire_alive := _fire_is_alive()
    var has_shelter := _hut_exists()
    var near_camp := _distance_to_camp(player) < _CAMP_RANGE
    
    if not fire_alive:
        needs["relatedness"] -= 70   # 火=家。没火=没有关联锚点
    if not has_shelter:
        needs["relatedness"] -= 30
    if near_camp:
        needs["relatedness"] += 20
    
    # 夹紧到 [0, 100]
    for key in needs:
        needs[key] = clamp(needs[key], 0, 100)
    
    return needs
```

## 第三步：意图引擎（欲望→意图）

**文件**: `scripts/core/player_intention.gd`（新建）

**核心逻辑**: 欲望池竞争 → 胜出者提升为意图 → GPS 反向规划 → 输出动作链。

```gdscript
# 欲望池：从可供性 + 需求推导候选目标
static func generate_desires(player: CardBase, affordances: Dictionary, needs: Dictionary) -> Array:
    var desires := []
    for key in affordances:
        var aff := affordances[key]
        var need_key := aff.get("need", "")
        if need_key in needs:
            # 需求越不满足，该可供性的优先级越高
            var urgency := 100 - needs[need_key]
            aff["urgency"] = urgency
            desires.append(aff)
    desires.sort_custom(func(a, b): return a.urgency > b.urgency)
    return desires

# 意图槽：从欲望池中选定一个，GPS 反向规划后提升为意图
static var _intention: Dictionary = {}      # 当前长期意图
static var _intention_time_pct: float = 0.3  # 长期意图的时间配比
static var _intention_progress: int = 0       # 子目标进度

static func tick_intention(player: CardBase, desires: Array, needs: Dictionary) -> String:
    # 如果意图已完成或条件失效，释放
    if not _intention.is_empty():
        if _intention_fulfilled(player) or _intention_impossible(player):
            _intention = {}
            _intention_progress = 0
    
    # 意图槽空 → 检查是否可以激活长期意图
    if _intention.is_empty():
        for desire in desires:
            if _is_long_term(desire) and _prerequisites_met(player, desire):
                _intention = desire
                _intention_progress = 0
                break
    
    return _intention.get("key", "")
```

## 第四步：执行承诺

**文件**: `scripts/core/player_intention.gd`（同文件）

**函数**: `static func should_abort(player: CardBase, current_action: String, threat: Dictionary) -> bool`

**改为**: 判断正在执行的动作是否可以被中断。

```gdscript
static func should_abort(player: CardBase, current_action: Dictionary, threat: Dictionary) -> bool:
    var threat_level := threat.get("level", 0)  # 0=无威胁, 1=便利, 2=生存, 3=物理
    var time_to_threat := threat.get("ticks_to_impact", 999)
    var time_to_finish := current_action.get("ticks_remaining", 0)
    
    # 物理威胁（狼<3格）：无条件中断
    if threat_level >= 3:
        return true
    
    # 生存威胁（火灭/饿死）：如果能完成当前动作还来得及→不中断
    if threat_level >= 2:
        return time_to_finish > time_to_threat
    
    # 便利性中断（新目标优先级更高）：不中断，先干完手头的
    return false
```

## 第五步：整合——替换旧硬编码

**文件**: `scripts/world/behaviors/player_behavior.gd`（修改现有文件或新建）

**改为**: 每 tick 的玩家行为调度。

```gdscript
static func tick(player: CardBase, delta: float) -> void:
    # 1. 可供性检测
    var aff := _PlayerAffordance.detect(player)
    
    # 2. 需求评估
    var needs := _PlayerNeeds.evaluate(player, aff)
    
    # 3. 欲望生成
    var desires := _PlayerIntention.generate_desires(player, aff, needs)
    
    # 4. 意图更新
    var intent_key := _PlayerIntention.tick_intention(player, desires, needs)
    
    # 5. 执行承诺：检查当前动作是否该中断
    var current_action := _current_action(player)
    var threat := _detect_threat(player)
    if _PlayerIntention.should_abort(player, current_action, threat):
        _execute_emergency(player, threat)
        return
    
    # 6. 正常执行：意图有动作→做意图的。无意图→做欲望池第一名。
    ...
```

---

## 验收

- 玩家卡不再无故屠杀水牛/鹿/羊
- 玩家卡优先生火（fire_bond → 关联需求 → 最高优先级）
- 玩家卡空闲时不绕圈——显示"休息中"，意图为空
- L0 断言不降
- 记 fix-log

---

## 涉及文件

| 文件 | 新建/修改 |
|------|---------|
| `scripts/core/player_affordance.gd` | 新建 |
| `scripts/core/player_needs.gd` | 新建 |
| `scripts/core/player_intention.gd` | 新建 |
| `scripts/world/behaviors/player_behavior.gd` | 修改（替换旧硬编码） |

## 约束

- 不碰生态管线/封闭模式/ecosystem_behavior_key
- 新代码零 card_type 硬编码
- player 卡类型不特殊对待——走标签查询
