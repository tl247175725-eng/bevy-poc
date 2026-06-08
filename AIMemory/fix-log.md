# fix-log — Cursor 修 bug 记录（只记事实，精准）

### 2026-06-06 | cursor | C# 热路径三步（handoff csharp-hotpath deepseek-v4）
- 新建 `FangcunShangguo.csproj` + `csharp/TagLookup.cs`、`DomainResolver.cs`、`FeedHuntSolver.cs`；`dotnet build` 通过。
- 新建 `scripts/core/hotpath_bridge.gd`：`OS.has_feature("dotnet")` 时委托 C#，否则 GDScript 回退。
- `world_rules.gd` 热点（tag/domain/camp/feed/hunt）改经 HotpathBridge；`world_manager._game_tick` 调 `begin_domain_tick`。
- L0：**993 PASS**（标准 Godot 走 GDScript 回退并忽略 C# autoload 加载错误；**Godot 4.6.1 Mono** 走 C# 热路径同样 993 PASS）。
- Mono 可执行：`…\WinGet\Packages\…\Godot_v4.6.1-stable_mono_win64_console.exe`；`project.godot` 已注册 `TagLookup` / `DomainResolver` / `FeedHuntSolver` autoload。

### 2026-06-06 | cursor | 空间索引 Phase 2（handoff spatial-index-phase2 deepseek-v4）
- 生态 `update()` 改 `being`+`autonomous` 桶；水生改 `query_tag("aquatic")`；environment 剩余全扫改索引。
- 寻路：`CardBase` 路径缓存 + 每 tick 节流；`SpatialIndex.blocked_cells`；`_sync_occupancy` 不再全扫卡。
- `card_has_tag` / `card_has_capability` 改卡上预建集合；`query_near` 偏移缓存。

### 2026-06-06 | cursor | 空间索引地基（handoff spatial-index deepseek-v4）
- 新建 `SpatialIndex`（tag + cell 双索引）；`GameState.spatial_index` + spawn/remove/move 钩子。
- `WorldRules` 高频查询（草/灌木/尸体/狼威胁/池/树/地下等）改索引读取；行为文件 4 处全扫改索引。
- `pool_occupants_at` 同时索引 `aquatic` + `floating`（菱角/莲仅 floating）；L0 993 断言 PASS。

### 2026-06-06 | cursor | 生态二期 D-后半（handoff deepseek-v4）
- D4：`cliff` 格型（山邻格 + 高海拔静态岩壁）；`_tick_cliff_weathering` 60s 掉碎石；锤子敲岩壁。
- D5：`landBug` 尸体吸引（范围 4、上限 8）、同格腐解×2；兔/鼠可猎；无尸游荡。
- D3：`wildYam` `in_ground` 容纳 UI；空手/锄挖 → `wildYamRoot`；竹鼠拱食；60s 蔓延上限 12。

### 2026-06-06 | cursor | 生态二期 D-前半（handoff deepseek-v4）
- D1：`acorn`/`pineCone` 卡 + `environment_manager._tick_contained_producers`；栎/松树 in_tree 周期掉落邻格（各限 2）。
- D2：`caltropFruit`/`lotusSeed` 采收；`interaction_manager._try_harvest_pool` 邻格空手采菱角/莲；采尽 40s 再生。
- D3：`birdNest` in_tree；初始 1–2 棵树随机挂巢；橡子/菱角等可食可烤，兔鼠雉鸡可觅食。

### 2026-06-06 | cursor | 生态二期 C（handoff deepseek-v4）
- `in_tree` + `set_in_tree`：栎树/松树藏入树林格；`tree_occupants_at` + `_tree_containment_entries` 点树看容纳。
- 菱角/莲走 `in_pool` + `floating` 标签；`enter_pool` 泛化支持水面附着卡。
- 四卡 `organize.locked` + `be_collected` 仅注册；初始在树/池内 spawn；行为留 D。

### 2026-06-06 | cursor | 生态二期 A+B（handoff deepseek-v4）
- A1：水虫/鱼仅在水潭格内移动，离水标记「搁浅」。
- A2：`WILDPREY_FEAR_RANGE=5`；鹿惧火半径 +1、惧狼改用野猎物半径。
- B1：雉鸡/雏鸡 CardDB + 能力 + 初始生成 + flocking 繁殖；`_tick_rabbit` 泛化 `omnivore.small`。
- B2：竹鼠 CardDB + 能力 + 林下繁殖；`FieldMouseBehavior` 泛化 `burrower`，竹鼠走树周草觅食+掘穴。

