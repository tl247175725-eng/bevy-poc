# UI 容纳关系与外观 — 执行 handoff

**From**: deepseek-v4  
**To**: cursor  
**Date**: 2026-06-02  
**Priority**: HIGH  
**建议模式**: Fast（交互实现 + L0）

---

## 目标（PROTOCOL §3.1b）

容纳关系在地图与信息栏双向可见：**被容纳者不可见时，宿主必须有入口标记；宿主被选中时，容纳内容可逐项查看并可跳转详情**。左键选中仅展示，不触发内容物生态行为。

---

## 现状（Cursor 合入前请先读代码）

| 项 | 状态 | 位置 |
|----|------|------|
| 狼窝/腐殖 overlay + 地面角标 窝/肥 | ✅ | `world_rules_ui.gd`、`card_base._refresh_corner_tag` |
| 狐窝卡角标 狐 | ✅ | `ground_corner_marker_for_card` |
| 灌木藏有 / 草伏 角标 藏/伏 | ✅ | `_cover_entry_marker` |
| 点草皮 → resolve 狼窝面板 | ✅ | `ui_resolve_selection_card` |
| 选中绿框 vs 面板卡解耦 | ✅ | `world_manager.select_card` |
| 容纳段 藏有/窝内/地下狼窝 计数文案 | ✅ | `housing_lines` / `build_den_lines` |
| 狐占灌木 → 删灌木 spawn foxDen | ✅ | `ecosystem_manager._convert_bush_to_fox_den` |
| §3.1b 全卡/全格审计 | ✅ | `ui_visual_coverage_audit.gd` |
| 格子选中有 overlay 时整页走 den/humus 面板 | ✅ | `build_cell_lines` 早退 |
| 容纳条目【】+ 点击跳转 | ❌ | 待做 |
| 草棚显示棚内玩家/存货 | ❌ | 待做 |
| 格子标题：有 overlay 但仍走「地形+地下」路径时改标题 | ⚠️ | 见 §一 |
| 选中含容纳宿主不扰动内容物 | ⚠️ | 见 §四，需 F5 复现后修 |

---

## 一、格子标题优先 overlay

**需求**：空地选中时，若该格存在 `cell.overlay`（狼窝/腐殖），**首行标题**用 overlay 的 `display_name`（狼窝 / 腐殖土），不要仍写「河岸」「荒地」等纯地形名。

**现状**：`build_cell_lines` 对狼窝/狐窝/腐殖已 `return build_den_lines` / `build_humus_lines`（标题正确）。若仍存在「地形标题 + 地下摘要」双轨混用，统一为：

1. 有 overlay → 标题 = `overlay.card_def.display_name`（或狼窝/狐窝专用标题）  
2. 无 overlay → `cell_display_title`  

**验收**：同格 grass + wolfDen overlay，点空地 → 首行「狼窝」，身份/窝内/砸毁完整；无英文键。

---

## 二、狐窝外观（灌木转化）

**需求**：狐狸占灌木后，地图上**只有一张狐窝卡**，不得灌木与狐窝叠层。

**现状**：`_convert_bush_to_fox_den` 已 `remove_card(bush)` + `spawn_card_at("foxDen")`。若 F5 仍见灌木图标，查 `refresh_face` / 残留引用 / 未从 `GameState.cards` 移除。

**验收**：筑窝后该格仅显示狐窝 icon/名，角标「狐」，选中为狐窝面板。

---

## 三、容纳关系可视化（本 handoff 核心）

### 3.1 信息栏条目格式

在 **容纳：** 段（`housing_lines` / `build_den_lines` 窝内行）列出可识别条目，每条用 **【】** 包裹，含性别时沿用 `sex_bracket` 规则（仅【公】【母】，禁止【无】）：

| 宿主 | 条目示例 |
|------|----------|
| 狼窝 | 【公狼】【母狼】【幼狼】+ 储肉行保留 |
| 狐窝 | 【公狐】【母狐】【幼狐】+ 储肉/清腐 |
| 灌木/活草 | 【田鼠】【幼鼠】…（`cover_occupants_of`） |
| 草棚 | 【玩家】（夜间 `sleeping` 且邻接该 hut）；棚内存货格物品显示 display_name |

计数摘要（`窝内：成年狼×2`）可保留，但 **须另有可点击的【】条目列表** 或把计数行拆成逐条【】。

### 3.2 点击跳转

