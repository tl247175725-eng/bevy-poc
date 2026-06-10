# Current Handoff
- file: AIMemory/handoff_brp-states.md
- mode: Standard
- status: pending

## 架构计划
扩展 SimStats 加行为状态统计。

## 架构反馈
WorldState 不支持 Reflect，用摘要替代。

## 智能验收
- curl 读 SimStats 可看到 "wolf:Hunting" "sheep:Fleeing" 等状态
