# 设计对齐整治：藏=容纳 + 玩家 AI 完整激活

**From**: deepseek-v4
**To**: cursor
**Date**: 2026-06-11
**Priority**: P0（设计文档已有，代码实现偏离设计）

**参考设计文档**：
- `E:\桌面\方寸商国：桃花源记\docs\design\core\设计规则圣经_v0.3.txt`
- `E:\桌面\方寸商国：桃花源记\docs\design\core\世界运转共识_v0.6.md`
- `E:\桌面\方寸商国：桃花源记\docs\design\core\意向系统与世界演化_v0.3.md`
- `E:\桌面\bevy-poc\AIMemory\design_flock-system_deepseek-v4.md`
- `E:\桌面\bevy-poc\AIMemory\design_interaction-system_deepseek-v4.md`

## 架构计划
基于现有架构修复，不引入新系统。藏复用容纳体系（in_tree/in_pool 模式）。玩家 AI 激活现有五层系统（compute_affordances → evaluate_needs → select_intention → ActionRunner → move/use）。

## 架构反馈
`in_cover` 字段和渲染过滤已存在但未完整覆盖所有路径。PlayerMind 代码完整但条件链在 headless 中断。

## 智能验收
- 兔在草上 → 消失，草显紫藏标
- 玩家自主生存：觅食→吃饱→找材料→做工具→建草棚
- 玩家遇狼→逃跑→狼离开→恢复行动

---

## P0-1：藏 = 容纳（走 in_tree 同路径）

设计文档 §2.2/§2.3：覆盖层可叠加、实体层有碰撞可拖拽。藏 = 实体进入覆盖层内部 → 实体不占格、不可见、显示容纳标记。

**修复 `try_hide_in_cover_at`**：
```rust
fn try_hide_in_cover_at(world, id, hide_tag) {
    // 1. 实体设 in_cover + host_cover_id
    // 2. cell_composition 中该实体不再占用（vacate_entity）
    // 3. 渲染层过滤 in_cover（已有）→ 实体卡消失
    // 4. 覆盖卡显示紫藏标（已有 hide_visual.rs）
}
```

**退出容纳**：
- 覆盖物被破坏（HP=0）→ 释放所有 in_cover 实体到相邻空格
- 威胁消失 + 饥饿 → 实体主动退出容纳

---

## P0-2：玩家 AI 完整激活

设计文档：需求驱动 → 物理一致性 → 最小满足 → 风险优先。

当前 Rust 代码已有完整骨架。问题是 headless 模式下资源检测不完整。

### 觅食
已有 `satisfy_hunger`。扩展目标：berry → bush → grass → foodSource。已有逻辑，只需确认 map 上有可觅食物。

### 制造
`plan_craft_knife` 和 `plan_build_hut` 已存在。检查条件：
- 篝火：需要 twig + stone → `craft_relation("twig","stone")`
- 草棚：需要 twig + grass → `craft_hut_relation`

**修复**：确保玩家附近有 twig/stone/grass 时，affordance 系统正确触发 craft intention。

### 基础生存链条
```
饥饿 → forage（觅食）→ 找 berry/bush/grass
↓↓ 吃饱后
安全 → build_hut（建草棚）→ 需要 twig + grass
↓↓ 有草棚后
工具 → craft_knife（做石刀）→ 需要 stone + twig
↓↓ 有工具后
狩猎 → 用刀猎小型动物
```

**这个链条的每一步在 Godot 设计中都已存在。Rust 代码中：
- `satisfy_hunger` ✓（已有）
- `plan_craft_knife` ✓（已有）
- `plan_build_hut` ✓（已有）
- `flee_from_threat` ✓（已有）

**缺失的是**：这些函数依赖 `InteractionState` 和 `SimEventQueue`——headless 模式下为空。需要让它们在 headless 模式下直接调用 `move_toward` / `world.move_entity` / `apply_smash_hit` 而不依赖交互。

### 修复方案
`ActionRunner::tick` 接收 Option 参数：headless 时 `interaction` 和 `events` 为 None。内部所有函数检查 None 时走直接操作路径。

```rust
pub fn tick(world, player_id, mind, delta, interaction, events) {
    // ...
    match mind.top_desire.as_str() {
        "craft_knife" => {
            plan_craft_knife_headless(world, player_id, mind);
        }
        "forage" => {
            satisfy_hunger_headless(world, player_id, mind);
        }
        "build_hut" => {
            plan_build_hut_headless(world, player_id, mind);
        }
    }
}
```

## 涉及文件

| 文件 | 改动 |
|---|---|
| `src/systems/tick_reactive.rs` | 藏：vacate_entity on hide |
| `src/player/action_runner.rs` | headless 模式直接操作 |
| `src/player/survival_tasks.rs` | headless 版觅食/逃跑 |
| `src/player/tool_tasks.rs` | headless 版制造 |
| `src/player/shelter_tasks.rs` | headless 版建草棚 |
| `src/card_visual.rs` | 确保 in_cover 渲染过滤完整 |

## 验收
- `cargo check` 0 错误
- 兔在草格 → 实体消失，草显紫藏标
- 草被破坏 → 兔出现在邻格
- 玩家自主完成：觅食→建草棚→做石刀
