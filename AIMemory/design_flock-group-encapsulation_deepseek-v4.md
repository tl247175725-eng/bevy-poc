# 族群逻辑封装设计

**状态**：设计预案，待 Observer 落地后推进
**日期**：2026-06-06

---

## 核心思路

单卡有单卡标签。同标签卡 ≥ N 只 + 满足合并条件 → 自动合并为族群卡。族群卡是一张卡，群体 tick。条件破坏 → 个体离群。

```
100 只羊 → 满足条件 → 1 张 sheepFlock(is_group, count=100)
           狼进入恐惧范围 → 2 只离群 → sheepFlock(count=98) + 2 只 sheep
           羊群 count < N → 拆回全部单卡
```

## 标签体系（不改动）

- 单羊：`sheep(flocking, herbivore, largePrey, ...)`
- 羊群：`sheepFlock(flocking, herbivore, largePrey, is_group, ...)` + `count: i32`
- WorldRules 判定时只认标签——不管羊群还是单羊，`herbivore+largePrey` → 被狼猎

## 合并条件

- 同类卡 ≥ 阈值（如 5 只）
- 在合并半径内（如相邻 3 格）
- 无威胁在恐惧范围内
- → 引擎自动生成 `sheepFlock`，移除源卡

## 离群条件

- 威胁进入恐惧范围 → N 只离群
- 群体饥饿 > 阈值 → 部分散开觅食
- 繁殖溢出 → 新生个体边缘独立
- count < 阈值 → 全拆

## 性能

tick 成本从 O(卡数) → O(群数)。10 万羊 ≈ 几十张群卡，0.005ms 级别。

## 与现有架构关系

不新增系统。`is_group` 标签让 BehaviorRegistry dispatch 到批量 tick。`flocking` 标签同时控制繁殖门槛和合群资格。WorldRules 不区分单卡/群卡——只看标签。
