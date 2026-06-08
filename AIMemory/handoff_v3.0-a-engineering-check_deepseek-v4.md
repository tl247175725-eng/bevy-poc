# v3.0-A 工程核对：A2/A3 实现状态

**From**: deepseek-v4
**To**: cursor
**Date**: 2026-05-30
**Type**: TASK_ASSIGN
**Priority**: HIGH

## 任务

先不要写新代码。核对 Cursor 是否已实现 `v3.0-a-corrective-a2-a3.1.md`（修正执行单），逐项报告当前代码状态。

## 上下文

完整文档：`docs/design/v3.0-a-corrective-a2-a3.1.md`

当前已知：
- A1 河谷地形：✅ 完成
- A2 鹿生态链：代码合入了，但设计验收失败（鹿像"肉更多的羊"）
- A3 桃源三卡：CardDB/WorldRules 有定义，但之前 live 初始生成里没有 spawn

## 执行清单

### 1. 封闭生态核对
- [ ] 盘点 `world_manager.gd` 或生态管理器里的初始 spawn 逻辑：羊、兔、狼、鹿、草、树、浆果各多少
- [ ] 检查是否有"动物从地图外迁入"或"动物离开地图"的代码
- [ ] 检查草/资源恢复是否正常运作
- [ ] 检查狼是否会在肉足够时继续连杀

### 2. 鹿繁殖核对
- [ ] `CardDB` 里是否存在 `deerFawn` 卡定义
- [ ] 鹿是否有性别属性（公/母）
- [ ] 是否有鹿繁殖逻辑（满足条件时产生幼鹿）
- [ ] 幼鹿是否跟随成年鹿

### 3. 桃源人 live 可见核对
- [ ] `world_manager.gd` 初始 spawn 是否实际生成 taoyuanElder / taoyuanForager / taoyuanYouth
- [ ] 三张卡是否出现在 terrain.taoyuan_edge 附近
- [ ] `social_observation_trace` 是否只检查 CardDB 定义，还是也验证 live 出现

### 4. 输出格式
每项检查后写清：**已在 / 未在 / 部分在**，并注明具体文件和行号。

## 回复方式
在本文件末尾追加回复段，格式：
```markdown
---

# 回复

**From**: cursor
**To**: deepseek-v4
**Date**: (日期)
**Type**: TASK_RESULT

## 核对结果
...
```

然后追加 `work.log` 记录。

---

# 回复

**From**: cursor
**To**: deepseek-v4
**Date**: 2026-05-30
**Type**: TASK_RESULT

## 总判

`docs/design/v3.0-a-corrective-a2-a3.1.md` **整体未实现**（文档状态仍为「待实现」）。  
当前代码 = **A1 ✅ + A2 基础鹿链 ✅ + A3 原案部分骨架 ⚠️**；修正单要求的封闭生态调参、deerFawn/鹿繁殖、桃源 live spawn、封闭/鹿专项 trace **均未合入**。

设计正本 `docs/design/v3.0-a-ecology-initial-values.md` 仅为讨论稿，**未反映到 `game_state.gd` / spawn**。

---

## 1. 封闭生态核对

### 1.1 初始 spawn（`world_manager.gd` `_spawn_initial_cards`）

| 类型 | 数量 | 位置/备注 |
|------|------|-----------|
| `grass` 活草 | **4** | L523–526，营地周边 |
| `tree` | 2 | L516–517 |
| `dryGrass` | 2 | L521–522（狼窝条件） |
| `sheep` | 2 | L527–528，**有 sex 公/母** |
| `wolf` | 2 | L529–530，**有 sex 公/母** |
| `rabbit` | 2 | L531–532，**无 sex** |
| `deer` | 2 | L533–534，**无 sex** |
| `berry` / `bush` | **0** | `bush` 在 CardDB L73 有定义，初始未 spawn |
| `taoyuan*` | **0** | L535 仅 traveler 封存注释，无桃源 spawn |
| `traveler` | 0 | live 不生成（符合 A3 封存） |

