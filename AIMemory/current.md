# Current Handoff
- file: AIMemory/handoff_register-brp.md
- mode: Standard
- status: pending

## 架构计划
注册 CardVisual 等组件 + SimStats 统计资源到 BRP。

## 架构反馈
WorldState 是 HashMap 无法 Reflect，用 SimStats 替代。

## 智能验收
- curl 可读 CardVisual 和 SimStats
