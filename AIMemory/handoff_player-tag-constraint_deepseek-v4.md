# 玩家卡标签约束 + 面板改造

**From**: deepseek-v4
**To**: cursor
**Date**: 2026-06-03

---

## 一、玩家猎杀约束

玩家当前走 `HUNT_PROFILE_TOOL`，无日猎上限，无冷却。改为跟狐狸一样的约束体系。

### 改法

1. `_hunt_target_allowed_by_profile` — `HUNT_PROFILE_TOOL` 分支：
   - 新增日猎检查：`dailyKills >= TOOL_HUNTER_MAX_KILLS_PER_DAY`（常量 = 2）→ 返回 false
   - 其余逻辑不动

2. `execute_capability_hunt_attack`：
   - 猎手为 tool_hunter 时，成功击杀后递增 `dailyKills`
   - `player_card.gd` 已无 `dailyKills` 字段，走 `actor.get("dailyKills")` get/set，同 fox 机制

3. `reset_ecology_feed_flag`：
   - tool_hunter 日重置时 `dailyKills = 0`

## 二、玩家信息面板标签化

### 改法

1. `game_ui.gd` `render_selected()`：
   - 当前：`ui_uses_player_panel(card)` → 走 `_render_player_panel`
   - 改为：`ui_uses_player_panel(card)` → 先调 `build_card_lines(card)` 输出标准标签行（身份/能力/状态/HP），再追加玩家专属信息（饥饿值、装备工具、今日猎杀数）

2. `_render_player_panel` 重构为：
   ```
   玩家
   身份：生物 · 杂食 · 火缘 · 就地取材
   能力：捕猎 · 觅食 · 制作 · 用工具 · 携物
   状态：待机  目标：巡林
   饥饿 46/100  今日猎杀 0/2
   工具 石刀 · 长矛
   HP 6/6
   ```

3. 玩家标签从 `card_db.gd` + `CARD_CAPABILITIES` 获取，与其他卡走同一套 `identity_labels` + `capability_labels` 渲染。玩家专属信息（饥饿、工具列表）在标准行后追加。

---

## 涉及文件

| 文件 | 改什么 |
|------|--------|
| `world_rules.gd` | TOOL_HUNTER_MAX_KILLS_PER_DAY + _hunt_target_allowed_by_profile tool_hunter 分支 |
| `world_rules.gd` | execute_capability_hunt_attack tool_hunter dailyKills 递增 |
| `world_rules.gd` | reset_ecology_feed_flag tool_hunter dailyKills 清零 |
| `game_ui.gd` | _render_player_panel 改为 build_card_lines + 追加 |
| `card_db.gd` | 玩家标签更新为 omnivore / tool_dependent / fire_bond / opportunistic |

## 约束

- L0 不降
- 不碰管线/签名/封闭模式
- 记 fix-log