### 2026-06-06 | cursor | 底部水潭地形 + 水生容纳
- 移除中央纵向河沟；`world_rules_terrain` 在地图底部中心生成同心水潭（源头 -300，外环 -5…-1），外两圈湿地。
- 水生卡 `in_pool` 隐藏于水潭格内；点选水潭显示容纳列表（藻/鱼/贝/虫），地表不可见。
- `terrain_visual_palette` 按曼哈顿距离渐变蓝色；空桶可在水潭取水。

### 2026-06-06 | cursor | 直觉配色全量落地
- `card_base._card_style`：补全鹿/狐/鼠/牛/桃源/水生/鱼肉等全部 CardDB 类型配色；草保持嫩绿 `c9f28b`。
- 新建 `terrain_visual_palette.gd`：水潭蓝青渐变、湿润土青绿、暗河源头深蓝、河岸湿地浅绿；`terrain_manager` 接入。
- `world_rules_ui`：水潭/湿润土地选中标题与状态文案同步。

### 2026-06-06 | cursor | handoff ecology-foundation：标签→行为全量落地
- `LIVING_GRASS_CAP` 30；初始种群（羊/鹿/水牛/兔/田鼠+幼鼠）按 handoff 表；西侧 `pool` 水潭 + 水生态初始 spawn。
- 标签行为：`flocking` 羊繁殖门槛、`pack_hunter` 寡狼只猎小猎物、`prolific` 兔 3s/胎3只且不迁入、`burrower` 无灌丛掘穴、`perishable` 生肉 90 tick 腐坏。
- 新增 `AquaticEcologyBehavior`（藻再生/虫鱼贝 tick，不经 `ecosystem_behavior_key`）；猎鱼/贝产出 `fishMeat`。
- L0 `_test_ecology_foundation_tags` + 草上限断言改 30；`card_rule_audit` 能力表同步。

### 2026-06-05 | cursor | handoff freeze-commerce：封存经营层卡
- `WorldRules.is_commerce_layer_frozen`：traveler/mushroomFarmer/table/coin/copperBlock 关闭 live spawn；`population_manager` 旅人/蘑菇农跳过。
- `interaction_manager`：`spawn_card_at` 与铜钱→铜块、铜饰合成产出 gated；`card_db` 配方保留。
- L0 `_test_commerce_layer_frozen`。

### 2026-06-04 | cursor | 狼搬肉治本：猎物逃 / 窝旁交付 / 删清路驱赶
- `is_grazer_flee_wolf_threat`：叼肉或实际携肉狼不因 `in_den`/`carryingMeat` 竞态豁免；`nearest_wolf` 改用它。
- `WOLF_DEN_DELIVERY_RANGE=2`：窝旁曼哈顿半径内可交付；幼狼出窝接肉后回窝；成年仅邻格才入窝消化。
- 删除 `ActionRecovery` 狼放下肉驱赶整条路径；暂放肉 goal 仅保留「筑窝后再叼回」。

### 2026-06-04 | cursor | 狼回窝空潜伏 + 取肉僵持 + 可供性去抖
- `ecosystem_manager._update_grass_den_work`：仅 `delivered` 成功才 `set_in_den`（修复邻窝无肉仍入窝 1213 tick）。
- 管线：肉源僵持提前到 den 分支前；`回狼窝*` 无肉立即 fallback；窝内无消化且无肉 `_tick_wolf_den_lurk_stuck` 出窝巡林。
- 玩家：`craft_knife`/`craft_spear`/`forage` 出现后 hold 3 tick，避免材料边界闪烁。

### 2026-06-04 | cursor | 狼取肉僵持 + 玩家可供性去抖（初版）
- 狼：`find_meat_source` 为空时 sync 清 `carryingMeat`；`取肉` 无源立即或 45 tick 后 `饱腹/巡林`。

### 2026-06-04 | cursor | 狼「搬肉」空叼同步
- 猎杀成功不再提前 `carryingMeat=true`（须实际 pickup 或身边暂放肉）。
- `WorldRules.sync_predator_carrying_meat`：无叼肉且无暂放 → 清标记。
- `_tick_wolf_den_work`：仅叼着生肉时 `state=搬肉`；仅暂放肉时 `取肉/暂放肉`。

