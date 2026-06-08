# Phase 5：玩家 AI 迁移

**From**: deepseek-v4
**To**: cursor
**Date**: 2026-06-06
**Priority**: P0
**基准**: Godot `scripts/player/` + `player_needs_manager.gd`

---

## 目标

把 Godot 的玩家 AI 五层大脑迁移到 Bevy。不改行为逻辑——只把 GDScript 翻译成 Rust。

---

## 一、五层架构（从 Godot 逐层搬）

```
Perception（感知）
  → WorldModel（世界模型）
    → Affordance（可供性）→ "我能干嘛"
    → Needs（需求评估）  → "我需要什么"
    → Intention（意向）  → "我现在做什么"
    → Execution（执行）  → "怎么做"
```

### 层 1：感知层

**Godot 源**：`scripts/player/player_brain_world.gd` + `player_brain_tags.gd`

Bevy 实现：
- `PlayerBrain` Resource：存储玩家 RuntimeState——`has_weapon, near_camp, predator_nearby_unsafe, hungry...`
- `sync_player_tags(world)`：每 tick 从 WorldState 读取玩家周围状态，更新 PlayerBrain 的运行时标签
- 感知半径 = 玩家视野（周围 N 格）

### 层 2：可供性层

**Godot 源**：`scripts/core/player_affordance.gd`（AFFORDANCE_TABLE）

Bevy 实现：
- `AFFORDANCE_TABLE` 静态表：tag 组合 → 可供的 action
- `compute_affordances(brain, world)` → `Vec<String>`（"hunt","forage","craft_knife","craft_spear"...）
- 参照 Godot 的注册表，逐项迁移

### 层 3：需求层（SDT）

**Godot 源**：`scripts/core/player_needs.gd`（NEED_TAG_RULES）

Bevy 实现：
- SDT 三维：autonomy（自主）、competence（胜任）、relatedness（关联）
- `NEED_TAG_RULES` 静态规则表：tag + 状态 → delta
- `evaluate_needs(brain, world)` → `Needs { autonomy: f32, competence: f32, relatedness: f32 }`

### 层 4：意向层（BDI）

**Godot 源**：`scripts/core/player_intention.gd`（LONG_TERM_TABLE）

Bevy 实现：
- `LONG_TERM_TABLE` 静态表：条件 → 长期目标
- `select_intention(brain, needs)` → `Intention { goal, priority }`
- 参照 Godot 的绝对前提模型——威胁 > 依赖根 > 压力 > 安稳

### 层 5：执行层

**Godot 源**：`scripts/player/action_runner.gd` + `scripts/player/tasks/`

Bevy 实现：
- `ActionRunner`：读意向 → 拆任务 → 执行任务状态机
- `TaskFSM`：plan → move → pickup → move → drop → act → done/fail
- 核心任务链：生火、工具制作、狩猎、烹饪、建棚

---

## 二、需要迁移的 Godot 文件

```
scripts/player/action_runner.gd       → src/player/action_runner.rs
scripts/player/tasks/tool_tasks.gd    → src/player/tool_tasks.rs
scripts/player/tasks/survival_tasks.gd → src/player/survival_tasks.rs
scripts/player/tasks/shelter_tasks.gd → src/player/shelter_tasks.rs
scripts/core/player_affordance.gd     → src/player/affordance.rs
scripts/core/player_needs.gd          → src/player/needs.rs
scripts/core/player_intention.gd      → src/player/intention.rs
scripts/core/player_behavior.gd       → src/player/behavior.rs
scripts/core/player_brain_tags.gd     → src/player/brain_tags.rs
scripts/core/player_brain_world.gd    → src/player/brain_world.rs
scripts/world/player_needs_manager.gd → src/player/needs_manager.rs
```

---

## 三、Bevy 实现架构

```
PlayerPlugin
├── PlayerBrain Resource          // 运行时标签 + 感知缓存
├── PlayerNeeds Resource          // SDT 三维值
├── PlayerIntention Resource      // 当前目标 + 优先级
├── PlayerTask Resource           // 当前执行任务的状态机
└── systems:
    ├── sync_player_tags          // 每 tick 更新感知
    ├── compute_affordances        // 计算可供性
    ├── evaluate_needs            // 计算 SDT 需求
    ├── select_intention          // BDI 选择长期目标
    ├── run_task_fsm              // 执行任务状态机
    └── display_mind (egui)       // 右侧面板显示玩家思维气泡
```

### 集成方式

`main_tick` 中 `tick_entity` 的 player 分支配发到 `PlayerPlugin`：

```rust
if entity.type_name == "player" {
    // PlayerPlugin handles player tick
    return;
}
```

玩家卡不经过 `BehaviorRegistry`——走独立的 `PlayerPlugin`。生态卡行为不受影响。

---

## 四、断言（Phase 5 新增 15 条）

对标 Godot `unit_test_cases.gd` 中 player 相关用例：

| # | 断言 |
|----|------|
| 1 | 玩家空手时可觅食 |
| 2 | 玩家持矛时可狩猎 |
| 3 | 玩家饥饿时优先觅食 |
| 4 | 狼在威胁范围内→玩家需求 autonomy 下降 |
| 5 | 附近有篝火→玩家安全感提升 |
| 6 | 玩家有材料时 craft_knife 可供性出现 |
| 7 | 已有刀时 craft_knife 可供性消失 |
| 8 | 生火任务链：捡石头→砸碎石→生火 |
| 9 | 烹任链：生肉→篝火→熟肉 |
| 10 | 建棚链：木头+草→草棚 |
| 11 | 任务被威胁打断：猎狼近身→逃跑 |
| 12 | 任务状态机：plan→move→pickup→move→drop→act→done |
| 13 | 失败记忆冷却：同任务不久重复试 |
| 14 | 绝对前提：威胁 > 生火 > 觅食 |
| 15 | 玩家卡不在 BehaviorRegistry 中 tick（走独立 PlayerPlugin） |

---

## 五、验收

- `cargo test`：158+ PASS（143 现有 + 15 新）
- `cargo run --release`：玩家卡能动——觅食、生火、砍树、躲狼
- 右侧 egui 面板显示玩家当前目标（"我想生火""我在捡石头"）
- 玩家卡拖拽可中断 auto-plan，松手后恢复自主行为

## 六、不做

- 经营层（营地/贸易/菇棚）——仍延后
- 手动控制 UI——Phase 5 只做 AI 自主行为
- 复合弓/盐业等 v4.x 内容

## 约束

- 不碰 M1-M4 逻辑层
- 不碰 Observer 调度
- 143 现有断言不降
- 新增 15 条 player 断言
- 记 FIX_LOG
