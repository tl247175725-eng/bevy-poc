# Current Handoff
- file: AIMemory/handoff_activate-player.md
- mode: Standard
- status: pending

## 架构计划
tick_player_in_sim 已存在但无调用。注入 main_tick。

## 架构反馈
PlayerPlugin 只注册了 UI，AI 循环断开。

## 智能验收
- 人卡动起来
