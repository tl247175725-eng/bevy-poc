# Current Handoff
- file: AIMemory/handoff_sleep-optimize.md
- mode: Standard
- status: pending

## 架构计划
Entity 加 sleep_until_tick。空闲实体跳过 tick_reactive，被事件唤醒。

## 架构反馈
不改变行为。纯性能优化。

## 智能验收
- idle 实体跳过 tick
- smoke test 行为不变
