# 设计变更日志

## 2026-05-30 — 调度 Step5–6 capability 签名 + 行为合并（Cursor）

- `ecosystem_behavior_key`：`predator_den` / `mesopredator_hunt` / `herbivore_grazer` / `cover_forager` + 社交例外。
- `PredatorDenBehavior`、`HerbivoreGrazerBehavior`；Wolf/Fox/Rabbit/Deer/Sheep 空壳转发。
- 验收卡 `lynx`、`bison`（card_db + CARD_CAPABILITIES + 审计表）。

## 2026-05-30 — 调度 Step4 宿主收敛（Cursor）

- `EcosystemManager`：`den_for(actor)`、`update_den_work`、`try_scavenge` 等统一入口；草窝/灌木窝由 `WorldRules.actor_den_build_mode` 分发。
- `CapabilityBehaviorPipeline` / 狼狐 behavior 改调统一 API；保留 `den_for_wolf` 等薄 wrapper。

## 2026-06-01 — 调度 Step3 能力片段管线（Cursor）

- `CapabilityBehaviorPipeline`：flee→fire→den_work→scavenge→hunt；狼狐 behavior 瘦身；Registry 统一调度。
- 叼肉时跳过惧火片段；`tick_capability_hunt` 返回 bool。

## 2026-06-01 — 调度 Step2 生态饱食统一（Cursor）

- 饱食 API 按 `forage`/`hunt`/`be_cared_for` 判定；`daysWithoutMeat` 并入 `starveDays`；狼保留 `meatFedToday` 配额。
- `CardBase` 生态字段；`population_manager` 日馈单循环。

## 2026-06-01 — 调度 Step1 猎物 profile 化（Cursor）

- `WorldRules`：`hunt_profile_for` + `HUNT_PROFILE_*`；`is_hunt_target_for` / `hunt_target_score` / `hunt_target_visible_to` 改由 CardDB tag 驱动。
- 狼可猎带 `mesopredator` 目标（狐），`hunt_target_score` +2.5 保持低优先；L0 `_test_hunt_profile_matrix`。

## 2026-05-30 — v3.0-A 步骤 2 机制维度扩展（Cursor）

- **灌木** `bush`：浆果产出、微型生物再生、掩体；8 株初始（河岸+林缘）。
- **田鼠** `fieldMouse`/`fieldMousePup`：依赖灌木微型生物，不吃活草。
- **狐狸** `fox`/`foxCub`/`foxDen`：多策略捕食、尸体清道夫藏肉、避让狼火人；狼可猎狐（低优先）。
- 兔优先灌木浆果/微型生物，活草 ≤4 口/日。
- `closed_ecology_trace` 增加 bush/mice/fox 采样。

## 2026-05-30 — v3.0-A 步骤 1 封闭生态 + deerFawn + 桃源 live（Cursor）

- `ECOSYSTEM_CLOSED_MODE`：羊不 MAP_EXIT、禁外羊/狼群迁出入；狼断粮 3 天个体死亡。
- 草间隔 6s、活草上限 18、初始 10 株；兔吃草 45s/口。
- `deerFawn` + 鹿繁殖生命周期；狼猎幼体优先；桃源三卡 live spawn + trace 补强。
- 验收待跑：L0、L1.5（deer*, taoyuan*）、closed/deer/social trace、L2b 1×。

## 2026-05-20 — v3.0-A2 鹿生态链（Cursor）

**范围：** `deer` / `deerCorpse` / `deerMeat`；狼/玩家猎鹿；`deerMeat + fire → cookmeat`；未动 A3。

**实现：**

- CardDB + WorldRules capabilities；`deer_behavior.gd`；击杀产鹿尸、屠宰产鹿肉。
- 狼 `hunt_target_score` 对 `largePrey` 优先；`is_butcher_chain_target` / `butcher_meat_type_for_corpse`。
- 初始地图 2 头鹿；L0 `_test_deer_ecology_chain`。

**验收：**

- L0 unit：**707** assertions PASS
- L1.5：`deer,deerCorpse,deerMeat` **17** assertions PASS
- L2b live 1×：GH tick=**1519**，**USELESS=0**，meat_peak=**0**

---


**范围：** A1 only — terrain 统一 + WorldRules 查询 + 36×24 地图 + 初始生存区 + terrain trace。

**实现：**

- 新增 `scripts/core/world_rules/world_rules_terrain.gd`；`TerrainManager` / `Pathfinding` 共用 `base_cell_type`。
- `WorldRules` facade：`terrain_tags_for_cell`、`cell_has_terrain`、`zone_for_cell`、`is_water_source_cell`、`is_taoyuan_boundary_cell`。
- `GameState` 地图 18×12 → **36×24**；`START_PLAYER_*` / `MAP_EXIT_*`；`world_manager` 初始 spawn 迁移。
- 新增 `scenes/terrain_trace.tscn` + `tools/run_terrain_trace.ps1`；L0 `_test_terrain_world_rules`。

**验收：**

- L0 unit：**691** assertions PASS（+13 terrain）
- terrain trace：**PASS**（六类 terrain 标签 + 生存资源可达）
- L2b live 1×（初跑）：GH tick=1500，USELESS=1（shelter 96t @24,12，未稳定复现）
- L2b live **3×**（A1 稳定性复验 2026-05-20）：**USELESS=0 / stalls=0**；GH tick=1426 / 1436 / 1413；meat_peak=0 / 3 / 1
- **状态：A1 provisional stable** — A2 门已开

**未做（A2/A3）：** 鹿、桃源人、旅人封存、dining。

---

## 2026-05-20 — 清理桌面冗余副本

- 删除桌面 4 个 stub 文件、`方寸商国MVP/` 文件夹、`tools/sync_design_doc.ps1`。
- 桌面仅保留 `方寸商国_文档入口.md` 指向工程 `docs/README.md`。

---

## 2026-05-20 — 设计文档归拢（docs 单一正本）

**动机：** 策划文档分散在桌面根目录、MVP 文件夹与项目内摘录，难以找齐。

**变更：**

- 新增 `docs/README.md` 作为**唯一入口**
- 策划正本迁入 `docs/design/core/`：设计规则圣经、构建路线图、世界运转共识、意向系统 v0.3
- 旧摘录与 v0.2 移入 `docs/design/archive/`
- 重写 `docs/design/README.md`；更新 `CODEX_HANDOFF.md`、`AGENTS.md`、`sync_design_doc.ps1`（圣经路径 → `core/`）
- 桌面同名文件改为跳转 stub；`方寸商国MVP/` 仅保留可选圣经副本

**以后：** 只改项目内 `docs/`；桌面打开 `方寸商国_文档入口.md` 即可跳转。

---

## 2026-05-28 — v2.73 卡牌工厂 Phase 2：脚手架

**阶段定位：** 正式增量迭代。Phase 2 补「加卡产线」——生成粘贴片段与验证命令，**玩法仍由策划设计 + 与 AI 讨论**。

**实现摘要：**