- 条目点击 → `world_manager.select_card(该生物)`，信息栏切换为该卡 `build_card_lines`  
- 被容纳卡 `visible=false` / `in_den` 时仍可通过条目选中（不要求恢复可见）  
- 实现建议：`game_ui` 用 `RichTextLabel` 的 `[url=card_id]【公狼】[/url]` + `meta_clicked`，或 `HBoxContainer` 小按钮；**逻辑放 UI 层**，查询走 `WorldRules`（勿散落 `card_type ==`）

### 3.3 草棚容纳（当前遗漏）

- 玩家：`_should_player_sleep_now` 为真且 `is_neighbor(hut)` → 该 hut 的容纳段含【玩家】  
- 存货：`CampHelpers.in_hut_storage_xy` / 同格堆叠 → 列出棚内物品中文名  
- 新增规则查询建议：`WorldRules.ui_hut_occupant_labels(hut) -> PackedStringArray`（或等价）

### 3.4 地图入口标记（已大部分完成，勿回退）

优先级保持：`狐` → `窝/肥` → `藏/伏`（`ground_corner_marker_for_card`）。

---

## 四、选中不触发行为

**现象（策划反馈）**：左键点灌木 → 田鼠逃出灌丛。  
**要求**：左键点 **含容纳的宿主**（灌木、狼窝格地面卡、草棚等）→ **仅选中 + 信息栏**，不调用 `exit_cover`、不开始拖拽扰动、不触发捕食/逃跑 tick。

**排查点**：

- `world_manager.handle_pointer_gui`：`select_card` 后 `start_drag` 对 `organize.locked` 灌木应早退，确认无移动探索碰撞  
- `_card_at` 是否误命中隐藏田鼠而非灌木  
- 生态 tick 是否把「玩家关注该格」当威胁（FieldMouseBehavior 躲狼/躲人）  

**建议**：对 `is_cover_plant` / `is_wolf_pack_den` 地面代理卡，指针 **press** 仅 `select_card`，**drag 在移动阈值后**再 `start_drag`；或宿主不可拖时绝不 `start_drag`。

**验收**：灌丛内藏鼠，左键点灌木 → 信息栏显示藏有，田鼠仍在灌丛、state 不变。

---

## 涉及文件

| 文件 | 改动 |
|------|------|
| `scripts/ui/selection_info_panel.gd` | 容纳条目结构化（id + 显示名）；草棚/query |
| `scripts/ui/game_ui.gd` | 可点击容纳行、跳转 `select_card` |
| `scripts/core/world_rules/world_rules_ui.gd` | 窝/灌/棚 occupant 查询 API |
| `scripts/core/world_rules.gd` | facade 转发 |
| `scripts/world/world_manager.gd` | 选中/点击与 drag 解耦（§四） |
| `scripts/world/ecosystem_manager.gd` | 仅当 §二回归失败时查灌木转化 |

---

## 测试

1. L0 不降；补 `_test_ui_containment`（或扩 `_test_selection_info_panel`）：  
   - 狼窝/灌木/草棚容纳【】文案  
   - 点击条目后 `selected_card` 为目标生物  
2. `UiVisualCoverageAudit` 仍 PASS  
3. 记 `AIMemory/fix-log.md`；`work.log` 一行 `WORK_END`

---

## 约束

- 不碰 `capability_behavior_pipeline`、`ecosystem_behavior_key`、封闭生态模式  
- 规则查询进 `WorldRules` / `world_rules_ui`，behavior 不写 `card_type ==`  
- 零英文：面板不出现 `capability.` / 原始 tag 键名  

---

## Cursor 回复

**状态**：已实现（2026-06-02）

### 改动摘要

- `world_rules_ui.gd`：`ui_containment_entries` / `ui_pointer_select_only` / `ui_cell_overlay_title`；狼窝·狐窝·灌丛·草棚容纳【】条目（含性别）；草棚 3×3 域内存货 + 夜间 sleeping 玩家。
- `game_ui.gd`：容纳段 `LinkButton` 点击 → `world_manager.select_card_by_instance_id`。
- `world_manager.gd`：`_card_at` 忽略不可见/灌丛内鼠；不可拖或 `ui_pointer_select_only` 时左键只选中不 `start_drag`。
- `selection_info_panel.gd`：空地标题优先 overlay；灌木「藏有」改由可点击条目承担。

### L0

**887** assertions PASS（含 `_test_ui_containment_handoff`）。

### F5 检查项

1. 灌木藏鼠 → 点灌木 → 信息栏【田鼠】可点 → 跳转田鼠详情，鼠仍在灌丛。  
2. 狼窝格点草皮/空地 → 标题「狼窝」、窝内【公狼】等可点。  
3. 狐占灌木后仅狐窝外观。  
4. 夜间玩家在草棚旁睡觉 → 选草棚见【玩家】与棚内货物。
