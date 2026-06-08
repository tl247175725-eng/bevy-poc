# 调度改造 Step 3：能力片段 + 优先级调度器

**From**: cursor | **To**: cursor（自执行）  
**Date**: 2026-06-01 | **Parent**: proposition 6 步表 Step 3  
**前置**: Step1 猎物 DONE（772）· Step2 饱食 DONE（772）

---

## 目标

在 `EcosystemTickRegistry` 为**成年狼/狐**引入**硬编码有序**能力片段管线（禁止 `for cap in capabilities_of` 无序循环）：

```
flee → fire → den_work → scavenge → hunt
```

每片段签名：

```gdscript
# 返回 true = 本 tick 管线在此截断（不再跑后续片段）
# 出参 consumed_tick = 本片段是否调用了 consume_move_tick
static func tick_capability_XXX(host, actor, real_delta) -> bool
static func tick_capability_XXX_consumed_move(...) -> bool  # 可选：由管线汇总
```

本步从狼/狐 behavior **抽出**：`flee`、`fire`（惧火）、`return_home`/`den_work`、`scavenge`；`hunt` 并入管线末尾（替代 Registry 末尾单独调用）。

---

## 不碰

- `ecosystem_behavior_key` 物种 match（Step5）
- 草食/田鼠/兔等其它 `*Behavior`（仍走原 tick）
- 幼体/窝内/眩晕逻辑（留在 `WolfBehavior`/`FoxBehavior` 壳）
- `game_ui` 大改

---

## 管线适用

```gdscript
applies_predator_pipeline(card):
  ecosystem_key in ["wolf_pack", "fox_family"]
  and can_hunt(card)
  and not (wolf_cub or fox_cub)
  and not (stunned)
```

---

## 片段职责（从现网平移）

| 序 | 片段 | 能力门闩 | 狼 | 狐 |
|----|------|----------|----|----|
| 1 | flee | `escape_cover` 或 mesopredator 躲猎威胁 | — | 躲人/躲狼 |
| 2 | fire | 通用（篝火恐惧） | 惧火+cd | 惧火+cd |
| 3 | den_work | `return_home` | 搬肉/筑窝/饱腹cd/日肉配额/取肉 | 搬肉/筑窝/饱腹cd |
| 4 | scavenge | `scavenge` | — | try_fox_scavenge |
| 5 | hunt | `hunt` | tick_capability_hunt | 同左 |

**狼**：去掉 behavior L56 全局 `consume_move_tick` 早退（与狐对齐）。

---

## 文件

| 文件 | 操作 |
|------|------|
| `scripts/core/capability_behavior_pipeline.gd` | **新建** 管线 + 片段 |
| `ecosystem_tick_registry.gd` | 调用管线，删末尾独立 hunt |
| `wolf_behavior.gd` / `fox_behavior.gd` | 瘦身，删已迁出分支 |
| `world_rules.gd` | `tick_capability_hunt` 改返回 bool；去 `is_adult_fox` 游荡特例改 tag |
| `unit_test_cases.gd` | `_test_capability_pipeline_order` |

---

## 验收

- L0 全绿（含既有狼窝/狐清腐相关测）
- 狐：躲狼/惧火/清腐/捕猎仍可达；筑窝不回归（无全局 move 挡）
- 狼：无全局 move 挡；搬肉/捕猎/惧火行为不变
- F5：§5 狼狐猎杀 > 0

---

# 回复

**From**: cursor | **Date**: 2026-06-01 | **Status**: DONE

- 新建 `capability_behavior_pipeline.gd`：硬编码序 flee→fire→den_work→scavenge→hunt；片段返回 `handled` 截断管线。
- `EcosystemTickRegistry` 对成狼/成狐调用管线；移除末尾独立 hunt。
- `WolfBehavior`/`FoxBehavior` 仅留幼体/窝内/眩晕/迁出；狼去掉全局 `consume_move_tick` 早退。
- `tick_capability_hunt` 改返回 bool；狐游荡用 `mesopredator` tag。
- 叼肉时跳过 fire（保留「搬肉优先于惧火」）。
- L0 **779** PASS；`_test_capability_pipeline` 新增。