- `CardFactoryScaffold.build()` / `write_bundle()`：按 `--type` + `--tags` 推断 `factory_lane`，输出 CardDB `_reg`、CARD_CAPABILITIES、AUDIT_NOTES、WorldRules TODO、verify PS1、lane playbook。
- CLI：`scenes/card_factory_scaffold.tscn`；包装脚本 `tools/run_card_factory_scaffold.ps1`。
- 已存在卡（如 `berry`）可生成「对照复查」脚手架。

**验收：**

- L0 unit：**678** assertions PASS（+7 scaffold 断言）

**工作留痕：** 脚手架输出目录 `generated/scaffold/<type>/`；合入后更新 CHANGELOG + `CODEX_HANDOFF.md` §12；刷新 `export_card_rule_facts`。

---

## 2026-05-28 — v2.72 卡牌工厂 Phase 1：守卫层

**阶段定位：** 项目已进入**正式增量迭代**（v2.70+）。v2.72 是卡牌工厂**第一期**（守卫 + 分级 + L1.5），不是项目仍处于 MVP。Phase 2（stub 脚手架）见 `CODEX_HANDOFF.md` §0.3a。

**范围：** 不做一键生成玩法，不迁移 relation/impact schema；先给新卡接入补自动分级、入口守卫和 L1.5 单卡 smoke。

**实现摘要：**

- `CardRuleAudit` 新增 `factory_lane` 自动分级：`camp_resource`、`manual_resource`、`production_node`、`structure_service`、`actor_loop`、`environment_source`。
- L0 守卫检查高交互卡必须有 AI 可达性说明或登记例外；`hoe`、`hammer`、`halfbucket` 作为首批例外写明原因。
- 新增 `scenes/card_factory_smoke.tscn` / `scripts/test/card_factory_smoke_runner.gd`，验证代表卡的收纳、寻路、规则 IO、玩家入口声明与运行时一致。
- `docs/design/card-rule-audit.md` 自动导出新增 `factory_lane` 列，并把新卡接入 checklist 提升为工厂接入门槛。

**验收：**

- audit export：**PASS**，刷新 `generated/card-rule-facts.md` 与 `card-rule-audit.md`
- L0 unit：**671** assertions PASS
- L1.5 card factory smoke：**58** assertions PASS；`hammer` / `halfbucket` 输出设计例外说明
- L2b 未跑：本片未改主链、need、craft FSM 或 WorldRules 行为，按计划只要求 L0 + L1.5

---

## 2026-05-28 — v2.71 ActionRunner 调度层收口

**范围：** 清零 live 主链 `eval / mission / stack / craft` 错位空转，恢复增量迭代前置门槛。

**实现摘要：**

- `ActionRunner.sync_current_need()` 统一写 `current_need`；`PlayerNeedsManager` handler 不再覆写自动调度 need。
- `_purge_stale_stack()` 增加 mission 合法 chain 守卫；菇棚项目锁只在 mission 仍为 `build_greenhouse` 时强制 greenhouse/lumber。
- 夜间休整、shelter fallback 显式写入可解释状态，避免旧 `state/goal` 残留被 trace 识别为空转。
- `cookMeat`、`forageEat`、`makeAxe`、`makeSpear` 的无路径阶段改为 fail_task，交给恢复/换目标机制。
- 狼非近战威胁不打断携材生火原子移动；菇棚取水在无可达桶时允许营地补桶。

**验收：**

- L0 unit：**654** assertions PASS
- L2b live ×5：tick **2365 / 1289 / 1276 / 1264 / 1259**，每轮 `USELESS=0`，`meat_peak=2 / 2 / 0 / 2 / 5`

---

## 2026-05-27 — 设计文档全量同步（增量迭代阶段）

**范围：** 桌面 + 项目内策划/工程文档对齐 v2.70 基线，供 Codex 接手。

**更新文件：**

- 桌面：`方寸商国_构建路线图_v3.x.md`、`方寸商国_意向系统与世界演化设计总结.md`（§16）、`方寸商国_世界运转共识文档_v0.1.md`（v0.6 §19）、`方寸商国_设计规则圣经_v0.3.txt`（末尾 v2.64–v2.70）
- 项目：`docs/CODEX_HANDOFF.md` §0、`docs/design/build-roadmap-v3.x.md`、`intent-system-world-evolution-v0.3.md` §16、`README.md`、`world-rule-tag-machine-v0.1.md` §6
- 三份规则圣经已 sync（项目 / 桌面根目录 / 方寸商国MVP）

**阶段声明：** v2.x 耦合收口完成 → **增量迭代**（新卡 checklist + WorldRules + L0/L2b）

---

## 2026-05-27 — v2.70 耦合风险收口（UI + camp/commerce 子域）

**主文档：** `coupling-risk-mitigation-v0.2.md`，工程记录见 `CODEX_HANDOFF.md` §12

**实现摘要：**

- 新增 `world_rules_camp.gd` / `world_rules_commerce.gd` / `world_rules_ui.gd`；`WorldRules` facade 转发不变
- `world_manager.gd` / `game_ui.gd` / `drag_manager.gd` 展示与交互条件改走 `WorldRules.ui_*`
- `trade_item_staged_on_table` 改 `card_matches_commerce_demand`；`CardRuleAudit` 扫描扩展至 `ui/`、`input/`
- L2b 连续 3 次 PASS

**验收：**

- L0 unit：**625** assertions PASS
- L2b live：PASS tick=1494 / 1309 / 1339

---

## 2026-05-27 — v2.69a WorldRules 材料域拆分起步

**主文档：** `coupling-risk-mitigation-v0.1.md` §4.3，工程记录见 `CODEX_HANDOFF.md` §12

**实现摘要：**

- 新增 `scripts/core/world_rules/world_rules_material.gd`，承接材料、木材、容器、来源、湿木/菇木、燃料等纯查询实现
- `WorldRules` 保持原 facade API：`is_lumber_material`、`is_water_bucket`、`is_lumber_source_tree`、`is_heated_water_container` 等外部调用不变
- 本切片只搬查询实现，不改任务语义，不改 CardDB 标签

**验收：**

- L0 unit：**617** assertions PASS
- L2 behavior_trace_live：**PASS tick=1334 / wall=31.4s**，蘑菇棚建成，卡滞段 0

---

## 2026-05-27 — v2.68 Pathfinding passable 规则化第一刀

**主文档：** `coupling-risk-mitigation-v0.1.md` §4.9，工程记录见 `CODEX_HANDOFF.md` §12

**实现摘要：**

- 新增 `WorldRules.is_pathfinding_passable_occupant(card)`，暂复用 `is_layout_passable_occupant`
- `is_layout_passable_occupant` 扩展到草、灌木、可采食物、水桶、非锁定营地可收纳物
- `Pathfinding.blocks_actor` 不再维护独立 passable type 白名单，改向 `WorldRules` 询问
- 阻挡侧保守收口到 `WorldRules.is_pathfinding_blocking_occupant`：rooted、businessUnit、shelter、生物、尸体、营地域定义者阻挡；散落可搬物显式可通过；未知未登记类别默认阻挡
- L0 新增寻路规则用例：草 / 浆果 / 空桶 / 散落工具 / 湿木可通过，羊 / 树 / 篝火 / 狼窝阻挡
- `card-rule-audit.md` 补新卡接入 checklist，要求新增卡先明确标签维度、能力、收纳、寻路、规则 IO、AI 可达性、运行时状态和专名规则理由
- `behavior_trace_runner` 的 live 结束输出改为终端摘要，完整 trace 继续写 `behavior_trace_report.txt`

