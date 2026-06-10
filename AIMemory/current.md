# Current Handoff
- file: AIMemory/handoff_fix-player-hide.md
- mode: Standard
- status: completed

## 架构计划
玩家 AI headless 走路 + 湿地 scale + 躲藏行为修正。

## 架构反馈
ActionRunner 在 headless 下直接 move_toward/flee_from。

## 智能验收
- 玩家自主移动
- 湿地扩至 3 环
- 兔/鼠逃入后停在草/灌木