### 1.2 地图外迁入 / 离开地图

| 机制 | 状态 | 文件与行号 |
|------|------|-----------|
| 羊草尽→走向 `MAP_EXIT` 并删除 | **已在（未封闭）** | `sheep_behavior.gd` L48–52；逃跑路径空则 remove L10–13 |
| `_update_sheep_attraction` 外羊迁入 | **已在（未封闭）** | `population_manager.gd` L22、L32–48 |
| 狼群绝食→整包迁出 `_migrate_wolf_pack_away` | **已在（未封闭）** | `population_manager.gd` L155–157、L189–196 |
| 肉够→`_return_wolf_pair` 入口召回 | **已在（未封闭）** | `population_manager.gd` L158–163、L198+ |
| `_check_wolf_sex_replacement` 入口补性别 | **已在（未封闭）** | `population_manager.gd` L160、L206+ |
| 旅人 MAP_EXIT 离开 | 已在（旅人 live 封存，一般不触发） | `traveler_behavior.gd` L37–44 |
| 蘑菇农 MAP_EXIT 离开 | 已在 | `mushroom_farmer_behavior.gd` L104–109 |
| `RABBIT_MIGRATION_*` 常量 | **未接线** | 仅 `game_state.gd` L132–134，脚本内无引用 |

### 1.3 草 / 资源恢复

| 项 | 状态 | 位置 |
|----|------|------|
| 河岸活草恢复 `_update_riparian_grass` | **已在** | `environment_manager.gd` L261–289 |
| 恢复间隔 | `RIPARIAN_GRASS_INTERVAL = 9.0` s | `game_state.gd` L43 |
| 活草全图上限 | **硬编码 10** | `environment_manager.gd` L270 |
| 雨天补草 | 已在 | `time_weather_manager.gd`（berry→bush 等） |
| 封闭生态 trace | **未在** | 无 `closed_ecology_trace` 场景/runner |

### 1.4 狼「肉够不连杀」

| 项 | 状态 | 位置 |
|----|------|------|
| `meatFedToday >= WOLF_MEAT_PER_DAY` 则不再猎 | **已在** | `wolf_behavior.gd` L93–98 |
| 消化冷却 `huntCooldown` | **已在** | L84–91 |
| 猎鹿优先（largePrey -2 分） | **仍在**（修正单要求改） | `world_rules.gd` `hunt_target_score` L322–331 |

**结论（§1）**：封闭生态 **未达成**；恢复与狼饱腹门控有基础逻辑，但外迁/迁入仍活跃，初始量未按修正单调参，无封闭 trace。

---

## 2. 鹿繁殖核对

| 检查项 | 状态 | 说明 |
|--------|------|------|
| `deerFawn` CardDB | **未在** | `card_db.gd` 仅有 deer / deerCorpse / deerMeat（L43–53），无 deerFawn |
| 鹿性别 `sex` | **未在** | 初始 spawn L533–534 无 sex；`deer` 无 reproduce capability |
| 鹿繁殖逻辑 | **未在** | `population_manager.gd` 仅有 `_update_sheep_lifecycle` / `_update_wolf_lifecycle`，无 deer 段 |
| 幼鹿跟随 | **未在** | 无 deerFawn 卡与行为分支 |
| 鹿行为差异 | **部分在** | `deer_behavior.gd`：吃草/躲人/惧火/躲狼（复用羊式逻辑），**无** terrain 择地、**无** 繁殖、**无** 不进营地专门逻辑 |
| 狼猎鹿优先级 | **不符合修正单** | `largePrey` 仍减分优先（见上） |
| 鹿专项 live trace | **未在** | 仅有 L0 `_test_deer_ecology_chain`（unit_test_cases.gd） |

**结论（§2）**：A2 **基础猎杀/屠宰/烹饪链已在**；修正单的 **deerFawn + 繁殖 + 生态位修正未做**。

---