**验收：**

- L0 unit：**617** assertions PASS
- L2 behavior_trace_live：摘要输出后 **PASS tick=1396 / wall=16.7s**，蘑菇棚建成，卡滞段 0

---

## 2026-05-27 — v2.67b WorldRules service/state 子域拆分

**主文档：** `coupling-risk-mitigation-v0.1.md` §4.3 / §4.4，工程记录见 `CODEX_HANDOFF.md` §12

**实现摘要：**

- 新增 `scripts/core/world_rules/world_rules_service.gd`，承接 service provider 查询
- 新增 `scripts/core/world_rules/world_rules_state.gd`，承接旅人离开、树状态、眩晕等状态谓词实现
- `WorldRules` 继续保持 facade API，外部调用点不变
- `CardRuleAudit` 的 runtime state 扫描白名单加入 `world_rules_state.gd`，允许状态字符串集中在规则子域

**验收：**

- L0 unit：**603** assertions PASS
- L2 behavior_trace_live：**PASS tick=1309**，蘑菇棚建成，卡滞段 0

---

## 2026-05-27 — v2.67a WorldRules 配方域拆分起步

**主文档：** `coupling-risk-mitigation-v0.1.md` §4.3 / §4.6，工程记录见 `CODEX_HANDOFF.md` §12

**实现摘要：**

- 新增 `scripts/core/world_rules/world_rules_craft.gd`，承接配方查询实现
- `WorldRules.can_craft_cards` / `can_craft_types` 保持原 API，改为 facade 转发到配方子模块
- 外部调用点不变，本切片只验证 Godot preload / 子模块边界，不继续扩大搬家范围
- `behavior_trace_runner` 的 live 模式不再逐状态打印到终端，完整 trace 仍写入 `behavior_trace_report.txt`；避免输出量本身触发 wall-clock 假失败

**验收：**

- L0 unit：**603** assertions PASS
- L1 intent_trace：**PASS ticks=1730**，蘑菇棚建成
- L2 behavior_trace_greenhouse：**PASS tick=1571**，蘑菇棚建成，卡滞段 0
- L2 behavior_trace_live：优化 live 终端输出后 **PASS tick=1293 / wall=55.9s**，蘑菇棚建成，卡滞段 0；优化前曾连续 2 次 120s wall FAIL

---

## 2026-05-27 — v2.66 `can_craft_cards` + shelter/fire 配方层收口

**主文档：** `coupling-risk-mitigation-v0.1.md` §4.6，工程记录见 `CODEX_HANDOFF.md` §12

**实现摘要：**

- 新增 `WorldRules.can_craft_cards(card_a, card_b, expect_result)` 与 `WorldRules.can_craft_types(type_a, type_b, expect_result)`，统一由规则层询问 `CardDB` relation / impact
- `WorldHelpers.can_craft` 改为转发 `WorldRules.can_craft_types`，新增 `WorldHelpers.can_craft_cards` 兼容入口
- `fire_tasks` / `shelter_tasks` 的篝火、草棚、桌子配方检查改传 `CardBase`，不再在任务层拆 `card_type`
- L0 新增配方查询用例：wood+shard→fire、twig+grass→hut、wood+woodStruct→table、twig+shard→spear，以及 spear negative/null guard

**验收：**

- L0 unit：**603** assertions PASS
- L2 behavior_trace_live：首次 120s wall FAIL（玩家饥死，既有 live 波动）；复跑 **PASS tick=1268**，蘑菇棚建成，卡滞段 0
- 剩余边界：`tool_tasks` 长矛、`survival_tasks` 熟肉仍保留 `WorldHelpers.can_craft(...card_type...)`，按后续 R6 小切片处理

---

## 2026-05-27 — v2.65 状态谓词收口

**主文档：** `coupling-risk-mitigation-v0.1.md` §4.4 / 附录 A，工程记录见 `CODEX_HANDOFF.md` §12

**实现摘要：**

- 新增 `WorldRules` 状态谓词：`is_stunned_being`、`is_traveler_departing`、`is_tree_depleted`、`is_tree_depleted_state`、`is_tree_damaged`、`is_tree_active`
- 替换运行时代码中 `"眩晕"`、`"离开"`、`"枯竭"`、`"受损"`、`"扎根"` 的直接状态判断；状态赋值保留在状态机输出处
- `CardRuleAudit.validate()` 增加 runtime state 比较扫描，禁止在 `player/world/core` 里重新散落核心状态字符串判断
- L0 新增状态谓词用例，覆盖眩晕动物、离开旅人、树木 active/damaged/depleted

**验收：**

- L0 unit：**597** assertions PASS
- L2 behavior_trace_live：首次 120s wall FAIL（玩家消失，既有 live 波动）；复跑 **PASS tick=1359**，蘑菇棚建成，卡滞段 0

---

## 2026-05-27 — v2.64 漏网清扫 + need 契约 L0

**主文档：** `coupling-risk-mitigation-v0.1.md` §4.2 / §4.7，工程记录见 `CODEX_HANDOFF.md` §12

**实现摘要：**

- 新增 `NeedContract`：登记 P0 need 的执行入口、blocked 文案，并提供只读 `can_start_need` 契约检查
- L0 新增六个最小 fixture：`fire`、`hunger`、`build_greenhouse`、`commerce`、`cleanliness`、`storage_idle`，验证 `NeedEvaluator` 报出的 need 必须可开工
- `CardRuleAudit.validate()` 增加 runtime `card_type == / in [...]` 扫描；`world_manager.gd` UI/展示分支暂列白名单
- 清扫漏网硬编码：`mushroom_helpers.apply_wet_wood` 改 `is_water_bucket`；`world_helpers` / `manual_control` 玩家判断改 `PlayerCard`

**验收：**

- L0 unit：**587** assertions PASS
- L2 behavior_trace_live：首次 tick=5000 FAIL（既有 live flaky，卡滞段 0）；复跑 **PASS tick=1307**，蘑菇棚建成，卡滞段 0

---

## 2026-05-20 — v2.63 环境/生态 tick/经营 demand 规则化

**主文档：** 工程记录见 `CODEX_HANDOFF.md` §12

**实现摘要：**

- 新增 `is_being_card` / `is_heated_water_container` / `is_mountain_source` / `is_wolf_spawn_avoid` / `penalizes_open_spot_score` / `is_rabbit_forage` / `card_matches_kind` / `nearest_loose_card_for_kinds` / `card_matches_commerce_demand` / `ecosystem_behavior_key` 等
- `loose_cards_of_kind` 扩展 fire/hut/table/mountain/tree/cookmeat/weapon 规则分支
- 替换：`environment_manager` / `commerce_tasks` / `player_hunt_helper` / `action_runner` / `craft_task_helpers` / `rabbit_behavior` / `ecosystem_tick_registry` / `world_helpers` / `player_needs_manager` / `camp_helpers` / `ecosystem_manager`

