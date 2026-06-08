# session_report 新增 §8：玩家大脑快照

**From**: deepseek-v4
**To**: cursor
**Date**: 2026-06-04

---

`session_report.md` 新增 §8，在游戏关闭时自动写入。格式：

```
## 8. 玩家大脑
| tick | 自主 | 胜任 | 关联 | 意图 | 可供性 | 状态 |
|------|------|------|------|------|--------|------|
| 86 | 45 | 60 | 20 | fire | craft_knife, hunt, forage | 生火 |
| 150 | 80 | 70 | 80 | shelter | build_hut, collect_wood | 搭棚 |
| 330 | 75 | 65 | 85 | hunger | hunt, forage | 猎兔 |
| 500 | 90 | 85 | 90 | — | forage, collect_wood | 休息中 |
```

数据来源：
- 自主/胜任/关联：`player_needs.evaluate()` 的返回值
- 意图：`player_intention._intention.key`，无为 "—"
- 可供性：`player_affordance.detect()` 的 key 列表（取前 3 个）
- 状态：玩家卡的 `state` 字段

**输出规则**：
- 不记录每 tick——仅在意图变化、需求巨变（任一变化 > 20）、状态切换时记录
- 最多保留 50 条，最早的被新条目覆盖
- 报告末尾输出最新 20 条

**读取**：`player.get_meta("player_brain")`，若 meta 为空则显示「大脑未初始化」。

---

## 涉及文件

`scripts/diagnostics/session_diagnostics.gd` — 新增 `_capture_player_brain()` + §8 写入

L0 不降。Fast。