### 2026-06-04 | cursor | handoff_player-brain-refactor（标签驱动五层）
- `player_brain_tags` / `player_brain_world` / `player_affordance_prey`：运行时标签 + 世界条件注册表。
- `player_affordance` `AFFORDANCE_TABLE`；`player_needs` `NEED_TAG_RULES`；`player_intention` `LONG_TERM_TABLE`。
- `WorldRules.nearest_threat`；`player_behavior` 去魔法数字；灌木 `berry.source`。

### 2026-06-04 | cursor | session_report §8 自主/可供性修复
- `player_needs`：篝火安全区半径与 `PLAYER_WOLF_THREAT_DIST` 对齐，消除 100↔20；饥饿用整数阈值。
- `player_affordance`：修复 `_fire_needs_fuel` 恒 false；补 `collect_fuel`/`craft_spear`/`build_hut`；已有刀不再报 craft_knife。

### 2026-06-04 | cursor | handoff_session-report-player-brain
- `session_report.md` §8：玩家大脑快照表（最新 20 条）。
- `PlayerBehavior.record_session_snapshot_if_needed`：意图/状态/需求Δ>20 写入 `player_brain_log`（最多 50）。
- `PlayerIntention.current_intention_key()`；关游戏 `SessionDiagnostics._append_player_brain`。

### 2026-06-04 | cursor | handoff_ui-containment（§一–三 对齐）
- `build_cell_lines`：overlay 优先首行标题；狐/狼窝早退，去掉灌木+狐窝双轨。
- `_convert_bush_to_fox_den`：清灌丛内鼠后 `remove_card(bush)`，地表仅狐窝。
- `world_rules_ui`：`is_fox_den_card` / `ui_hut_occupant_labels`；地表格字「狐窝」；点灌木 resolve 到狐窝。
- `game_ui`：移除灌木+地下狐窝双面板；容纳仍 LinkButton 跳转。
- L0 **895** PASS。

### 2026-06-04 | cursor | handoff_player-brain（五层人卡智能）
- 新建 `player_affordance` / `player_needs`(SDT) / `player_intention` / `player_behavior`。
- 接入 `player_needs_manager` + `action_runner`：猎杀门控、生火优先、合法 idle「休息中」。
- `hunt_target_score`(tool_hunter) 偏好小型猎物，回避成体大型/水牛。

### 2026-06-03 | cursor | handoff_player-tag-constraint
- 玩家猎杀：`TOOL_HUNTER_MAX_KILLS_PER_DAY=2` + `dailyKills` 日重置（同狐机制）。
- `card_db` 玩家 tag：`omnivore` / `tool_dependent` / `fire_bond` / `opportunistic`。
- `game_ui` 玩家面板改 `build_card_lines` + 饥饿/猎杀/工具/HP 追加行。

### 2026-06-03 | cursor | handoff_tag-essence（标签本质性）
- 狐：`MESOPREDATOR_MAX_KILLS_PER_DAY=2` + `dailyKills` 日重置；满额不再选猎。
- `card_db`：`body.*` 补全；孤立装饰 tag 删除；`is_raw_meat` 改能力判定。
- `selection_info_panel`：`TAG_ZH`/`_SKIP_TAGS` 同步；`game_state` 生态常量加标签依据注释。

### 2026-06-02 | cursor | handoff_pre-iteration-cleanup（三部分）
- `world_rules.gd`：13 处 `is_*` 改 tag/capability；补 `RULE_EMITS` 文档字典。
- `world_rules_camp.gd`：`wolfDen` → `den`+`animalHome` 且排除 `den.candidate.fox`（避免狐窝进 `domain.wolf_pack`）。
- `card_base.gd`：`set_in_den` 可见性改 `capability.return_home` + `uses_underground_den`。
- 删 5 个 behavior 空壳；`unit_test_cases` 改 `PredatorDenBehavior`。

### 2026-06-02 | cursor | 左键选中绿色高亮框
- 卡牌：`set_selected` 绿框置顶（`z_index=30`）；空地：`CellSelectionHighlight` 与卡同尺寸同色。
- `world_manager` 选卡显框、选格显格框，互斥刷新。

### 2026-06-02 | cursor | handoff_ui-containment 容纳 UI
- 容纳【】条目 + LinkButton 跳转；草棚玩家/存货；格子 overlay 标题；选中不拖/不命中藏鼠。
- L0 **887** PASS。