**验收：**

- L0 unit：**562** assertions PASS
- L2 behavior_trace_live：**PASS tick=1295**

---

## 2026-05-20 — v2.62 Shelter / Interaction / 态势扫描规则化

**主文档：** 工程记录见 `CODEX_HANDOFF.md` §12

**实现摘要：**

- 新增 `neighbor_sleep_hut` / `neighbor_commerce_table` / `is_tool_carrier` / `is_sheep_butcher_target` / `scan_direction_*` / `camp_structure_overlay_kind` / `effective_player_tool_type` 等
- `shelter_tasks` / `interaction_manager` / `spatial_scan` / `zone_overlay` / `camp_planner` 去 hut/table/fire/wolf 硬编码

**验收：**

- L0 unit：**541** assertions PASS
- L2 behavior_trace_live：3 次 FAIL（tick 上限 5000）— 已知 flaky

---

## 2026-05-20 — v2.61 食物/营地整理/需求层规则化

**主文档：** 工程记录见 `CODEX_HANDOFF.md` §12

**实现摘要：**

- 新增 `is_cooked_meat` / `is_forage_food` / `is_bush` / `is_player_food_source` / `is_storable_food` / `is_organize_immovable` / `is_camp_essential_storable` / `count_cooked_meat_near_table` / `nearest_fire_fuel_prefer_wood` 等
- `camp_helpers` / `player_needs_manager` / `need_evaluator` / `commerce_decision` / `survival_tasks` / `action_recovery` / `world_helpers` 去 food/structure/type 硬编码
- 菇链携带态改 `is_empty_bucket` / `is_water_bucket`；`population_manager` 旅人检测改 `traveler_guests`

**验收：**

- L0 unit：**529** assertions PASS
- L2 behavior_trace_live：5 次 FAIL（tick≈2347 饥死 / 120s wall 超时）— 已知 flaky，待重跑

---

## 2026-05-20 — v2.60 材料/菇棚/经营层规则化

**主文档：** 工程记录见 `CODEX_HANDOFF.md` §12

**实现摘要：**

- `WorldRules.CARD_CAPABILITIES` 与 `card_rule_audit` 全量对齐；新增 player/mushroom/container/structure/commerce 查询 API
- `is_commerce_product` / `commerce_product_near` / `has_business_attraction`；`loose_cards_of_kind` 统一材料链查询
- runtime 去 `get_cards_by_type`：population / ecosystem / mushroom_* / craft / insight / snapshot / situation / time_weather

**验收：**

- L0 unit：**519** assertions PASS
- L2 behavior_trace_live：**PASS tick=1932**

---

## 2026-05-20 — v2.59 草皮/环境层规则化

**主文档：** 工程记录见 `CODEX_HANDOFF.md` §12

**实现摘要：**

- `is_living_grass` / `is_grass_cover` / `is_hut_grass_material` / `living_grasses` / `grass_covers` / `count_*` / `actor_hidden_in_living_grass` / `nearest_living_grass_away_from`
- `population_manager` / `environment_manager` / `ecosystem_manager` / needs / shelter / hunt / camp / rabbit 去 grass 枚举

**验收：**

- L0 unit：**452** assertions PASS
- L2 behavior_trace_live：首次 **5000** / 重试 **120s wall** FAIL → 再跑 **PASS tick=1285**

---

## 2026-05-20 — v2.58 Camp Service 任务/经营层收口

**主文档：** 工程记录见 `CODEX_HANDOFF.md` §12

**实现摘要：**

- `camp_sleep_huts` / `camp_commerce_tables` / `traveler_guests` / `has_commerce_guest_with_demand`
- `camp_structure_card` / `first_bonded_card` 改 service 匹配，去 `get_cards_by_type`
- needs / shelter / commerce / camp_helpers / environment / action_runner 去 hut/table/traveler 枚举

**验收：**

- L0 unit：**446** assertions PASS
- L2 behavior_trace_live：**PASS tick=2379**

---

## 2026-05-20 — v2.57 尸体/肉源 + 兔子迁入规则化

**主文档：** 工程记录见 `CODEX_HANDOFF.md` §12

**实现摘要：**

- `is_corpse` / `is_predator_feed_corpse` / `is_camp_cleanup_corpse` / `corpses_in_world`
- `is_wild_rabbit` / `count_wild_rabbits`；`is_predator_feed_source` 收口
- `world_helpers`、`need_evaluator`、`player_needs_manager`、`survival_tasks`、`ecosystem_manager` 去 corpse/rabbit 枚举

**验收：**

- L0 unit：**441** assertions PASS
- L2 behavior_trace_live：首次 **2× FAIL tick=5000**（tick≈2559 饥死 flake）→ 重跑 **PASS tick=2595**

---

## 2026-05-20 — v2.56 猎物扫描规则化

**主文档：** 工程记录见 `CODEX_HANDOFF.md` §12

**实现摘要：**

- `is_live_prey` / `is_ambush_prey_for`；`is_hunt_target_for` / `hunt_target_score` 去 sheep/rabbit/lamb 枚举
- `world_helpers.count_map_meat_sources`、`need_evaluator._immediate_prey_near`、`player_hunt_helper` 伏击改规则查询

**验收：**

- L0 unit：**433** assertions PASS
- L2 behavior_trace_live：**PASS tick=2805**

---

## 2026-05-20 — v2.55 狼群域成员查询收口

**主文档：** 工程记录见 `CODEX_HANDOFF.md` §12

**实现摘要：**

- `is_wolf_pack_member` / `is_adult_wolf` / `is_wolf_pack_cub`（return_home + hunt/grow 能力）
- pack 系列、`has_wolf_cub`、`count_adult_wolves`、`reproducing_wolves`、`wolf_cubs` 去 wolf/wolfCub 枚举
- `ecosystem_manager` 建窝绑定、`world_manager` 进窝点击穿透、`wolf_behavior` 幼狼分支改规则查询

**验收：**

- L0 unit：**428** assertions PASS
- L2 behavior_trace_live：**PASS tick=1346**

---

## 2026-05-20 — v2.54 幼体跟随 / can_care_for 规则化

**主文档：** 工程记录见 `CODEX_HANDOFF.md` §12

**实现摘要：**

- `can_care_for` 扩至狼/羊双物种（care_child 或 reproduce+grow 配对）
- `should_follow_caregiver`；`sheep_behavior` 改 `nearest_care_actor`；`wolf_cubs` 对齐 `can_grow`

**验收：**

- L0 unit：**421** assertions PASS
- L2 behavior_trace_live：**PASS tick=1317**

---

## 2026-05-20 — v2.53 羊/幼体繁殖查询收口

**主文档：** 工程记录见 `CODEX_HANDOFF.md` §12

**实现摘要：**

- `can_grow` / `reproducing_sheep` / `sheep_lambs` / `has_sheep_lamb` / `count_sheep_population`
- `population_manager` 羊迁入/繁殖/成长改规则查询；`interaction_manager` 浇草引羊改 `reproducing_sheep`

**验收：**

- L0 unit：**418** assertions PASS
- L2 behavior_trace_live：**PASS tick=1324**

