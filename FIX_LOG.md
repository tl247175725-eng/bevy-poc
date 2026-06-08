# FIX_LOG — Bevy 迁移

## 2026-06-07 Phase 0–5: Plugin 架构 + 渲染/交互/仿真/UI 迁移

- **Phase 0**: `SimPlugin` / `RenderPlugin` / `InputPlugin` / `UiPlugin`；`coords.rs` 统一坐标 API；删除 `bevy_ecs_tilemap`；`tests/coords_tests.rs` round-trip/zoom 断言
- **Phase 1**: `surface_label_with_stress`（狼穴/狐窝/河沟·稳紧低）；河压格色；`render/card_view.rs` 拖拽/幽灵 overlay
- **Phase 2**: 右键幽灵拖拽 + `try_ghost_drop` 合成/落格；左键点击采收；物理砸击保留
- **Phase 3**: `pathfinding.rs` A* + greedy-first `move_toward`；`behavior_registry.rs` 能力签名 dispatch；`tick_environment` 河弹/水上火/河岸草（`LIVING_GRASS_CAP`）；`count_living_grasses` 对齐 Godot；`tests/godot_parity.rs` 非经营用例
- **Phase 4**: `selection_info` 狼穴/狐窝/腐殖/水势/覆盖/池容纳计数；`ecology_log` → `SimEventQueue` → UI 日志流
- **Observer 深化**: bucket tick 降级（仅捕食者 patrol）；`BehaviorRegistry` → `EventRegistry`（Spawn/Move 事件链）；环境/繁殖/腐坏仍 FixedTimestep
- **Observer B（两刀）**: `sim_observer.rs` — `pending_events` 统一 `SimEvent`；spawn/kill choke point；`OnMove` 邻居 hunt/flee（`sim_observer_depth` 防重入）
- **Phase 5**: `PlayerPlugin` brain/execution/craft 脚手架
- **延后**: `CampPlugin` / `CommercePlugin` 占位（UI 禁用 trade/recruit）

**坐标约定**：仿真 `(x,y)` Y 向下；输入仅经 `WorldView::area_to_world`；渲染 `WorldRoot` 负责 zoom/pan + Y 翻转。

## 2026-06-06 M3: UI 布局 + 操作交互 + 设计文档同步

- 格网 tile 化：`grid_render.rs` 用单 mesh 批次渲染 36×24 地形格（`terrain_colors.rs`）
- 卡牌仍用独立 sprite；选中绿色 2px 边框（`SELECTION_BORDER`）
- 右侧信息面板：`panel_ui.rs` + `selection_info.rs`（身份/能力/状态/HP/性别/容纳列表）
- 交互：`ui_interaction.rs`（左键选中、拖拽放置、滚轮缩放、中键平移）
- 中文标签：`tag_zh.rs` 全量 `TAG_ZH` / `CAP_ZH`
- 测试：POC 51 + M2 50 + M3 20 = **121 PASS**；验收 `target/release/bevy-poc.exe`
- 设计文档：`docs/design/game-design-overview.md` 增补 Bevy 技术栈章节
- `.cursorignore` 追加 `AIMemory/`、`docs/`（与 `target/` 并列）

## 2026-06-06 M2: 九子系统生态行为 + 50 断言

- 新增 `game_constants.rs`、`main_tick.rs` 及 8 个 `tick_*.rs` 子系统
- 食草 5 profile、捕食（狼/狐/窝）、覆盖觅食、水生 4 卡、分解、环境 tick、9 物种繁殖、容纳、采收
- 修复：`perish_ticks` 默认 -1（非腐坏实体不再被误删）；水生 dispatch 优先级；鱼/水虫分池迁入
- 测试：POC 51 + M2 50 = **101 PASS**
- Bench：`avg_tick_ms=0.279`（<< 5ms 目标）
- 新增 `.cursorignore` 排除 `target/`（约 7.4 GB / 5949 文件）

## 2026-06-06 M1: 全卡定义 + WorldRules + 颜色

- 从 Godot `card_db.gd` + `card_base.gd` 生成 `assets/card_defs.ron`（85 卡）
- 从 Godot `world_rules.gd` 生成 `src/capabilities.rs`（CARD_CAPABILITIES）
- 扩展 `CardDef`：icon、color、is_rooted；标签前缀匹配
- 新增 `terrain_colors.rs`（地形色自 `terrain_visual_palette.gd`）
- 扩充 `world_rules.rs`：标签判定、狩猎链、繁殖、腐坏等（无 tick 逻辑）
- `grid_render.rs` 使用 CardDef.color + 地形色
- 测试：POC 20 + M1 30 = 50 条；POC #03/#06/#07 对齐 Godot 狼群狩猎门槛
- 工具：`tools/gen_card_defs.py`、`tools/gen_capabilities.py` 可重复生成