### 2026-06-02 | cursor | PROTOCOL §3.1b 可视化全覆盖审计
- `ui_visual_coverage_audit.gd`：遍历 CardDB 全卡 + 地图格型，校验标签/能力中文化、选中 UI、overlay/藏有/伏草角标。
- `world_rules_ui.gd`：地面角标优先级 狐窝→窝/肥→藏/伏；灌木/草皮容纳与潜伏入口标记。
- `selection_info_panel`：`capability.consume_service`；L0 **875** PASS。

### 2026-06-02 | cursor | 信息栏零遗漏（全标签/能力/容纳/状态）
- `selection_info_panel.gd`：`build_card_lines` / `build_cell_lines` 经 `_build_card_core` 统一输出「身份」「能力」「容纳」「状态」；`TAG_ZH`/`CAP_ZH` 全量 + `_token_zh` 回退，过滤英文键名。
- 卡：藏有/藏身/地下狼窝·腐殖/在窝内/携带/同格、旅人/狼狐生态/尸体/树/经营/灌木窝储等动态行；窝/腐殖仍 `build_den_lines`/`build_humus_lines` 叠窝内·砸毁。
- 格：地形+火/湿/暗河/河岸径流、海拔、地表卡列表、地下 overlay 摘要、格状态。
- 单测 `_test_selection_info_panel` 放宽藏有/未知 capability 断言；L0 **849** PASS。

### 2026-06-02 | cursor | handoff_ui-fix-five（五项 UI 修复）
- `sex_bracket`：空性别不显示【无】。
- `select_card`：绿框 `ring_card` / 面板 `panel_card` 解耦（项 2，延续）。
- `ground_corner_marker_for_card`：overlay 窝/肥 + 可见狐窝「狐」角标（项 3、5）。
- `build_cell_lines`：`elevation > 0` 时显示「海拔：N」。

### 2026-06-02 | cursor | overlay 角标 + 选中框分离
- 同格 `cell.overlay`：地面卡角标「窝」「肥」（`world_rules_ui.overlay_marker_at_cell` + `card_base._refresh_corner_tag`）。
- `select_card`：绿框留在原始命中卡，信息栏仍 `ui_resolve_selection_card`。

### 2026-06-02 | cursor | handoff_ui-label-render 对齐
- `world_rules_ui.gd`：`overlay_card_at_cell` / `ui_resolve_selection_card` / 格子 `cell_display_title`·`cell_terrain_tag_keys`·`cell_state_label`。
- `selection_info_panel.gd`：handoff 翻译表、无分隔线、空分区不显示、灌木「藏有」、humus/窝格式、零英文 fallback。
- `world_manager.select_card` 经 overlay 解析；空地/overlay 格 `build_cell_lines` 可走窝/腐殖面板。

### 2026-05-30 | cursor | 信息栏标签化 + 窝 UI + 空地选中 + 删猞猁
- `selection_info_panel.gd`：卡牌/空地/狼窝·狐窝统一分区（身份/形态/能力/状态/HP）；性别 `【公】【母】`；capability 直译无「可」前缀；无关系区。
- `world_manager`：`select_cell_at` + 左键空地选中；`game_ui.render_selected(card, cell)`。
- 窝面板：`窝内：成年×N 幼×N 储肉×N` + `砸毁 n/2` 随 tick 刷新。
- 移除 `lynx`：`card_db`、`CARD_CAPABILITIES`、`spawn`、审计、单测改狐/狼断言；`_test_selection_info_panel` 补覆盖。

### 2026-05-30 | cursor | handoff 二→五（cell.overlay / 地形 / 火域干草）
- **二 humus**：`humus` 卡 `cell.overlay`；尸体自然腐解 spawn（屠宰不产生）；同格≤2 层、4 游戏日消散；活草再生间隔×`GRASS_REGEN_HUMUS_MULT`。
- **三 干草**：删初始干草 spawn 与 `_grass_aging`；`FIRE_DRY_GRASS_RANGE/TICKS` 火边草皮变干草。
- **四 狼窝**：`wolfDen` 改 overlay + `TerrainManager.underground_state`；地面格仍可长草；进窝隐藏（`PredatorDenBehavior` 同步）。
- **五 地形**：`WorldRulesTerrain` 海拔梯度、D8 径流、河岸刷草格、暗河入/出口；卡 `visual_world_pos` Y 偏移。
- L0 **849** PASS（+4 测例）。

### 2026-05-30 | cursor | 食物网重建（handoff 一）
- `bison`→`waterBuffalo`（`tough` 成体不可猎）+ `waterBuffaloCalf`；羊/羔补 `herbivore`/`largePrey` 标签。
- `_pack_prey_allowed` / `mesopredator_diet_key` / `hunt_target_score` / `feed_source_score`（狐腐肉低优先）。
- L0 `_test_food_chain`；**828** assertions PASS。