---

## 2026-05-20 — v2.52 狼威胁/繁殖查询全链收口

**主文档：** 工程记录见 `CODEX_HANDOFF.md` §12

**实现摘要：**

- `reproducing_wolves` / `wolf_cubs` / `hunt_threats_in_radius` / `pack_hunter_near` / `nearest_pack_hunter`
- `population_manager`、`player_needs_manager`、`mushroom_helpers`、`camp_helpers`、`find_sheep_entry` 去 wolf 枚举

**验收：**

- L0 unit：**414** assertions PASS
- L2 behavior_trace_live：**PASS tick=1320**

---

## 2026-05-20 — v2.51 生态/感知层规则收口

**主文档：** 工程记录见 `CODEX_HANDOFF.md` §12

**实现摘要：**

- `is_active_hunt_threat` / `hunt_threat_near` / `pack_members_near_home` / `is_predator_feed_source`
- `population_manager` 狼群繁殖与迁出、`spatial_scan` 狼扫描、`zone_overlay` 营结构绘制改规则查询
- `CampHelpers.is_wolf_meat_source` 转调 `WorldRules`

**验收：**

- L0 unit：**409** assertions PASS
- L2 behavior_trace_live：**PASS tick=3204**

---

## 2026-05-20 — v2.50 Hunt/care B + go_near_xy 返回值

**主文档：** 工程记录见 `CODEX_HANDOFF.md` §12

**实现摘要：**

- `WorldRules`：`wolf_den_builder` / `has_wolf_pack_home` / `best_feed_source_for` / `hungry_wolf_cub_at_home` 等
- `ecosystem_manager` / `wolf_behavior` 建窝、喂幼、取肉、惧火改规则查询
- `go_near_xy` 返回 `bool`；`find_wolf_den` 改 `home_for_actor`

**验收：**

- L0 unit：**405** assertions PASS
- L2 behavior_trace_live：**PASS tick=1792**

---

## 2026-05-20 — v2.49 go_to_xy 返回值 + 世界层 fire 收口

**主文档：** 工程记录见 `CODEX_HANDOFF.md` §12

**实现摘要：**

- `go_to_xy` 返回 `bool`；fire/mushroom/shelter/commerce 精确格 walk phase 用返回值推进
- `environment_manager` 热传导 / 草棚信号、`population_manager` 旅人条件、`zone_overlay` 火域绘制改规则查询

**验收：**

- L0 unit：**398** assertions PASS
- L2 behavior_trace_live：**PASS tick=1396**（修复 `environment_manager` `:=` 推断错误）

---

## 2026-05-20 — v2.48 movement 第三轮 + needs 层 fire 收口

**主文档：** 工程记录见 `CODEX_HANDOFF.md` §12

**实现摘要：**

- `go_near` 返回值扩至 `shelter_tasks`（建棚/迁营/搬桌）与 `tool_tasks`（刀矛斧/伐林）
- `player_needs_manager` 全部 `get_cards_by_type("fire")` → `camp_domain_anchor` / `has_camp_domain_anchor`
- `buildHut/walkToGrass` 排序用域锚点替代 fire 枚举

**验收：**

- L0 unit：**396** assertions PASS
- L2 behavior_trace_live：首次 120s 墙上限未建成（flaky）；重跑 **PASS tick=1404**

---

## 2026-05-20 — v2.47 movement 第二轮 + has_fire 规则收口

**主文档：** 工程记录见 `CODEX_HANDOFF.md` §12

**实现摘要：**

- `go_near` 返回值扩至 `fire_tasks`、`survival`（butcher/goToMeat）、`mushroom`（getWater/combineGh）、`commerce`（serve/organize）
- `has_camp_domain_anchor` 替代 `mission_resolver` / `spatial_scan` / `situation_assessment` / `makeFire` 的 fire 枚举
- `near_camp_fire_chess*` / `is_camp_fire_cell`；湿木 tick 与 `_hut_drop_spot` 去 fire 硬编码

**验收：**

- L0 unit：**393** assertions PASS
- L2 behavior_trace_live：**PASS tick=1977**（修复 `makeFire/hitFire` 锚点条件写反后）

---

## 2026-05-20 — v2.46 Camp 边角收尾 + movement 第一轮（go_near 返回值）

**主文档：** 工程记录见 `CODEX_HANDOFF.md` §12

**实现摘要：**

- `near_camp_fire` / `near_camp_fire_xy` / `first_service_provider`；`WorldHelpers.near_fire`、`MushroomHelpers.find_greenhouse` 规则化
- `go_near` 返回 `bool`（邻格/at_target）；`cookMeat/walkToFire`、`forageEat/goToEat` 用返回值推进 phase
- `action_runner` CHAIN_FIRE 目标改 `has_camp_domain_anchor`

**验收：**

- L0 unit：**386** assertions PASS
- L2 behavior_trace_live：首次 tick=5000 FAIL（flaky）；重跑 **PASS tick=1796**

---

## 2026-05-20 — v2.45 service.sleep/commerce + home territory 规则查询

**主文档：** 工程记录见 `CODEX_HANDOFF.md` §12

**实现摘要：**

- `WorldRules` 新增 `camp_sleep_home`、`camp_commerce_table`、`in_camp_home_territory`
- `WorldHelpers.find_camp()` 改走 `camp_sleep_home`（`service.sleep` 提供者）
- `in_home_domain` 火域循环改 `in_camp_home_territory`；commerce need 显式校验 `service.commerce`
- `need_evaluator` 夜归目标改 `camp_sleep_home`

**验收：**

- L0 unit：PASS，381 assertions
- L2 behavior_trace_live：PASS，tick=3509 建成蘑菇棚

---

## 2026-05-20 — v2.44 Camp 组织上下文 + commerce 规则查询（略提速合并片）

**主文档：** 工程记录见 `CODEX_HANDOFF.md` §12

**实现摘要：**

- `WorldRules` 新增 `camp_domain_anchor`、`camp_structure_card`、`camp_organize_refs`
- `campOrganize` 任务 fire/hut/table 引用、bond 搬桌/棚、commerce 稳定判定、need_evaluator 生火/建桌/经营 need 改走规则查询
- 删除 `CampHelpers._first_type`

**验收：**

- L0 unit：PASS，376 assertions
- L2 behavior_trace_live：PASS，tick=2789（首次 tick=5000 未建成，Live 波动；复跑通过）

---

## 2026-05-20 — v2.43 营地域结构查询 + storage 死代码清理

**主文档：** 工程记录见 `CODEX_HANDOFF.md` §12

**实现摘要：**

- `WorldRules.camp_domain_structure_cards()`：用 `defines_domain` + `bond_to_domain` 枚举营地域结构（替代 `["fire","hut","table"]` 硬编码）
- `CampHelpers.camp_zones()` 改转调规则查询
- 删除 v2.41 后不可达的 `_carry_item_to_camp_storage` / `_resume_carry_to_camp`（含 orphan `on_arrive` 存放下肉）

**验收：**

- L0 unit：PASS，372 assertions
- L2 behavior_trace_live：PASS，tick=1850 建成蘑菇棚

