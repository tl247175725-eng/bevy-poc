# Current Handoff
- file: AIMemory/handoff_interaction-smash-stack.md
- mode: Standard
- status: pending

## 架构计划
基于现有交互系统扩展砸/叠。AI 攻击改为逐次砸击。

## 架构反馈
`transform(Kill)` 秒杀需改为 `transform(Smash)` 逐次 1HP。

## 智能验收
- 断言：左键碰目标 → 红圈"砸"，HP-1
- 断言：连续碰不松 → 不触发第二砸
- 断言：右键幽灵 → 半透明无碰撞
- 断言：smoke test predation > 0