### 2026-05-30 | cursor | 初始生成猞猁/野牛
- `world_manager._spawn_initial_cards`：各 spawn 一只 `lynx`（河谷东侧）、`bison`（营地东南），用于 F5 验收 capability 签名吸附。

### 2026-05-30 | cursor | FIX 狐/猞猁追猎零击杀 + 狐不筑窝
- **零击杀**：`is_hunt_target_for` 排除眩晕猎物 → 首击只晕后脱靶；`attackTimer` 未在 `CardBase` 声明导致计时不可靠；邻格仍记 §7「攻击」但 `try_hunt_attack` 常被挡。
- **修**：眩晕猎物可补刀；`attackTimer`/`huntCooldown` 入 `CardBase`；`execute_capability_hunt_attack` 用 `real_delta` 累计攻击间隔；`apply_hunt_miss_cooldown` 覆盖 mesopredator。
- **不筑窝**：`flee` 先于 `den_work`，邻狼时母狐永远躲而不占灌木。
- **修**：无窝且 `should_build_den` 时 `den_work` 优先于 `flee` 跑一轮。

### 2026-05-30 | cursor | FIX 狼卡窝不出（Step3 管线回归）
- **现象**：F5 狼 900+ tick 全在窝里。
- **根因**：`PredatorDenBehavior` 消化完出窝 → 同 tick `CapabilityPipeline._tick_wolf_den_work` 见 `meatFedToday` 满/饱腹仍调 `update_den_work` → 邻格再 `set_in_den` + 重置 `huntCooldown` → 死循环。
- **修**：出窝且 `in_den=false` 时不再 `update_den_work`，改 `goal=巡林`；`_update_grass_den_work` 对「已饱且未叼肉」早退不入窝。
- **续修**：`wolf.in_den` 时管线 `den_work` 直接 return，不再调 `update_den_work`（L208 每 tick 把 `huntCooldown` 重置 28s）；窝内冷却只由 `PredatorDenBehavior._tick_in_den` 倒数。

### 2026-05-30 | cursor | Step5–6 capability 签名
- Registry match 缩为 predator_den / mesopredator_hunt / herbivore_grazer / cover_forager + traveler/taoyuan/farmer
- 狼狐兔鹿羊 behavior 合并；lynx/bison 仅标签注册即吸附

### 2026-05-30 | cursor | Step4 宿主收敛
- `EcosystemManager` 狼狐平行 API → `den_for` / `update_den_work` / `try_scavenge`；`actor_den_build_mode` 区分草窝/灌木窝
- 叼肉无狐窝时 `update_den_work` 仍筑灌木窝（避免 carrying 分支挡 build）

### 2026-05-30 | cursor | FIX 生态 UI + 狼窝 + 狐动
- **丛内鼠不可见**：点灌丛无鼠信息 → `world_rules.cover_occupants_of` + `game_ui` 灌丛分支列丛内田鼠
- **狼不出窝**：`wolf_behavior` 窝内 `huntCooldown<=0` 时被重置为 28s → 删除重置，只倒计时；进窝时由 `ecosystem_manager` 设冷却
- **狐似不动**：`fox_behavior` 眩晕未查 `stun_timer`；惧火/躲人/躲狼未走 `consume_move_tick` → 对齐狼；狐窝休整设 `WOLF_DIGEST_SECONDS`

### 2026-06-01 | cursor | FIX 能力片段管线（Step3）
- `CapabilityBehaviorPipeline` 有序片段；狼狐 flee/fire/den/scavenge/hunt 迁出 behavior；Registry 统一入口

### 2026-06-01 | cursor | FIX 生态饱食统一（Step2）
- `mark_ecology_fed` 等四函数 capability 化；`starveDays` 统一；`CardBase` 生态字段

### 2026-06-01 | cursor | FIX 猎物 profile 化（Step1）
- `is_hunt_target_for` / `hunt_target_score` / `hunt_target_visible_to` 改 `HUNT_PROFILE_*` + CardDB tag；狼可猎 `mesopredator`（狐，低优先 +2.5）
- `is_active_hunt_threat` 改 `predator` 标签，避免狐因 `return_home+hunt` 被当成狼威胁

### 2026-05-30 | cursor | FIX 狐捕猎迁移后分支
- 去掉 FoxBehavior 顶部 consume_move_tick 挡筑窝/清腐；tick_capability_hunt 先找猎物再 move_tick；恢复 §7 _fox_diag