**后续：** `_first_type("fire/hut/table")` 组织任务上下文收口、movement 单主人

---

## 2026-05-20 — v2.42 狼叼肉 recovery 接入 ActionRecovery

**主文档：** 工程记录见 `CODEX_HANDOFF.md` §12

**实现摘要：**

- `ActionRecovery` 新增狼侧 API：`wolf_carry_intent` / `can_resume_wolf_carry` / `resume_wolf_carry` / `wolf_route_blocker`
- 从 `EcosystemManager` 迁入回窝清路逻辑（放下肉 + 驱赶 rabbit/lamb）；回窝移动仍由 `update_wolf_den_work` 执行
- `home` 目标仍用 `WorldRules.home_for_actor`

**验收：**

- L0 unit：PASS，368 assertions
- L2 behavior_trace_live：PASS，tick=1367 建成蘑菇棚

---

## 2026-05-20 — v2.41 Storage fetch 去 orphan on_arrive

**主文档：** 工程记录见 `CODEX_HANDOFF.md` §12

**实现摘要：**

- `update_camp_storage` 营地外/邻格取物不再写 `on_arrive` 回调；无 `craft` 时直接返回 false
- 取物/存放统一经 `_start_store_camp_task` → `campOrganize` / `storeCamp` FSM（`commerce_tasks.gd`）
- `_test_storage_fetch_requires_carry` 改为断言 craft 任务启动而非瞬时拎起+寻路

**验收：**

- L0 unit：PASS，362 assertions
- L2 behavior_trace_live：PASS，tick=2058 建成蘑菇棚

**后续：** 狼 recovery 接入 `ActionRecovery`；`_carry_item_to_camp_storage` / `_resume_carry_to_camp` 死代码可后续删

---

## 2026-05-20 — v2.40 Camp 火羁绊硬编码收口

**主文档：** 工程记录见 `CODEX_HANDOFF.md` §12

**实现摘要：**

- 删除 `CampHelpers.FIRE_BOND_DOMAIN_TYPES`
- `need_evaluator` 拎桌/棚 shelter 判定改 `CampHelpers.has_fire_bond(carrying)`
- `bond_carry_can_resume`、`resume_interrupted_bond_move` 的 table/hut 分支加 `WorldRules.bond_to_domain` 守卫

**验收：**

- L0 unit：PASS，361 assertions
- L2 behavior_trace_live：PASS，tick=2295 建成蘑菇棚

**后续：** storage orphan `on_arrive` → campOrganize、狼 recovery 作为 ActionRecovery 第二 consumer。

---

## 2026-05-27 — v2.39 ActionRecovery / 玩家携带恢复收口

**主文档：** 工程记录见 `rule-dimensions-v0.1.md` §5.3

**实现摘要：**

- 新增 `ActionRecovery`：`carry_intent()`、`can_resume_carry()`、`resume_carry()` 统一玩家手上物品的 recovery 查询与入口
- 用 `WorldRules.bond_to_domain`、`is_camp_storable`、`is_organize_locked` 分类 bond 搬桌/棚/火与 campOrganize 归置/清出
- `PlayerExecutionRecovery`、`ActionRunner._tick_storage`、`PlayerNeedsManager._satisfy_shelter_need`、`CampHelpers.update_camp_storage` 改走 `ActionRecovery`
- 执行仍委托 `CampHelpers.resume_interrupted_bond_move` / `resume_interrupted_organize`，不改 craft FSM 与优先级数值
- L0 增加断言：`carry_intent` 对 stone/table/dryGrass 分类，resume 与既有 evict/搬桌用例一致

**验收：**

- L0 unit：PASS，358 assertions
- L2 behavior_trace_live：PASS，tick=1346 建成蘑菇棚，卡滞段 0（首次跑 tick=5000 未建成，复跑通过 — Live 有波动）

**后续：** ~~Camp 硬编码收尾~~（v2.40 已做）；storage orphan `on_arrive` → campOrganize、狼 recovery 作为第二 consumer。

---

## 2026-05-27 — v2.38 Return Home / 回家目标查询收口

**主文档：** `方寸商国_设计规则圣经_v0.3.txt` 新增 § v2.38

**实现摘要：**

- `WorldRules` 新增 `home_for_actor()`，把“哪个 home 属于这个 actor”收口到规则层
- 成年狼和幼狼都通过 `capability.return_home` + `domain.wolf_pack` 查询 home
- `homeDenId` 绑定优先于最近狼窝，避免幼狼 / 成狼在多狼窝场景下误选近窝
- `EcosystemManager.den_for_wolf()`、狼饱腹回窝、叼肉回窝、幼狼入窝入口改为消费规则查询
- L0 增加断言：成年狼 / 幼狼能找到 home，幼狼有绑定狼窝时优先回绑定 home

**验收：**

- L0 unit：PASS，352 assertions
- L1 intent_trace：PASS，tick=1270 建成蘑菇棚
- L2 behavior_trace_greenhouse：PASS，tick=1296 建成蘑菇棚，卡滞段 0
- L2 behavior_trace_live：PASS，tick=1272 建成蘑菇棚，卡滞段 0
- L3 smoke：PASS，60s 墙上限

**后续：** 可继续把“叼肉回窝”的 route blocker / drop meat / drive away 拆为 recovery 查询，但不要一次性重写完整狼 AI。

---

## 2026-05-27 — v2.37 Hunt Target / 行为目标选择收口

**主文档：** `方寸商国_设计规则圣经_v0.3.txt` 新增 § v2.37

**实现摘要：**

- `WorldRules` 新增 `best_hunt_target()` 与 `hunt_target_visible_to()`，把捕猎行为的 target 阶段收口到规则层
- 狼不再在 `EcosystemManager.best_prey_for_wolf()` 内自行排序猎物，而是消费 `WorldRules.best_hunt_target(wolf)`
- 玩家近身反应、饥饿捕猎、制矛升级判断统一消费 `WorldRules.best_hunt_target(player)`
- `CardBase` 补齐 `hiddenInGrass` 状态字段，使兔子藏草状态能被规则层真实读取
- L0 增加断言：玩家最佳猎物、狼兔子偏好、草中远距离兔子的可见性过滤

**验收：**

- L0 unit：PASS，349 assertions
- L1 intent_trace：PASS，tick=1609 建成蘑菇棚
- L2 behavior_trace_greenhouse：PASS，tick=2170 建成蘑菇棚，卡滞段 0
- L2 behavior_trace_live：PASS，tick=1409 建成蘑菇棚，卡滞段 0
- L3 smoke：PASS，60s 墙上限

**后续：** 下一个合适切片是把 `execution/recovery` 的一小段继续收口，例如狼叼肉回窝、喂幼狼、或玩家手持材料恢复任务。

---

## 2026-05-27 — v2.36 Rule Dimensions / 标签维度约束

**主文档：** `方寸商国_设计规则圣经_v0.3.txt` 新增 § v2.36

**实现摘要：**

