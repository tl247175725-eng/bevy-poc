# 行为调度心跳：检测自动卡 tick 是否被执行

**From**: deepseek-v4
**To**: cursor
**Date**: 2026-05-30
**Priority**: HIGH

## 为什么需要这个

狐狸六轮追不出原因——§7 诊断完全为空，但我们始终无法确认 `FoxBehavior.tick` 到底有没有被 `EcosystemTickRegistry` 调用。此后每张新卡都可能踩同一个坑。

## 改法

### 1. 心跳记录

每个 behavior tick 的入口最前面加一行：

```gdscript
SessionDiagnostics.heartbeat(card)
```

记录 `{card_type: "fox", grid: (x,y), tick: 当前tick, real_time: 当前时间}`。

### 2. 报告新增 §8 心跳检测

```
## 8. 行为调度心跳
| 类型 | 位置 | 最后 tick | 距今(秒) | 状态 |
|------|------|----------|---------|------|
| fox | (12,15) | 0 | 300 | ⚠️ 从未 tick |
| wolf | (5,20) | 432 | 0 | ✅ |
```

- 距今 > 5 秒 → ⚠️ 可能卡死
- 从未 tick（last=0）→ ❌ 调度未生效

### 3. 集成

在 FoxBehavior、WolfBehavior、SheepBehavior、DeerBehavior、RabbitBehavior、FieldMouseBehavior、TaoyuanBehavior 的 tick 入口各加一行 `heartbeat` 调用。

## 验收

- 报告 §8 列出所有自动卡的 tick 心跳
- 正常卡显示 ≤1 秒
- 如果狐狸重新接入，能立即看到它是否被调度
