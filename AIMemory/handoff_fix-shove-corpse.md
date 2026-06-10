# 挤开门禁 + 尸体/肉修正

**From**: deepseek-v4
**To**: cursor
**Date**: 2026-06-10
**Priority**: P0

## 架构计划
全部标签驱动。挤开加 `being` 检查。尸体内型用 `corpse_type` 标签统一。砸尸体变肉用 smash 系统已有逻辑。

## 架构反馈
无。

## 智能验收
- 狼窝不被挤开
- 狐死 → 狐尸 → 砸 ×2 → 狐肉
- 不再出现 sheepCorpse fallback

---

## 修复 1：挤开只限生物

`movement.rs` `try_shove` / `try_yield_and_enter`：检查 blocker 是否有 `being` 标签。无则不挤。

## 修复 2：corpse_type 补全

`card_defs.ron` 给所有动物加 `corpse_type:` 标签：

```
wolf → corpse_type:wolfCorpse
fox → corpse_type:foxCorpse
rabbit → corpse_type:rabbitCorpse
pheasant → corpse_type:pheasantCorpse
waterBuffalo → corpse_type:waterBuffaloCorpse
fieldMouse → corpse_type:fieldMouseCorpse
bambooRat → corpse_type:bambooRatCorpse
fish → corpse_type:fishCorpse
```

同样 `meat_product:` 标签补全：
```
fox → meat_product:foxMeat
rabbit → meat_product:rabbitMeat
pheasant → meat_product:pheasantMeat
waterBuffalo → meat_product:buffaloMeat
deer → meat_product:deerMeat
fieldMouse → meat_product:mouseMeat
wolf → meat_product:wolfMeat
fish → meat_product:fishMeat
```

## 修复 3：砸尸体 → 肉

当前 `hunt_kill` 直接生成肉卡。改为：生成 `xxxCorpse` 卡。玩家/AI 左键砸尸体 2 次 → 尸体变为对应肉卡（数量=meat_yield）。

`smash.rs` 中尸体被砸 → 累计 2 次 → `remove_entity(corpse) + spawn(meat_product, meat_yield)`。

## 涉及文件

| 文件 | 改动 |
|---|---|
| `src/systems/movement.rs` | try_shove/try_yield 加 being 检查 |
| `assets/card_defs.ron` | corpse_type + meat_product 补全 |
| `src/interaction/smash.rs` | 砸尸体 ×2 → 肉 |
| `src/systems/tick_reactive.rs` | hunt_kill 改为生成尸体而非直接肉 |

## 验收
- 狼窝不被挤
- 每个动物的尸体和肉类型匹配
- 砸尸体两次变对应数量的肉
