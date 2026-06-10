# 砸击系统修复

**From**: deepseek-v4
**To**: cursor
**Date**: 2026-06-10
**Priority**: P0（砸击手感、猎物冻结、配方失败）

## 架构计划
砸击是视觉层操作，不应阻塞模拟层。修复三个独立问题。

## 架构反馈
公理层 transform(Smash) 和 tick_reactive 之间缺少解耦——砸击不应锁住 AI。

## 智能验收
- 断言：狼攻击时羊仍然可以 flee
- 断言：拖动卡牌碰目标 → 卡牌停在目标边缘，不穿透
- 断言：石+石 2 次砸 → 碎石生成

---

## 修复 1：猎物不冻结

`hunt_kill` 中设置 `hunt_cooldown` 后不阻塞猎物的 `tick_reactive`。当前逻辑可能在砸击过程中跳过了猎物的 AI tick。

修法：`hunt_kill` 只修改 HP 和 cooldown，不改变猎物 ecology_state。猎物自己的 `tick_reactive` 在下一 tick 应该触发 `flee`（因为 predator 就在邻格）。

## 修复 2：砸击不穿透目标

在 `detect_drag_smash` 或 `update_drag_follow` 中，当拖动卡接近目标卡时，将拖动位置 clamp 到目标卡的外边缘。拖动卡不应穿过目标。

```
目标外边缘 = 目标中心 ± CARD_SIZE/2
拖动卡位置 = clamp(鼠标位置, 目标边缘 - 砸击距离)
```

触发砸击时，拖动卡应短暂"弹回"，然后可以再次接近触发下一砸。

## 修复 3：石+石 → 碎石

检查 `SmashRecipe` 或 `try_relation` / `RecipeBook` 中石+石的配方定义。砸击触发后是否正确调用了产品生成。

预期流程：
1. 左键拖石 → 碰另一石 → 红圈"砸"→ smashes_accumulated = 1
2. 拉开再碰 → 红圈"2" → smashes_accumulated = 2 → 触发配方 → 碎石生成

## 涉及文件

| 文件 | 改动 |
|---|---|
| `src/interaction/smash.rs` | 砸击穿透修复 + 配方触发 debug |
| `src/systems/tick_reactive.rs` | hunt_kill 不锁猎物 AI |
| `src/ui_interaction.rs` | 拖动碰撞 clamp |

## 验收
- `cargo check` 0 错误
- `cargo test --release` 全 PASS
- `cargo run --release -- --smoke-test` SMOKE: PASS
- 拖石碰石 ×2 → 碎石出现
- 狼追羊 → 羊仍在逃跑
- 拖动物碰目标 → 停在边缘不穿透
