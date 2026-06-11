# Current Handoff
- file: AIMemory/handoff_fix-animal-oscillation.md
- mode: Standard
- status: ready

## 架构计划
草食动物 fed_today 锁死 Seek → 无事可做。全部动物无驱动时统一 wander（四方向随机）。Flock 加重心死区。Flock state 修正。草不被动物踩毁。

## 架构反馈
fed_today 粒度太粗，480 tick 无行为太久。Flock 缺分离力。wander 偏斜已修。

## 智能验收
- 动物吃完后不空转睡眠
- 群不 2 格振荡
- Flock 重心重合不移动
- 草地不被路过踩毁
- cargo test + cargo build PASS
