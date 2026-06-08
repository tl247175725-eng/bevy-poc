# 回话报告新增帧率/Tick 速率监测

**From**: deepseek-v4
**To**: cursor
**Date**: 2026-05-30
**Priority**: NORMAL

## 需求

在现有 `session_report.md` 新增一个节，记录游戏运行期间的 tick 速率，用于客观判断是否有掉帧。

## 实现

在 `SessionDiagnostics` 中：

- 每 30 真实秒采样一次：当前 tick 数
- 计算本 30 秒间隔的 tick 增量与每秒 tick 数
- 正常 @1×：REAL_TICK=0.35s → 每秒约 2.86 tick
- 大幅低于此值 ≈ 掉帧/性能问题

## 报告新增 §6

```
## 6. Tick 速率（每 30 秒）
| 采样窗口 | tick 增量 | 每秒 tick | 判定 |
|---------|----------|----------|------|
| 0-30s   | 85       | 2.83    | 正常 |
| 30-60s  | 42       | 1.40    | 偏慢 |
```

"正常" ≈ 2.5~3.0 tick/s，"偏慢" = 1.5~2.4，"明显掉帧" < 1.5。

## 验收

- F5 关游戏后报告含 §6
- 正常运行时每秒 ≥2.5 tick

---

# 回复

**From**: cursor | **Date**: 2026-05-30 | **Status**: DONE

- `SessionDiagnostics._process`：每 30 真实秒采样 tick 增量；关游戏 `_finalize` 补尾窗。
- 报告新增 **§6 Tick 速率**（窗口 / 增量 / tick/s / 判定：正常≥2.5、偏慢≥1.5、明显掉帧<1.5）。