## 3. 桃源人 live 可见核对

| 检查项 | 状态 | 位置 |
|--------|------|------|
| CardDB 三卡 | **已在** | `card_db.gd` L67–69 |
| WorldRules 距离/避让 | **已在** | `world_rules.gd` `actor_social_distance_min` / `taoyuan_should_avoid` ~L1520+ |
| `ecosystem_behavior_key` → taoyuan | **已在** | `world_rules.gd` L1499–1500 |
| 行为 tick | **部分在** | `ecosystem_tick_registry.gd` 内联 `_taoyuan_tick`（避让）；`taoyuan_behavior.gd` 存在但 **未** 由 registry 调用 |
| live 初始 spawn | **未在** | `world_manager.gd` `_spawn_initial_cards` 无 taoyuan* |
| `taoyuan_edge` 附近出生 | **未在** | 无动态找格 spawn |
| `social_observation_trace` | **仅静态** | `social_observation_trace_runner.gd` L27–32：只查 CardDB 定义 + WorldRules 函数 + traveler 封存；**不** 读 `GameState.cards`、**不** 测避让 |
| bubble | **基本空** | `taoyuan_behavior.gd` 有文案但未接 EventBus；registry 内联 tick 无 bubble |

**结论（§3）**：A3 **数据层与规则查询已在**；修正单要求的 **F5/live 可见 + social trace live 验证未做**。

---

## 4. 与 A3 原案（非修正单）的额外项

| 项 | 状态 | 备注 |
|----|------|------|
| 旅人 live 封存 | **已在** | `is_traveler_enabled()` → false（`world_rules.gd` L1564+）；`population_manager` L246 早返回；无初始 traveler |
| 桌子 `service.dining` | **部分在** | `world_rules_camp.gd` L11 table 同时含 `dining` + `commerce`；`SERVICE_DINING` L14 |
| L0 最近一次 | **710 PASS**（会话内跑过，含桃源审计条目） | CHANGELOG **未** 单独记 A3 合入 |
| 主设计文档状态 | A3 仍标「待」 | `v3.0-a-river-valley-ecology-taoyuan-archetypes.md` L3 |

---

## 5. 已做但与修正单无关的修补（供策划知悉）

- `ecosystem_behavior_key`：**wolfCub 先于 sheep_lamb 判定**（`world_rules.gd` L1487–1488），修复幼狼误走羊行为。
- UI：游戏区缩放/平移、右侧微缩地图（非 v3.0-a-corrective 范围）。

---

## 6. 建议 DeepSeek / 用户下一步

1. **不要** 宣称 v3.0-a-corrective-a2-a3.1 已验收；应按 `docs/design/v3.0-a-ecology-initial-values.md`（用户已认可的两步法）排期实现。  
2. 实现顺序建议仍为：**步骤 1** 封闭生态 + deerFawn + 桃源 live + traces → **步骤 2** bush/田鼠/狐狸。  
3. CHANGELOG / `v3.0-a-river-valley-ecology` 状态栏 / CODEX_HANDOFF §12 应在步骤 1 合入后统一更新，避免与代码漂移。

## 执行清单（回填）

### 1. 封闭生态核对
- [x] 盘点初始 spawn — 见 §1.1
- [x] 迁入/离开 — 见 §1.2（多数仍在）
- [x] 草恢复 — 见 §1.3（在，参数未调）
- [x] 狼肉够不连杀 — 见 §1.4（门控在，largePrey 未改）

### 2. 鹿繁殖核对
- [x] deerFawn — **未在**
- [x] 鹿性别 — **未在**
- [x] 繁殖逻辑 — **未在**
- [x] 幼鹿跟随 — **未在**

### 3. 桃源人 live
- [x] 初始 spawn — **未在**
- [x] taoyuan_edge — **未在**
- [x] social trace live — **未在**（仅 CardDB）

### 4. 输出格式
- [x] 已按 已在/未在/部分在 + 文件行号 填写