### 2026-05-30 | cursor | FIX 恢复狐狸 spawn
- `capability.hunt` 已接入后重新启用 `_spawn_fox_family_near`（此前 disable-fox-spawn 临时关闭）

### 2026-05-30 | cursor | ARCH capability.hunt 片段
- 狼/狐捕猎合并为 `WorldRules.tick_capability_hunt`；Registry 在生态 tick 后按标签调用

### 2026-05-30 | cursor | BALANCE 田鼠节奏
- 吃虫 1.5s/口（`mouseEatTimer`）；丛虫 5 蚯蚓+3 甲虫、再生 20s；满 HP 饱歇 6s 再换丛

### 2026-05-30 | cursor | PERF 寻路 AStarGrid2D
- `find_path` 换 Godot `AStarGrid2D`（C++）；删 BFS/MAX_SEARCH/盲向步；每查路前同步地形+卡占位

### 2026-05-30 | cursor | FIX 狐行为重写（狼模式）
- **四轮修仍不动**：`fox_behavior` 整段对齐 `wolf_behavior` 管线；差异仅灌丛筑窝、`best_hunt_target`、`try_fox_scavenge`；from_card 失败回退 actor

### 2026-05-30 | cursor | FIX 狐 from_card / §7 空
- **§7 无记录**：`tick` 首行加 `enter_tick`；`from_card` 失败且 type 为 fox 时回退 `actor`；`from_card` 对齐 WolfCard

### 2026-05-30 | cursor | FIX 狐 spawn 初始状态
- **spawn 写死「寻灌丛」**：tick 早 return，200+ tick 不动 → 初始改「巡林」/「觅食巡逻」；母狐仍由 `fox_should_build_den` 触发筑窝

### 2026-05-30 | cursor | FIX 狐寻灌丛卡死（可达过滤）
- **河对岸 8 丛全不可达**：`_nearest_bush_for_fox_den` 每丛 `can_reach_neighbor` BFS → 全 null → 狐卡「寻灌丛」+ 掉帧
- **改**：删可达过滤，按距离选最近；走不到靠 `FOX_BUILD_STUCK_TICKS` 换目标 + `random_step`

### 2026-05-30 | cursor | FIX 报告不动 + 狼迁出 + 狐筑窝死锁
- **报告全员不动293**：`_is_idle_card` 仅看 path/craft → `_is_stuck_card` 需 state 不变；吃草/啃食 `eat_time` 递增、桃源远观/避让白名单不算
- **封闭模式狼被移除**：`should_wolf_leave` 无 `ECOSYSTEM_CLOSED_MODE` 守卫 → 近窝>2 仍 `remove_card`
- **狐占灌木死锁**：`_nearest_bush_for_fox_den` 不判可达 → `can_reach_neighbor`；移动失败 `random_step`；45 tick 换灌木目标

### 2026-05-30 | cursor | FIX 狼可见 + 鼠卡丛 + 饿死 + 狐筑窝
- **成狼入窝消失**：`card_base.set_in_den` 对狼 `visible=false` → 窝内保持可见；`wolf_behavior` 窝内同步窝格坐标
- **鼠呆丛内**：空灌丛仍 `return` → 窝内无虫先 `exit_cover`；觅食改 `nearest_cover_plant_with_food`（含草）
- **草无虫源**：草皮加 `worm_count/beetle_count`，`environment_manager` 再生 35s；灌丛再生 90→40s
- **生物饿死**：`population_manager._on_ecology_feed_new_day`；`WorldRules.mark_ecology_fed`；狐 `daysWithoutMeat`；封闭模式 3 日未进食移除
- **狐找灌木不动**：`bushes()` 空只 random → `_ensure_bush_for_fox_builder` 补灌丛；占灌木每帧 `move_one_step_near`；非公狐清「寻灌丛」→「巡林」
- **灌丛测空虫**：`BushCard._ensure_ecology_defaults` 会填回初始虫 → `depleted_microfauna` meta 跳过填充

### 2026-05-31 | cursor | FIX 编译级联 CardBase 找不到
- **现象**：`world_manager`/`wolf_card` 等报 CardBase、CardDB、WorldRules 未声明
- **改法**：`card_base.gd` 把 `path` 等成员变量挪到 `set_in_den` 之前（此前插在函数体中间破坏类解析）；`--import` 刷新 global class cache
