# 玩家 AI headless 完整激活

**From**: deepseek-v4
**To**: cursor
**Date**: 2026-06-11
**Priority**: P0（玩家卡完全不行动）

## 架构计划
玩家 AI 代码链完整（tick_brain → ActionRunner → move_toward/flee_from/satisfy_hunger）。但条件不满足：饥饿增长太慢、可觅食物品缺失、威胁感知不贯穿。修复条件而非重写逻辑。

## 架构反馈
PlayerMind 依赖交互输入（berry、prey 等类型查找），headless 模式下卡牌类型对不上。需要扩展觅食/狩猎目标类型。

## 智能验收
- 玩家主动移动、觅食、躲避
- BRP 的 SimStats 中玩家显示 SeekingFood/Fleeing 状态

---

## 修复

### 1. 饥饿驱动移动
`starve_days` 字段已有。饥饿时（未进食 1 天+）`tick_player_world` 强制触发觅食：
- 找最近的 bush/grass/foodSource → move_toward
- 邻格时 eat（同草/灌木逻辑）

### 2. 威胁驱动逃跑
让 player 的 `tick_player_world` 同样享受 reactive 的感知：`query_near_filtered(x,y,"predator",6,id)`。有 predator 时 → flee。

### 3. 玩家显示在 state_breakdown
`sync_sim_stats` 中添加 player 的 ecology_state（与标准生态同步，设置 `needs_grazing_tick`）

---

## 涉及文件

| 文件 | 改动 |
|---|---|
| `src/player/needs_manager.rs` | tick_player_world 加饥饿觅食+威胁逃跑 |
| `src/player/survival_tasks.rs` | satisfy_hunger 扩展目标：grass/bush/foodSource |
| `src/player/action_runner.rs` | 无改动（已有逻辑/条件需被满足） |
| `src/player/behavior.rs` | tick_brain 设置 player ecology_state |

## 验收
- `cargo check` 0 错误
- 启动游戏 30 秒内玩家有位移
- BRP state_breakdown 含 player
