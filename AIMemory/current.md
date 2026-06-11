# Current Handoff
- file: AIMemory/handoff_sleep-optimize.md
- mode: Standard
- status: completed

## 架构计划
Entity 加 sleep_until_tick，无活跃 drive 的实体跳过 tick_reactive，外部事件唤醒。

## 架构反馈
不改变行为逻辑，纯性能优化。

## 智能验收
- idle 实体跳过 tick
- smoke test 行为不变