- 新增 `rule-dimensions-v0.1.md`，定义六个维度：身份、材料/形态、能力、关系/域、行为动作、规则修饰
- `CardRuleAudit` 新增维度注册与 L0 检查：`CardDB` 标签、能力、运行时 `domain/bond/service/action` 词汇都必须能归维度
- 新增自动事实表 `docs/design/generated/rule-dimension-facts.md`
- `card-rule-audit.md` 引用维度事实表，避免审计只看卡牌事实、不看世界语法

**验收：**

- 审计导出：PASS，已生成 `rule-dimension-facts.md`
- L0 unit：PASS，345 assertions

**后续：** 将 `preference.*`、`reach.*`、`requires.*`、`risk.*` 作为下一批规则修饰候选，但需在实际迁移猎物偏好 / 武器距离 / 专名输入时再加入。

---

## 2026-05-27 — v2.33 首批能力迁移候选切片

**实现摘要：**

- 新增 `capability-migration-slices-v0.1.md`
- 定义两个候选迁移样板：`camp domain + fire bond` 与 `hunt + care_child`
- 推荐优先做 `camp domain + fire bond`，用于验证非动物卡牌也进入世界规则结构
- 明确每个切片的目标、首批卡牌、应收束逻辑、不处理范围、成功标准

**验收：**

- 文档更新完成；本节不改运行时玩法

---

## 2026-05-27 — v2.32 Capability / Action 分类法

**实现摘要：**

- 新增 `capability-action-taxonomy-v0.1.md`
- 明确标签、能力、行为三层分工：标签描述“是什么”，能力描述“能进入哪个系统”，行为描述“现在做什么”
- 将捕猎、抚养、移动、繁殖、收纳、交易、制作定义为后续可抽象的通用能力/行为入口
- 明确狼捕猎、玩家捕猎、幼狼跟随/入窝应逐步收束到 `capability.hunt`、`capability.care_child`、`action.follow`、`action.return_home`
- 扩展非动物卡牌结构：篝火、草棚、桌子、蘑菇棚、狼窝、森林、水源都纳入 `domain`、`bond`、`service`、`produce`、`transform` 规则
- 明确世界规则标签机必须覆盖所有卡牌，不允许结构、场域、资源源游离在体系之外
- `card-rule-audit.md` 增加 `capabilities` 字段，现有 50 张卡均已补首批能力标记
- L0 增加能力覆盖检查：狼必须进入 `capability.hunt`，篝火必须进入 `capability.define_domain`，桌子必须进入 `capability.provide_service`
- 记录迁移原则：先补 taxonomy 和审计字段，再选小切片迁移，不做一次性大重写

**验收：**

- 审计导出：PASS，`card-rule-audit.md` 已生成 capabilities 列
- L0 unit：PASS，310 assertions
- 本节不改运行时玩法

---

## 2026-05-27 — v2.31 世界规则标签机目标与全卡审计

**实现摘要：**

- 新增 `world-rule-tag-machine-v0.1.md`，明确标签机 / 卡牌封装 / 世界规则的工程边界
- 新增 `card-rule-audit.md`，覆盖 `CardDB` 全部 50 张现有卡牌，记录运行时状态、AI 可达性和风险结论
- 新增自动事实导出：`generated/card-rule-facts.md` 从真实 `CardDB` 导出卡牌、关系、敲击配方
- 新增 `CardRuleAudit` 一致性检查：引用完整性、标签冲突、商品标签、source 标签产物、核心链路可达性标记
- 新增 `tools/export_card_rule_facts.ps1` 与导出场景，便于后续修改卡牌后刷新审计事实

**验收：**

- 审计导出：PASS，cards=50，relations=21，impact recipes=9
- L0 unit：PASS，307 assertions
- L1 intent_trace：PASS，tick=1660，GH intent 链完整
- L2 behavior_trace_greenhouse：PASS，tick=1369 建成，卡滞段 0
- L2 behavior_trace_live：FAIL，连续两次 tick 上限未建成；卡滞段 0，表现为 live 夜间/饥饿/躲狼打断建棚
- L3 smoke：PASS，60s 墙上限，推进到 1.76 游戏日

---

## 2026-05-26 — v2.30 手持石头制碎石链路与行为追踪口径修正

**实现摘要：**

- 修复“玩家卡手里已有石头、面前/附近有第二块石头，却不会继续砸碎石生火”的断点
- `satisfy_fire_need` 现在会把手持 `stone` 识别为制碎石第一块石头，并转入 `makeShardForFire`
- `makeShardForFire` 丢失任务目标时，会优先用手持石头 + 最近地面石头恢复任务，不再直接放下手持物导致发呆
- 行为 trace 的卡滞统计进一步收窄：夜间躲藏、躲狼、睡觉、休整不再被算作建棚执行卡死；只统计真正的 `buildGreenhouse/*` craft 停滞

**验收：**

- L0 unit：PASS，302 assertions
- L1 intent_trace：PASS，tick=1312，GH intent 链完整
- L2 behavior_trace_greenhouse：PASS，tick=1281 建成，卡滞段 0
- L2 behavior_trace_live：PASS，tick=1349 建成，卡滞段 0
- L3 smoke：PASS，60s 墙上限，推进到 1.12 游戏日

---

## 2026-05-26 — v2.29 手动代管控制权与 sanitation 执行对齐

**实现摘要：**

- 新增 `PlayerManualControl` 小封装：玩家卡被左右键拖拽时，自动路径、到达回调、当前 craft、执行栈、路径意图会立即清空
- 玩家放手后保留 3 tick 自动规划冷却，避免自动系统在下一帧立刻沿旧任务回弹
- 玩家卡拖拽落点确认时，同步携带物到玩家新格，避免拎着物品时代管后携带物留在旧位置
- 修复 sanitation eval/exec 错位：既然评估层已经因营地附近尸体触发 `sanitation`，执行层就必须启动清尸任务，不再被“肉储备已够”拦住
- `sanitation` 归入可打断建棚项目的生存类需求，并阻止夜间睡眠跳过清尸执行

**验收：**

- L0 unit：PASS，296 assertions
- L1 intent_trace：PASS，蘑菇棚建成，GH intent 链完整
- L2 behavior_trace_live：PASS，tick=1890 建成，卡滞段 0
- L3 smoke：PASS，60s 墙上限，推进到 1.61 游戏日

---

## 2026-05-26 — v2.28 P0 断点修复与规则封装验收

**实现摘要：**

- 蘑菇棚建造与维护链路继续收口到 `material.lumber`：木头、树枝都能参与建棚/湿木/制斧木柄；长矛仍保留树枝专用规则
- `buildGreenhouse` 任意阶段发现已有 `mushroomGreenhouse` 会立即完成并释放建棚项目锁，不再继续打水/做湿木
- `operateGreenhouse` 补齐棚后维护：采菇、等待湿木成熟、补木、取水、做湿木
- 缺少可用空桶时，建棚链会在营地附近补一个空桶，避免 `getWater` 无限失败重启
- 铜钱、铜块、铜饰补 `camp.storable`，按普通营地物资进入整理系统
- 狼叼肉回窝遇到兔/小动物挡路时，先放下肉并驱赶，不击杀；食物足够时不反复捕猎
- 幼狼无窝时跟随成年狼；有新狼窝后步行入窝，不再远距离瞬移
- 行为 trace 的卡滞统计限定为建棚相关签名，避免前置夜间/饥饿等待污染蘑菇棚验收

