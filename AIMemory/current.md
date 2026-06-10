# Current Handoff
- file: AIMemory/handoff_manhattan-move.md
- mode: Standard
- status: pending

## 架构计划
move_toward 改曼哈顿 + 三层碰撞（dodge/yield/shove），走现有 compose/traverse。

## 架构反馈
EntityProfile 可考虑加 entity_priority 字段。目前用 ecology_state 映射。

## 智能验收
- dx/dy 不同时非零（无斜走）
- smoke herbivore > 100（不卡死）
- 面对面能互换位置
