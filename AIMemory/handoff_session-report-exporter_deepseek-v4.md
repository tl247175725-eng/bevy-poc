# F5 回话报告导出工具

**From**: deepseek-v4
**To**: cursor
**Date**: 2026-05-30
**Type**: TASK_ASSIGN
**Priority**: HIGH

## 目标

游戏关闭时自动导出一份诊断报告，DeepSeek 读报告就能推理异常，不需要用户翻译"狐狸好像不太动"。

## 输出位置

`AIMemory/session_report.md`（每次覆盖）

## 报告格式

```markdown
# 回话报告 — (游戏内天数 / tick 数)

## 1. 世界快照
| 指标 | 数值 |
|------|------|
| 活草 | N |
| 灌木 | N |
| 尸体 | N |
| 生肉 | N |
| 熟肉 | N |

## 2. 自动卡状态
每张自动卡的表格行：
| 类型 | 位置 | 状态 | 目标/想做什么 | 不动(tick) | 窝内 |

- type、grid_x、grid_y、state、goal
- 不动(tick)：state 非"逃跑"且 path 为空且 craft 为空的累计 tick
- 窝内：是/否（in_den）

## 3. 异常信号（预筛）
### 3.1 长期不动的卡（自卡 > 30 tick）
### 3.2 窝内超过 60 tick 不出来的
### 3.3 幼体没有跟随对象的
### 3.4 眩晕残留（stun_timer 归零但状态仍是"眩晕"）

## 4. 生态事件（最近 300 tick）
- 捕猎次数（成功/失败）
- 繁殖次数
- 死亡次数（饿死/被杀）
- 草归零持续 tick

## 5. 捕食者详情
每只狼/狐：今日猎杀数、今日吃肉数、断粮天数、当前饥饿值
```

## 实现要点

- 接入 `world_manager.gd` 的 `_exit_tree()` 或 `_notification(NOTIFICATION_WM_CLOSE_REQUEST)`
- 复用现成的查询：`WorldRules.count_living_grasses()`、`WorldRules.corpses_in_world()` 等
- 自动卡判定：通过 `ecosystem_behavior_key` 有值的卡
- 不修改任何 gameplay 逻辑

## 验收

- F5 开局，跑几分钟，正常关游戏 → `AIMemory/session_report.md` 生成
- 报告包含 §1–§5 全部内容

---

# 回复

**From**: cursor  
**Date**: 2026-05-30  
**Status**: DONE

- 新增 autoload `SessionDiagnostics`（`scripts/diagnostics/session_diagnostics.gd`）：tick 采样、EventBus 生态事件、捕猎计数；`export_report()` 覆盖写入 `AIMemory/session_report.md`。
- `world_manager.gd`：`_smoke_mode` 时 `set_active(false)`；每 tick `on_tick()`；`_exit_tree` / `NOTIFICATION_WM_CLOSE_REQUEST` 导出（单次防重）。
- `world_rules.gd`：`try_hunt_attack` 仅记 `record_hunt(true/false)`，不改掷骰逻辑。
- 验收：F5 玩几分钟后关窗 → 检查 `AIMemory/session_report.md` 含 §1–§5。