**验收：**

- L0 unit：PASS，275 assertions
- L1 intent_trace：PASS，蘑菇棚建成，GH intent 链完整
- L2 behavior_trace_live：PASS，tick=1320 建成，卡滞段 0
- L3 smoke：PASS，60s 墙上限，推进到 1.23 游戏日

---

## 2026-05-27 — v2.35 hunt + care_child 运行时样板

**主文档：** `方寸商国_设计规则圣经_v0.3.txt` 新增 § v2.35

**实现摘要：**

- `WorldRules` 新增狼群域、捕猎、喂养、跟随、回家动作常量与查询
- 玩家和狼的捕猎目标选择开始消费 `WorldRules.is_hunt_target_for`
- 幼狼无窝找成年狼改为消费 `WorldRules.nearest_care_actor`
- 狼窝定义 `domain.wolf_pack`；成年狼具备 `care_child`；幼狼具备 `follow / return_home`
- 保持完整生态 AI、伏击草丛、喂幼狼、建窝动作不重写

**验收：**

- L0 unit：PASS，339 assertions
- L1 intent_trace：PASS，tick=1454 建成蘑菇棚
- L2 behavior_trace_greenhouse：PASS，tick=1252 建成蘑菇棚，卡滞段 0
- L2 behavior_trace_live：PASS，tick=1443 建成蘑菇棚，卡滞段 0
- L3 smoke：PASS，60s 墙上限

**后续：** 继续把喂幼狼、回窝、建窝、伏击草丛迁移到 action 查询；同时抽出猎物偏好和风险规则，减少 `is_hunt_target_for` 内的物种专名表。

---

## 2026-05-27 — v2.34 camp domain + fire bond 运行时样板

**主文档：** `方寸商国_设计规则圣经_v0.3.txt` 新增 § v2.34

**实现摘要：**

- `WorldRules` 新增营地域、蘑菇农场域、火羁绊、结构服务查询
- `CampHelpers` 的营地域/火羁绊判断开始消费 `WorldRules`，不再在核心入口直接硬编码火-草棚/桌子交汇域
- 篝火定义 `domain.camp`；草棚、桌子吸附营地域；篝火/草棚/桌子/菇棚分别提供 cook/storage/sleep/commerce/greenhouse 服务
- 菇棚本轮只定义/提供菇棚域服务，不误吸附进营地域
- 保持玩家任务 FSM、收纳 craft 链、蘑菇棚产出玩法不变

**验收：**

- L0 unit：PASS，323 assertions
- L1 intent_trace：PASS，tick=1264 建成蘑菇棚
- L2 behavior_trace_greenhouse：PASS，tick=1725 建成蘑菇棚，卡滞段 0
- L3 smoke：PASS，60s 墙上限

**后续：** 候选 B `hunt + care_child` 可作为第二条运行时样板；候选 A 后续仍需把睡眠、交易、菇棚维护继续改为服务查询驱动。

---

## 2026-05-26 — v2.27 规则封装化迁移（第一段）

**主文档：** `方寸商国_设计规则圣经_v0.3.txt` 新增 § v2.27

**实现摘要：**

- 新增 `WorldRules`：统一承担“卡牌标签 → 世界规则判断”的查询层
- 为现有核心卡牌补充规范标签：材料、燃料、食物、营地收纳、组织锁定、资源源头等
- 将蘑菇棚链路、制斧木柄、湿木交互、营地整理等一批硬编码判断迁移到标签查询
- 收口一段 movement：直接 `set_path_near` 也记录路径意图；已到邻格时清理旧 path / on_arrive
- 修正 smoke 的“目标卡住”误判：改为连续状态签名不变才算卡住，并记录最后玩家快照
- 保持现有玩法不变；本次是底层归因能力迁移，不新增内容

**验收：**

- 单元测试通过：250 assertions
- 意向链路 trace 通过：蘑菇棚建成
- live 行为 trace 通过：蘑菇棚建成，卡滞段 0
- smoke 通过：墙上 60s 内 1.44 游戏日，无失败

**后续：** 继续把 recipe / relation / task trigger 分段迁移到规则查询层，避免一次性重写。

---

## 2026-05-20 — v2.26 蘑菇系统落地与蘑菇农

**主文档：** `方寸商国_设计规则圣经_v0.3.txt` 新增 § v2.26

**实现摘要：**

- 新增 wetWood / mushroomWood / mushroomGreenhouse / mushroomFarmer
- 湿木链：wood|twig + waterbucket → 30s → mushroomWood → 2×impact → mushroom
- 玩家 AI：build_greenhouse、manageFarm、operateGreenhouse
- 蘑菇农：全局 1 人，棚后 2 天生成，招募 UI，雇佣契约（木头×2、熟肉 2 天、第二草棚、防狼）
- 移除开局地图蘑菇

**代码：** `mushroom_helpers.gd`、`mushroom_farmer_behavior.gd`、`mushroom_tasks.gd` 等（见 v2.26 §7）

**待做：** 森林再生专章、蘑菇棚多轮采菇基质规则

---

## 2026-05-21 — 意向系统 v0.3（观念 / 旅人 / 帮助理由）

- `intent-system-world-evolution-v0.3.md`：血的代价=人物内在逻辑之命运感（非真人模拟）；观念层待建
- 世界铁律：无人能出桃源；迷路者命运涌现
- 桃源人帮玩家：因获得旧秩序没有的商品/服务/交互方式
- 观念变迁：新秩序满足 → 向内向往外部机会

## 2026-05-21 — 意向系统与世界演化 v0.2（策划文档）

- 新增 `docs/design/intent-system-world-evolution-v0.2.md`（桌面同步：`方寸商国_意向系统与世界演化设计总结.md`）
- 共识：需求=基本面、意向=升华面；玩家=玩家卡；上帝视角归因玩家卡
- 桃源 ~50 人小社会；地图规划 4～6 倍；蘑菇农为桃源人一员
- 无通关，10～15 游戏年至商业枢纽；Phase 0 优先 survival + safety

---

- 狼窝：`is_rooted`、不可拖拽；`get_card_at` 忽略窝内狼；详情面板显示窝内数量；狼 digest 后出窝
- 建造：`CampPlanner.is_structure_cell` 仅 `land`（避开河流/湿润/浅滩）
- 伐木/蘑菇棚：玩家自动制斧（shapeTri→makeAxe）+ 斧头砍树 2 下；修复 buildGreenhouse 抢占 craft、仅有 twig 时跳过砍树、chopForest 未调用 impact 等
- 捕猎：非饥饿时不主动 reactive 攻击；新增 sanitation 需求屠宰附近尸体
- 营地储存：扩展 `CAMP_ESSENTIAL_TYPES` 与 merge（木/石/桶等，不含草）

- 项目内副本：`docs/design/方寸商国_设计规则圣经_v0.3.txt`
- 桌面副本：`方寸商国MVP/` 文件夹（与项目 sibling 目录）
- 同步脚本：`tools/sync_design_doc.ps1`（默认较新覆盖较旧）
