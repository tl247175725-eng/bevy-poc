# 封存非生态卡

**From**: deepseek-v4
**To**: cursor
**Date**: 2026-06-05

---

将以下卡从初始生成和运行时激活中移除。卡定义保留在 card_db.gd 和 CARD_CAPABILITIES 中不动——只停用 spawn，不删数据。

## 封存清单

| 卡 | 封存内容 |
|----|---------|
| 旅人 traveler | `_spawn_initial_cards` 中不生成。`population_manager._update_traveler_attraction` 跳过 |
| 蘑菇农 mushroomFarmer | `_spawn_initial_cards` 中不生成。`population_manager._update_farmer_spawn` 跳过 |
| 桌子 table | `_spawn_initial_cards` 不生成 |
| 铜钱 coin | `_spawn_initial_cards` 不生成 |
| 铜块 copperBlock | `_spawn_initial_cards` 不生成 |
| 铜饰 copperCraft | 合成关系保留，但不触发生成 |

## 不碰

- card_db.gd 定义不动
- CARD_CAPABILITIES 定义不动
- 生态管线/封闭模式不动

## 原因

这些卡属于玩家经济扩展层，当前封闭生态不需要它们。生态自洽后再激活。

L0 不降。记 fix-log。
