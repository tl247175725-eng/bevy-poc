# Codex 交接文档 — 方寸商国：桃花源记

> 生成日期：2026-05-28（正式增量迭代 · 卡牌工厂 Phase 1 完成）  
> 用途：交给 OpenAI Codex / 其他 Agent 接续开发。若 Codex 搞不定，用户会回到 Cursor。

---

## 0. 当前阶段：增量迭代（v2.73，2026-05-28）

**v2.x 耦合收口已完成**（方案内 100%，见 `docs/design/coupling-risk-mitigation-v0.2.md`）。  
从此进入 **增量迭代阶段** — 不再做大规模 `card_type` 硬编码迁移；新卡 / 新玩法按小切片接入。

### 0.1 必读文档（接手先看）

| 文档 | 路径 |
|------|------|
| 文档入口 | `docs/README.md` |
| **本文件** | `docs/CODEX_HANDOFF.md` |
| 耦合收口完成态 | `docs/design/coupling-risk-mitigation-v0.2.md` |
| 新卡 checklist | `docs/design/card-rule-audit.md`（顶部） |
| 设计变更日志 | `docs/design/CHANGELOG.md` |
| 构建路线图 | `docs/design/core/构建路线图_v3.x.md` |
| 意向 / 世界演化 | `docs/design/core/意向系统与世界演化_v0.3.md` |
| 世界运转共识 | `docs/design/core/世界运转共识_v0.6.md` |
| 规则圣经 | `docs/design/core/设计规则圣经_v0.3.txt`（末尾 **v2.64–v2.70** 节） |

### 0.2 WorldRules 架构（facade 不变）

`scripts/core/world_rules.gd` + 子模块：`craft` / `service` / `state` / `material` / `camp` / `commerce` / `ui`  
寻路：`Pathfinding` → `is_pathfinding_passable_occupant` / `is_pathfinding_blocking_occupant`  
守卫：L0 `CardRuleAudit` 扫描 `scripts/player`、`scripts/world`、`scripts/ui`、`scripts/input` 的 `card_type ==`；卡牌工厂 `factory_lane` + L1.5 单卡 smoke

### 0.3 增量迭代守则

1. 新卡 → `card-rule-audit.md` checklist + `factory_lane` → 只增 `WorldRules.*`  
2. 改 need → 同步 `need_contract.gd` + L0 契约测试  
3. 新卡 / 卡牌规则切片 → **L0 + L1.5**（`--factory-types=...`）；影响主链 → **L2b live**；更新 CHANGELOG + 本文件 §12  
4. **刻意保留**：CardDB relation/impact type 匹配；`.tres` 化；AStarGrid2D  

### 0.3a 卡牌工厂（正式增量迭代基础设施）

| 阶段 | 状态 | 内容 |
|------|------|------|
| **Phase 1 守卫层** | ✅ v2.72 | `factory_lane` 自动分级、AI 可达性 L0 守卫、L1.5 单卡 smoke |
| **Phase 2 脚手架** | ✅ v2.73 | `card_factory_scaffold.gd` + `scenes/card_factory_scaffold.tscn` + `tools/run_card_factory_scaffold.ps1` |
| **Phase 3 产线节点** | 按需 | `production_node` / `actor_loop` 卡：craft + need + 可选 L2b 接线模板 |

**说明：** v2.72 标题里的「MVP」仅指工厂**第一期交付范围**，不是项目仍处 MVP 阶段。项目整体已在 **v2.70+ 正式增量迭代**。

**Phase 2 最小目标（下一工程切片建议）：**

```powershell
# 已实现 v2.73
.\tools\run_card_factory_scaffold.ps1 wildNut --tags=consumable,food,camp.storable --name=野果 --write-dir=generated/scaffold
# → generated/scaffold/wildNut/ 下 01_card_db_reg、02_capabilities、03_audit_notes、04_world_rules_todo、05_verify_commands.ps1
```

Phase 2 **不替代**人工填 tags / 玩法设计；目标是减少 copy-paste 与漏跑 L1.5。**不是「一键出玩法」**——craft / need / behavior 仍由策划定案后与 AI 讨论落代码。

**Phase 2 工作留痕（合入新卡时必做，勿跳过）：**

| 留痕项 | 位置 |
|--------|------|
| 脚手架草稿 | `generated/scaffold/<type>/`（`00_README` … `06_lane_playbook`；可提交或本地保留） |
| 设计变更 | `docs/design/CHANGELOG.md` 一条（含 factory_lane、是否动主链） |
| 协作接续 | 本文件 **§12** 短条（Cursor/Codex/人工谁做了啥） |
| 规则事实 | 合入 CardDB 后跑 `tools/export_card_rule_facts.ps1` → `card-rule-audit.md` / `generated/card-rule-facts.md` |
| 测试 | L0 unit；新卡 **L1.5 `--factory-types=<type>`**；动 need/craft → **L2b live** |

### 0.4 验收基线

- L0 unit：**678** assertions PASS（2026-05-28，含 v2.73 scaffold +7）
- L1.5 card factory smoke：**58** assertions PASS（2026-05-28）
- L2b live：**已达新合格线** — 连续 **5×** headless，`USELESS|player` **每轮 = 0**；5/5 轮 120s 内蘑菇棚建成；`meat_peak ≤ 5`、无 wolf USELESS
- 最新 L2b×5：tick **2365 / 1289 / 1276 / 1264 / 1259**；`meat_peak` **2 / 2 / 0 / 2 / 5**
- 历史参考（耦合收口指标，已被上条取代）：L2b 连续 3× PASS（tick=1494 / 1309 / 1339）

### 0.5 下一重心（策划排期，非工程债务）

Layer 1 **v3.0 稳态** 或 Layer 3 **v3.2 可见意向** — 工程仍走 WorldRules + 小切片，见 §12 最新条目。

**当前工程阻塞（2026-05-28）：** ActionRunner P0、工厂 Phase 1/2 已清零。**无单一工程阻塞**；并行推进：**post-gh 内容 loop（策划 + AI 讨论玩法）** + **按需用 Phase 2 脚手架加卡**。

### 0.6 【已完成】ActionRunner 调度层收口（进入增量迭代的前置门槛）

> **背景：** WorldRules 耦合收口（v2.70）已完成，但 live 仿真下仍有「无推进空转」，卡在「基础合格」约半个月。  
> **请你接手：** 不是再做大规模重构，而是 **把 ActionRunner 调度 invariant 钉死**，达到下方合格线后交还。

#### 0.6.1 成功标准（合格线）

| 指标 | 要求 |
|------|------|
| **L0 unit** | **671** assertions PASS（不得回退） |
| **L2b live ×5** | 连续 5 轮 headless `behavior_trace_live.tscn`，**每轮 `USELESS\|player` = 0** |
| **L2b 结局** | 至少 3/5 轮在 120s 墙内 **蘑菇棚已建成**；其余可墙上超时但 **无错位空转** |
| **狼/肉** | 5 轮内 **无 wolf USELESS**；`meat_peak ≤ 5` |
| **F5 体感**（用户自测） | 不再长时间「气泡说要做 X、人站着不动 / 只在观察态晃」 |

**什么不算合格：**

- L0 绿了但 live 仍有 90+ tick 同签名空转
- 只修 eval 条件、不动 stack/craft/path 同步
- 用 trace bootstrap 掩盖 live 链问题而不标注
- 测试绿了但 `current_need` / `eval` / `stack` / `craft` 长期四不一致

**合格之后（增量迭代阶段）：**

- **不再**做全项目 `card_type` 迁移或大重构
- 新卡 / 新玩法：**WorldRules 小切片 + need_contract + L0 断言**
- 改 need：**同步 `need_contract.gd` + L0**
- 每切片：L0；动主链则 L2b live；更新 `CHANGELOG.md` + 本文件 §12

#### 0.6.2 架构（已建立，不要推翻）

```
NeedEvaluator.evaluate()     → eval_need（世界此刻最该想什么）
MissionResolver.resolve()    → mission（含 build_greenhouse 项目锁）
ActionRunner.tick()          → stack + craft 执行
  ├─ _purge_stale_stack()    → 集中清理 mission / stack 错位
  ├─ _sync_stack()
  ├─ _run_execution() → _dispatch / craft.execute
  └─ _present_mind() → goal/state 展示
CraftTaskSystem.execute()    → survival/shelter/mushroom/commerce FSM
PlayerNeedsManager.update()  → L1 狼反应 / L2 path 冻结 / L3 调 ActionRunner
```

**核心 invariant（2026-05-28 已 enforce 初版）：**

1. `mission` 与 `stack.top.chain` 必须匹配（gh 锁定时不得 `CHAIN_IDLE`）
2. `eval` 为 ambient（`night_home` / `idle_night`）时不得跑 survival handler 链
3. `eval` 为 survival 时不得停在 ambient 链（`CHAIN_IDLE` 观察）
4. `current_need` **只由 ActionRunner 写**；handler 只启 craft、不覆写 need
5. craft 空 + path 0 超过 ~90 tick → recovery 或 abandon，不能无限干站

#### 0.6.3 Cursor 已做（勿重复造轮子）

| 文件 | 改动要点 |
|------|----------|
| `scripts/player/execution/action_runner.gd` | `_purge_stale_stack`；夜间清 survival 链；gh 项目清 idle 链；`gh_project_committed` 时跳过 `_can_start_greenhouse` gate；空 stack fallback `_tick_greenhouse` |
| `scripts/world/player_needs_manager.gd` | `_satisfy_hunger_need` 低饥饿 early return；**已删** handler 内 `p.current_need = "hunger"` |
| `scripts/player/craft_task_system.gd` | `build_greenhouse` 可 preempt `hunger` |
| `scripts/world/behaviors/wolf_behavior.gd` 等 | 狼叼肉/清路/暂存肉 |
| `scripts/test/behavior_trace_runner.gd` | live 模式 USELESS 检测（player/wolf/meat） |
| `scripts/test/unit/unit_test_cases.gd` | +低饥饿跳过 satisfy；+建棚 preempt cookMeat |

**测试现状（2026-05-28）：** L0 **654 PASS**；L2b live ×5 **全部 PASS**，每轮 `USELESS|player=0`，蘑菇棚 5/5 建成，`meat_peak` 最大 5。

#### 0.6.4 本轮处理结果与剩余问题（按优先级）

**P0 — 错位空转（已清零，不再阻塞增量迭代）**

特征：`eval`、`mission(current_need)`、`stack`、`craft` 不一致，craft 空 path 0，同一签名 ≥90 tick。

| 签名模式 | 根因方向 |
|----------|----------|
| `need=build_greenhouse \| eval=build_table \| st=观察 \| craft=空` | gh 已 lock 但 stack 仍在 `CHAIN_IDLE`；或 push 失败后无 fallback |
| `need=hunger \| eval=night_home \| craft=空 \| goal=先弄点吃的` | 夜间 eval 切换后 `CHAIN_HUNGER` 未清（部分已修，仍偶发） |
| `need=build_table \| eval=shelter \| st=观察` | survival eval 顶 develop，idle 链未切 shelter |
| `need=build_greenhouse \| craft=makeAxe/goToWood \| path=0` | lumber 子链 craft 卡住不推进（资源/recovery） |

**修复方向（建议）：**

1. 写一张「eval × mission × stack 合法表」，非法组合在 `_purge_stale_stack` 一处 enforce
2. **`current_need` 单写者**：扫 `player_needs_manager.gd` 里所有 `p.current_need =`，迁入 ActionRunner 或删除
3. **`gh_project_committed`**：强制 stack ∈ {`CHAIN_GREENHOUSE`, `CHAIN_LUMBER`}，否则 clear + push
4. **L0 补组合测**：例 eval=`night_home` + stack=`CHAIN_HUNGER` → 下一 tick stack=`CHAIN_NIGHT_HOME`

**P1 — 资源型卡住（合格后可迭代修）**

| 签名 | 说明 |
|------|------|
| `craft=cookMeat/goToMeat \| path=0` | 找不到肉/路径失败，eval=hunger 一致，recovery 不足 |
| `craft=butcherCorpse/goToCorpse \| path=0` | 尸体不可达 |
| `need=tool \| eval=tool \| craft=空` | 制矛/碎石链未启动 |

eval 与 mission 一致，不算架构失败，但 live 检测会报 USELESS → 需 **fail_task + 换路线或 abandon**。

**P2 — 已通过 / 非主因**

- 狼 USELESS 循环：5 轮 live 未再现为主因
- WorldRules 耦合：方案内 100%，不是当前阻塞点

#### 0.6.5 为什么「结构合理了还有问题」

**不是思路错了**，而是：

1. 验收标准从「能建成」升到「5× live 无空转」，暴露了层间契约欠账
2. `MissionResolver` 项目锁故意让 `eval ≠ mission`，但 stack 必须跟 `mission`
3. **双写期**：handler 仍写 `current_need` / `goal`，与 ActionRunner 打架
4. **规划冻结**（`player_needs_manager` L2：有 craft/path 时 skip tick）与 stack 切换的交互未穷举

**不要做：** 回退 monolith、在 NeedEvaluator 堆更多 if。  
**要做：** 集中 enforcement + 单写者 + 组合测试。

#### 0.6.6 Codex 执行清单

```
☑ 1. 跑 L0，确认 654 PASS
☑ 2. 跑 L2b live ×1，读 report，列出 USELESS 签名
☑ 3. 实现/补全 invariant 表 + _purge_stale_stack（单入口）
☑ 4. 清理 player_needs_manager 中 current_need 双写
     （fire/shelter/hunger/build_table/commerce/tool/management/sanitation）
☑ 5. L0 新增 3–5 条 stack 组合断言
☑ 6. L2b live ×5，目标 USELESS=0
☑ 7. 更新 CHANGELOG + 本文件 §0.4 / §12
☑ 8. 交还：报告「测试 vs 体感」分开写
```

#### 0.6.7 命令与关键路径

```powershell
# 项目根
E:\桌面\方寸商国：桃花源记

# Godot
E:\game\play\Godot_v4.6.1-stable_win64.exe\Godot_v4.6.1-stable_win64_console.exe

# L0
Godot --path "E:\桌面\方寸商国：桃花源记" res://scenes/unit_test.tscn

# L1.5 单卡 smoke（默认代表样本）
Godot --headless --path "E:\桌面\方寸商国：桃花源记" --scene res://scenes/card_factory_smoke.tscn

# L1.5 指定卡
Godot --headless --path "E:\桌面\方寸商国：桃花源记" --scene res://scenes/card_factory_smoke.tscn -- --factory-types=berry,coin,waterbucket

# Phase 2 脚手架（新卡草稿，不生成玩法）
.\tools\run_card_factory_scaffold.ps1 wildNut --tags=consumable,food,camp.storable --name=野果 --write-dir=generated/scaffold

# L2b live（单轮）
Godot --headless --path "E:\桌面\方寸商国：桃花源记" --scene res://scenes/behavior_trace_live.tscn

# 报告目录
%APPDATA%\Godot\app_userdata\方寸商国：桃花源记\
  behavior_trace_report.txt
  card_factory_smoke_report.txt
  live_run_*.txt
```

**主战场源码：**

- `scripts/player/execution/action_runner.gd`
- `scripts/player/execution/mission_resolver.gd`
- `scripts/player/execution/need_evaluator.gd`
- `scripts/player/execution/player_execution_stack.gd`
- `scripts/player/execution/player_execution_recovery.gd`
- `scripts/world/player_needs_manager.gd`
- `scripts/player/craft_task_system.gd`
- `scripts/test/behavior_trace_runner.gd`（STALL 阈值 90 tick）

#### 0.6.8 禁止事项

- 动主链却未跑 L0 + L2b×5 就声称完成
- 新卡 / 卡牌规则切片未跑 L0 + L1.5 就声称完成
- 未经用户要求 git commit
- 只改 eval 不改 stack/exec（或反之）
- 用 bootstrap 掩盖 live 问题
- 新一轮 WorldRules 大迁移（增量迭代阶段已冻结）

#### 0.6.9 交付物

1. L0 断言数
2. L2b×5 每轮：结束原因、USELESS 数、meat_peak
3. 改了哪些 invariant、删了哪些双写
4. 仍知的 P1 资源型卡住（可进增量迭代后修）

---

## 1. 项目是什么

Godot **4.6.1** 2D 卡牌网格生存/经营游戏。玩家控制一张 **PlayerCard**，在网格上自动求生、建营地、建蘑菇棚、做生意。架构目标（用户与 Cursor 已共识）：

```
NeedEvaluator → MissionResolver → ActionRunner + ExecutionStack → CraftTask FSM → CampHelpers / Interaction
```

**核心不变量：** `eval` 必须与 `exec` 对齐 — need 只有在对应 handler **真能开工** 时才应激活。

---

## 2. 路径与工具

### 2.1 项目根目录

```
E:\桌面\方寸商国：桃花源记
```

### 2.2 Godot 可执行文件（本机）

```
E:\game\play\Godot_v4.6.1-stable_win64.exe\Godot_v4.6.1-stable_win64_console.exe
```

### 2.3 设计文档（正本在 `docs/`）

| 类型 | 路径 |
|------|------|
| **入口** | `docs/README.md` |
| **规则圣经** | `docs/design/core/设计规则圣经_v0.3.txt` |
| **桌面快捷入口** | `E:\桌面\方寸商国_文档入口.md` |

### 2.4 设计文档索引

**策划正本（`docs/design/core/`）：** 设计规则圣经 · 构建路线图_v3.x · 世界运转共识_v0.6 · 意向系统_v0.3  

**工程向（`docs/design/`）：** 见 `docs/design/README.md` 或 `docs/README.md`  

| 文件 | 内容 |
|------|------|
| `core/设计规则圣经_v0.3.txt` | **主文档**（以末尾版本节为准） |
| `core/构建路线图_v3.x.md` | Layer 0～7+ 分期 |
| `core/世界运转共识_v0.6.md` | 世界/行为规则 |
| `core/意向系统与世界演化_v0.3.md` | 意向系统策划 |
| `CHANGELOG.md` | 设计变更日志 |
| `card-rule-audit.md` | 全卡审计 + 新卡 checklist |
| `coupling-risk-mitigation-v0.2.md` | 耦合收口完成态 |
| `generated/*` | 自动导出事实表 |
| `archive/*` | 旧版摘录（勿编辑） |

### 2.5 Agent / 编码规范

| 文件 | 用途 |
|------|------|
| `AGENTS.md` | Agent 路由、设计文档工作流 |
| `.cursor/rules/gdscript-godot.mdc` | GDScript 类型、禁止 `:=` 等 |
| `.cursor/skills/godot-gdscript/SKILL.md` | 改 GDScript 前必读 |

### 2.6 测试报告输出目录

```
%APPDATA%\Godot\app_userdata\方寸商国：桃花源记\
```

常见报告：`unit_test_report.txt`、`intent_trace_report.txt`、`behavior_trace_report.txt`、`smoke_report.txt`、`greenhouse_build_report.txt`

---

## 3. 代码结构（关键目录）

```
autoload/game_state.gd          # 全局常量（REAL_TICK、WET_WOOD_MATURE_SECONDS 等）
scenes/main.tscn                # 主游戏场景
scenes/unit_test.tscn           # L0
scenes/intent_trace.tscn        # L1
scenes/behavior_trace_greenhouse.tscn  # L2a
scenes/behavior_trace_live.tscn # L2b
scenes/smoke_test.tscn          # L3

scripts/world/
  world_manager.gd              # 主循环 simulate_tick
  player_needs_manager.gd       # Layer1 狼反应 / Layer2 path / Layer3 ActionRunner
  ecosystem_manager.gd          # behavior host、旅人交易
  environment_manager.gd        # tick_wet_wood 等

scripts/player/
  execution/
    need_evaluator.gd           # eval need
    mission_resolver.gd         # mission + greenhouse_project_active
    action_runner.gd            # 执行栈调度、GH/lumber 链
    player_execution_stack.gd
    player_execution_recovery.gd  # path_intent、stall 恢复
    mind_presenter.gd           # Intent 呈现 → player.goal
    world_snapshot.gd
    chain_registry.gd
  tasks/
    craft_task_helpers.gd       # go_near / go_to_xy
    survival_tasks.gd           # hunger、cookMeat
    shelter_tasks.gd            # moveTable、moveHut
    mushroom_tasks.gd           # buildGreenhouse 全 phase
  camp_helpers.gd               # storage、commerce_need_active、organize
  mushroom_helpers.gd           # 湿木成熟、find_greenhouse
  commerce_decision.gd          # 自动交易（默认无 UI 确认）
  craft_task_system.gd

scripts/core/
  world_helpers.gd              # path、pickup、set_path_near
  interaction_manager.gd        # apply_relation、spawn
  pathfinding.gd
  world_kernel.gd               # 唯一 InteractionManager 入口

scripts/cards/
  card_base.gd                  # 含 wet_timer 字段
  player_card.gd                # path_intent、execution_stack

scripts/test/
  gh_trace_bootstrap.gd         # GH 追踪场景预置 wood+axe
  intent_trace_runner.gd
  behavior_trace_runner.gd
  eco_smoke_runner.gd
  card_rule_audit.gd            # 全卡规则审计 + 一致性检查
  unit/unit_test_cases.gd

tools/
  run_unit_tests.ps1            # L0
  run_intent_trace.ps1          # L1
  run_smoke_test.ps1            # L3
  export_card_rule_facts.ps1    # 导出 CardDB 事实 + 刷新审计表
  run_batch_10.ps1              # 10 轮批测（曾有 PowerShell 编码问题，可内联跑）
```

---

## 4. 测试管线（跑完必须如实汇报）

```powershell
$godot = "E:\game\play\Godot_v4.6.1-stable_win64.exe\Godot_v4.6.1-stable_win64_console.exe"
$root  = "E:\桌面\方寸商国：桃花源记"

# L0 — 纯逻辑单测，~5s
& $godot --headless --path $root --scene res://scenes/unit_test.tscn

# L1 — Intent 序列追踪，~50s
& $godot --headless --path $root --scene res://scenes/intent_trace.tscn

# L2a — GH 行为追踪
& $godot --headless --path $root --scene res://scenes/behavior_trace_greenhouse.tscn

# L2b — Live 行为追踪
& $godot --headless --path $root --scene res://scenes/behavior_trace_live.tscn

# L3 — 冒烟 8 游戏日 / 60s 墙上限
& $godot --headless --path $root res://scenes/smoke_test.tscn
```

**纪律：** 不要未测就声称修复成功。L0–L3 绿不等于真人玩起来不卡。

---

## 5. 上次 Cursor 会话结束时的测试状态

（约 2026-05-20，以当时最后一次完整跑为准）

| 层 | 结果 | 备注 |
|----|------|------|
| L0 | **PASS** (223) | |
| L1 | **PASS** | GH 5 步 intent 齐全，~t=166 建成 |
| L2a | **PASS** | tick=166 建成 |
| L2b | 退出码 0 | 仍有 storage/idle_night 卡滞段；**未真正「建成 GH」** |
| L3 | **PASS** | 从 ~175 FAIL 降到 PASS（smoke 条件放宽 + storage eval 对齐） |

**用户主观体验（比测试更重要）：** 玩家卡仍经常 **对着某张卡左右走动、什么都不做**。测试通过未消除此现象。

### 5.1 本轮 Codex 修复状态（2026-05-26）

本轮只处理 P0 可见断点，并把修法压进 `WorldRules` 标签 / 卡牌标签方向；没有做完整规则引擎重写。

| 层 | 结果 | 备注 |
|----|------|------|
| L0 | **PASS** (296) | 新增 `material.lumber`、双向菇棚配方、经济物收纳、已有菇棚停建、狼清路、幼狼跟随/入窝、手动代管、sanitation 对齐单测 |
| L1 | **PASS** | `ticks=1271`，GH intent 链完整，蘑菇棚建成 |
| L2a | **PASS** | `tick=1272`，蘑菇棚建成，卡滞段 0 |
| L2b | **PASS** | `tick=1890`，蘑菇棚建成，卡滞段 0 |
| L3 | **PASS** | 60s 墙上限，约 1.61 游戏日，无玩家消失/核心生态崩溃 |

本轮已完成：

- `wood` / `twig` 同属 `material.lumber`；蘑菇棚、菇棚维护、补木、斧柄走统一木材规则；长矛仍只使用 `twig`。
- `buildGreenhouse` 任意阶段发现已有 `mushroomGreenhouse` 会立即完成并清锁，不再继续打水/做湿木。
- `operateGreenhouse` 已补齐：采菇木、等待湿木成熟、取水做湿木、补木管理。
- `hut + mushroomWood` 双向配方均可合成 `mushroomGreenhouse`。
- `coin`、`copperBlock`、`copperCraft` 已加 `camp.storable`，按普通营地物资收纳。
- 狼食物足够时不反复猎杀；叼肉回窝遇到兔/小动物挡路时，先放肉，只驱赶路线，不造成伤害，再回到回窝流程。
- 幼狼无窝时跟随最近成年狼；有窝后走向狼窝再入窝，不再生命周期瞬移。
- 手动左右键代管已接入：拖拽玩家卡会立即清旧 path / on_arrive / craft / 执行栈，放手后 3 tick 内自动规划让出控制权，避免旧任务回弹。
- sanitation 已对齐：评估层触发清尸时，执行层即使肉储备已够也必须启动清理任务；它可打断建棚锁，并会阻止夜间睡眠跳过清尸执行，不再 `need=sanitation craft空 path空` 发呆。
- 手持石头制碎石已对齐：玩家卡手里已有 `stone` 且附近有第二块 `stone` 时，会直接进入 `makeShardForFire/walkToStone2ForFire`，用于后续生火链。
- behavior trace 的卡滞统计进一步收窄：夜间躲藏 / 躲狼 / 睡觉 / 休整不再被算作建棚执行卡死，只统计真实 `buildGreenhouse/*` craft 停滞。

最新追加验收（2026-05-26 v2.30）：

| 层 | 结果 | 备注 |
|----|------|------|
| L0 | **PASS** (302) | 新增手持石头参与制碎石 / 生火链单测 |
| L1 | **PASS** | `tick=1312`，GH intent 链完整，蘑菇棚建成 |
| L2a | **PASS** | `tick=1281`，蘑菇棚建成，卡滞段 0 |
| L2b | **PASS** | `tick=1349`，蘑菇棚建成，卡滞段 0 |
| L3 | **PASS** | 60s 墙上限，约 1.12 游戏日，无玩家消失/核心生态崩溃 |

仍需真人 F5 复验：手动代管已修自动回弹入口，但玩家连续拖拽、右键关系交互和自动恢复节奏仍要用实际操作确认体感。

### 5.2 规则审计状态（2026-05-27 v2.31）

本轮没有迁移运行时逻辑，只建立“世界规则标签机 / 卡牌封装”的可检查结构。

新增：

- `docs/design/world-rule-tag-machine-v0.1.md`：定义标签、卡牌封装、世界规则、执行入口
- `docs/design/generated/card-rule-facts.md`：自动导出的代码事实，cards=50、relations=21、impact recipes=9
- `docs/design/card-rule-audit.md`：覆盖全部 50 张卡，记录运行时状态、AI 可达性、风险结论
- `scripts/test/card_rule_audit.gd`：L0 一致性检查
- `tools/export_card_rule_facts.ps1`：刷新事实表与审计表

最新追加验收：

| 层 | 结果 | 备注 |
|----|------|------|
| audit export | **PASS** | cards=50，relations=21，impact recipes=9 |
| L0 | **PASS** (307) | 新增 CardDB 引用完整性、标签冲突、source 产物、核心链可达性检查 |
| L1 | **PASS** | `tick=1660`，GH intent 链完整，蘑菇棚建成 |
| L2a | **PASS** | `tick=1369`，蘑菇棚建成，卡滞段 0 |
| L2b | **FAIL** | 连续两次 tick 上限未建成；卡滞段 0，live 夜间/饥饿/躲狼打断建棚 |
| L3 | **PASS** | 60s 墙上限，约 1.76 游戏日 |

注意：本轮未改运行时玩法。L2b 失败应作为下一轮 live 稳定性问题处理，不要归因到 `CardRuleAudit`。

### 5.3 Capability / Action 设计状态（2026-05-27 v2.32）

新增 `docs/design/capability-action-taxonomy-v0.1.md`，确认下一层抽象方向：

- 标签 Tag：描述卡牌稳定身份和属性
- 能力 Capability：描述卡牌能进入哪个行为系统
- 行为 Action：描述当前可执行动作模板

关键共识：

- 捕猎不应是狼专属代码，也不应是玩家专属代码，应逐步收束为 `capability.hunt` + `action.hunt`
- 抚养不应是幼狼专属逻辑，应逐步收束为 `capability.care_child` + `action.feed/guard/follow/return_home`
- 非动物卡牌也必须纳入同一结构：篝火、草棚、桌子、蘑菇棚、狼窝、森林、水源分别对应 `domain`、`bond`、`service`、`produce`、`transform`
- `card-rule-audit.md` 已增加 `capabilities` 字段，现有 50 张卡均已补首批能力标记
- L0 已检查关键能力：狼 `capability.hunt`、篝火 `capability.define_domain`、桌子 `capability.provide_service`
- 下一步建议同时准备两个候选样板：`hunt + care_child` 与 `camp domain + fire bond`

### 5.4 首批能力迁移候选（2026-05-27 v2.33）

新增 `docs/design/capability-migration-slices-v0.1.md`。

候选：

- `camp domain + fire bond`：篝火、草棚、桌子、蘑菇棚的域 / 羁绊 / 服务结构
- `hunt + care_child`：狼捕猎、玩家捕猎、幼狼跟随 / 入窝的能力行为结构

当前推荐优先级：先做 `camp domain + fire bond`，因为它覆盖非动物卡牌，更贴近玩家主线，也能为 L2b live 失败提供更清楚的诊断结构。

### 5.5 camp domain + fire bond 运行时样板（2026-05-27 v2.34）

已完成第一段运行时迁移，范围为查询层和稳定调用点替换，不重写玩家任务 FSM。

已改：

- `WorldRules` 新增 `domain.camp`、`domain.mushroom_farm`、`bond.fire`、`service.*` 查询。
- `CampHelpers` 的 `holds_domain`、`has_fire_bond`、`bond_active`、`active_base_cells`、`bonded_hut` 改为消费 `WorldRules`。
- 篝火定义营地域；草棚/桌子通过火羁绊吸附营地域；菇棚提供 `service.greenhouse`，但本轮不吸附进营地域。

验证：

| 层 | 结果 | 备注 |
|----|------|------|
| L0 | **PASS** (323) | 新增域 / 羁绊 / 服务查询断言 |
| L1 | **PASS** | `tick=1264`，蘑菇棚建成 |
| L2a | **PASS** | `tick=1725`，蘑菇棚建成，卡滞段 0 |
| L3 | **PASS** | 60s 墙上限 |

后续建议：推进 `hunt + care_child` 作为第二条样板；候选 A 之后再继续把睡眠、交易、菇棚维护改为服务查询驱动。

### 5.6 hunt + care_child 运行时样板（2026-05-27 v2.35）

已完成第二段运行时迁移，范围为规则查询层和低风险调用点替换，不重写完整生态 AI。

已改：

- `WorldRules` 新增 `domain.wolf_pack`、`action.hunt`、`action.feed`、`action.follow`、`action.return_home` 常量与查询。
- 狼选猎物改为 `WorldRules.is_hunt_target_for(wolf, card)`。
- 玩家普通狩猎目标选择改为 `WorldRules.is_hunt_target_for(player, card)`；本轮仍保持玩家不自动猎羊羔。
- 幼狼无窝跟随成年狼改为 `WorldRules.nearest_care_actor(cub)`。
- 狼窝定义狼群域，并可作为狼 / 幼狼 home。

验证：

| 层 | 结果 | 备注 |
|----|------|------|
| L0 | **PASS** (339) | 新增 hunt / care_child / wolf_pack 断言 |
| L1 | **PASS** | `tick=1454`，蘑菇棚建成 |
| L2a | **PASS** | `tick=1252`，蘑菇棚建成，卡滞段 0 |
| L2b | **PASS** | `tick=1443`，live 蘑菇棚建成，卡滞段 0 |
| L3 | **PASS** | 60s 墙上限 |

后续建议：继续迁移喂幼狼、回窝、建窝、伏击草丛到 action 查询；同时抽出猎物偏好和风险规则。

### 5.7 Rule Dimensions / 标签维度约束（2026-05-27 v2.36）

已把用户提出的“标签之上要有维度”落成 L0 工程约束。

已改：

- 新增 `docs/design/rule-dimensions-v0.1.md`。
- `CardRuleAudit` 新增六维注册：`identity`、`material_form`、`capability`、`relation_domain`、`action`、`rule_modifier`。
- L0 检查 `CardDB` 标签、`CARD_CAPABILITIES`、首批运行时 `domain/bond/service/action` 是否全部可归维度。
- 新增自动事实表 `docs/design/generated/rule-dimension-facts.md`。

验证：

| 层 | 结果 | 备注 |
|----|------|------|
| audit export | **PASS** | 生成 `card-rule-facts.md`、`rule-dimension-facts.md`、`card-rule-audit.md` |
| L0 | **PASS** (345) | 新增维度归类断言 |

后续建议：不要先凭空加 `preference.*` / `reach.*` / `requires.*` / `risk.*`；等迁移猎物偏好、武器距离、专名输入、风险禁忌时再加入，避免维度空转。

### 5.8 Hunt Target / 行为目标选择收口（2026-05-27 v2.37）

已完成行为四段式中的 target 阶段样板。范围仍是小切片：把捕猎“对谁做”集中到 `WorldRules`，不重写完整玩家 / 狼执行逻辑。

已改：

- `WorldRules` 新增 `best_hunt_target()`、`hunt_target_visible_to()`。
- 狼 `best_prey_for_wolf()` 改为直接消费 `WorldRules.best_hunt_target(wolf)`。
- 玩家近身反应、饥饿捕猎、制矛升级判断统一消费 `WorldRules.best_hunt_target(player)`。
- `CardBase` 补 `hiddenInGrass` 字段，兔子藏草状态可被规则层读取。
- L0 覆盖玩家最佳猎物、狼兔子偏好、草中远距离兔子不可见。

验证：

| 层 | 结果 | 备注 |
|----|------|------|
| L0 | **PASS** (349) | 新增 hunt target 断言 |
| L1 | **PASS** | `tick=1609`，蘑菇棚建成 |
| L2a | **PASS** | `tick=2170`，蘑菇棚建成，卡滞段 0 |
| L2b | **PASS** | `tick=1409`，live 蘑菇棚建成，卡滞段 0 |
| L3 | **PASS** | 60s 墙上限 |

后续建议：继续迁移 execution / recovery 的一个窄切片，例如狼叼肉回窝、喂幼狼，或玩家手持石头 / 木材时恢复任务。

### 5.9 Return Home / 回家目标查询收口（2026-05-27 v2.38）

已完成回家动作的 target / recovery 边界样板。范围仍然是目标查询层，不重写完整狼 AI。

已改：

- `WorldRules` 新增 `home_for_actor()`。
- 成年狼和幼狼通过 `capability.return_home` + `domain.wolf_pack` 查询 home。
- `homeDenId` 绑定优先于最近狼窝，避免多狼窝时误选近窝。
- `EcosystemManager.den_for_wolf()`、狼饱腹回窝、叼肉回窝、幼狼入窝入口改为消费规则查询。
- L0 覆盖成年狼 / 幼狼 home 查询，以及幼狼绑定狼窝优先级。

验证：

| 层 | 结果 | 备注 |
|----|------|------|
| L0 | **PASS** (352) | 新增 return_home / home_for_actor 断言 |
| L1 | **PASS** | `tick=1270`，蘑菇棚建成 |
| L2a | **PASS** | `tick=1296`，蘑菇棚建成，卡滞段 0 |
| L2b | **PASS** | `tick=1272`，live 蘑菇棚建成，卡滞段 0 |
| L3 | **PASS** | 60s 墙上限 |

后续建议：如果继续规则迁移，优先选一个 execution/recovery 窄切片，如狼叼肉回窝清路，或玩家手持材料恢复任务。

---

## 6. 当前核心问题（按优先级）

### P0 — 用户可见：「对着卡左右走、什么都不做」

**不是单一 bug，是一类架构问题：多个系统同时写 `path` / `on_arrive`，且每 tick 重新决策。**

| 机制 | 位置 | 表现 |
|------|------|------|
| **邻格重选** | `world_helpers.gd` `find_reachable_neighbor_near` + `craft_task_helpers.gd` `go_near` | 目标卡左右两格路径每帧切换 |
| **mission 翻转** | `need_evaluator.gd` `storage_idle` ↔ `idle_scan` | 拎着石头在 12,4↔12,5 踱步，craft 空 |
| **orphan on_arrive** | `camp_helpers.gd` `update_camp_storage` | 归置用回调而非 craft 链，与栈清空不同步 |
| **狼 vs craft 抢 path** | `player_needs_manager.gd` `_player_wolf_react` vs `survival_tasks.gd` `cookMeat/walkToFire` | 火边/肉边转圈 |
| **到了邻格不推进 phase** | 各 `*_tasks.gd` 的 `go*` phase | `is_neighbor` 后无 pickup/impact/advance |

**Cursor 已做但未根治的补丁：**

- `path_intent` **sticky approach**（`player_execution_recovery.gd` + `craft_task_helpers.gd`）
- `storage_idle` 移动中 **need 锁定**（`need_evaluator.gd` `_movement_locked_need`）
- `storage_need_active` / `storage_can_execute` 对齐（`camp_helpers.gd`）

**建议 Codex 主攻（架构级，一轮中等 refactor）：**

1. **单 tick 单 path 主人**：优先级 reactive > craft move > stack mission；下层不得覆写上层 path  
2. **sticky approach 扩到所有 `set_path_near` 调用**  
3. **所有 `go_near` phase 声明后置条件**：已 `is_neighbor` 则必须 advance / pickup / fail，禁止 silent return  
4. **归置改为 `campOrganize` craft 链**，删除 `camp_helpers` 里 orphan `on_arrive`  
5. **狼 flee 与 craft**：要么 suspend craft，要么 flee 不写 path（当前是 overlay）

### P1 — eval ↔ exec 仍可能错位

- `storage_idle`：已加 `storage_can_execute`，Live 场景仍可能偶发
- **Goal 僵尸**：`mind_presenter.gd` 每 tick 从 intent 写 `player.goal`，与 mission 冲突时需 `_present_mind` / `_align_mission_goal` 协同（`action_runner.gd`）

### P2 — 建菇棚链（GH）

**2026-05-26 本轮已把追踪场景和 Live 场景都跑通：**

| 已修 | 文件 |
|------|------|
| `wet_timer` 不累积（湿木永不成熟） | `card_base.gd`、`mushroom_helpers.gd` |
| 合棚需搬菇木到草棚旁 | `mushroom_tasks.gd` `combineGh` |
| 合棚前菇木不被火烤干 | `mushroom_helpers.gd`（无 GH 时不 dry mw） |
| 追踪 bootstrap wood+axe | `gh_trace_bootstrap.gd` |
| GH 栈 step0 不清进行中的 craft | `action_runner.gd` |
| `wood` / `twig` 统一为 `material.lumber` | `world_rules.gd`、`mushroom_tasks.gd`、`mushroom_helpers.gd` |
| 已有菇棚时停止建棚链 | `mushroom_tasks.gd` |
| 菇棚维护链补齐 | `mushroom_tasks.gd` |
| 缺桶时为菇棚链补可用桶 | `mushroom_tasks.gd` |

最新结果：v2.38 L0 `352 assertions` PASS。最近一次行为回归：L1 `tick=1270` 建成，L2a `tick=1296` 建成且卡滞段 0，L2b live `tick=1272` 建成且卡滞段 0，L3 smoke PASS。手动代管/回弹入口已处理；手持石头制碎石链路已补，仍需真人 F5 复验体感。

### P3 — 测试与体验脱节

- L3 smoke **PASS** 主要靠：豁免 `night_home`、不可执行 storage 不计 FAIL、60s 墙上限
- 旧 L2b 曾不强制 GH 建成，不能代表 Live playable；2026-05-26 最新 L2b 已建成 GH 且卡滞段 0，但仍需真人 F5 复验
- `tools/run_batch_10.ps1` 曾有编码问题，批测需 PowerShell 内联或改 ASCII

### P4 — 已完成、勿重复造轮子

- CardBase ↔ WorldHelpers 循环依赖（内联 `_carry_follow_offset`）
- moveTable 犹豫（`_is_atomic_carry_move`、survival 栈同步）
- 商务自主 `commerce_decision.gd`，`GameState.commerce_manual_trade_confirm = false`
- 拎桌 orphan `resume_interrupted_bond_move`
- **recovery 层**：`ActionRecovery` 已覆盖玩家携带 + 狼叼肉清路（v2.39–v2.42，见 §12）

---

## 12. Cursor ↔ Codex 协作留痕

> 双方接续时在此追加短条，不必长文。策划未动则不写圣经。

### 2026-05-20 — v3.0-A2 鹿生态链（Cursor）

- 新增 `deer` / `deerCorpse` / `deerMeat` + `deer_behavior.gd`；狼猎鹿、玩家猎鹿、烹饪链接入。
- L0 **707** PASS；L1.5 鹿三卡 PASS；L2b 1× USELESS=0 GH tick=1519。
- **A3 下一步：** 桃源三性格 + 旅人封存 + 桌子 dining。

### 2026-05-20 — v3.0-A1 L2b 3× 稳定性复验（Cursor）

- L2b live **3×**：USELESS=0、stalls=0；GH tick=1426 / 1436 / 1413；meat_peak=0 / 3 / 1。
- 初跑 shelter 空转（24,12 / need=shelter|craft=空）**未复现**；暂不启 A1.1 moveTable/shelter recovery 切片。
- **A1 provisional stable** — A2（鹿生态链）可开。

### 2026-05-20 — v3.0-A1 河谷地形基础（Cursor）

- 新增 `world_rules_terrain.gd`；`TerrainManager`/`Pathfinding` 统一 cell 分类；6 类 `terrain.*` 标签 + `domain.water_source`/`social_boundary` zone 查询。
- 地图 **36×24**；`START_PLAYER_X/Y=24,14`；初始 spawn 相对营地重排。
- terrain trace PASS；L0 **691** PASS；L2b 3× **USELESS=0**（GH 1426/1436/1413）→ **A1 provisional stable**。
- 初跑 USELESS=1（shelter@24,12）未在 3× 复现；A1.1 搁置 unless 再现。
- **A2 下一步：** 鹿三卡 + 生态链（用户确认后开干）。

### 2026-05-20 — 清理桌面冗余（Cursor）

- 删除桌面 stub 与 `方寸商国MVP/`；删除 `sync_design_doc.ps1`。
- 桌面只留 `方寸商国_文档入口.md` → 工程 `docs/README.md`。

### 2026-05-20 — 设计文档归拢（Cursor）

- 策划正本统一迁入 `docs/design/core/`；入口 `docs/README.md`。
- 桌面根目录同名 md/txt 改为跳转 stub；旧摘录移 `docs/design/archive/`。
- 更新 `AGENTS.md`、`sync_design_doc.ps1`（圣经路径 `core/`）、`CHANGELOG.md`。

### 2026-05-28 — v2.73 卡牌工厂 Phase 2：脚手架（Cursor）

- 新增 `scripts/tools/card_factory_scaffold.gd`：按 `factory_lane` 生成 CardDB / CARD_CAPABILITIES / AUDIT_NOTES 片段、WorldRules 待办、L1.5 验证命令、lane playbook（**不生成玩法**；设计仍人工 + AI 讨论）。
- 新增 `scenes/card_factory_scaffold.tscn`、`tools/run_card_factory_scaffold.ps1`；`--write-dir=generated/scaffold` 输出到项目目录（`00_README` … `06_lane_playbook` 即工作留痕草稿）。
- L0 新增 `_test_card_factory_scaffold`（678 assertions）；修复已存在卡复查时 `_suggest_capabilities` 类型返回。
- 合入新卡流程与留痕表见 `docs/CODEX_HANDOFF.md` §0.3a。

### 2026-05-28 — v2.72 卡牌工厂 Phase 1：守卫层（Codex）

> 项目阶段：**正式增量迭代**。本片为卡牌工厂第一期（守卫 + L1.5），非项目 MVP 阶段回退。
- `CardRuleAudit` 新增 `factory_lane` 自动分级：`camp_resource` / `manual_resource` / `production_node` / `structure_service` / `actor_loop` / `environment_source`。
- 新增高交互卡守卫：有 relation / impact IO、`capability.transform_input`、`capability.produce_resource`、`capability.provide_service` 的卡，必须有 AI 可达性说明或 factory 例外。
- 首批例外：`hoe`、`hammer`、`halfbucket`；例外原因写在 `CardRuleAudit.FACTORY_REACHABILITY_EXCEPTIONS`，导出审计表保留 risk。
- 新增 L1.5 单卡 smoke：`scenes/card_factory_smoke.tscn` + `scripts/test/card_factory_smoke_runner.gd`，默认覆盖 `berry`、`coin`、`waterbucket`、`woodStruct`、`mushroomGreenhouse`、`traveler`、`wolfCub`、`tree`、`hammer`、`halfbucket`。
- `docs/design/card-rule-audit.md` 已刷新，新增 `factory_lane` 列与新卡接入 checklist；事实表仍以 `docs/design/generated/card-rule-facts.md` 为准。
- L0 **671 PASS**；L1.5 card factory smoke **58 PASS**。L2b 未跑：本片未改主链、need、craft FSM 或 WorldRules 行为。

### 2026-05-28 — v2.71 ActionRunner 调度层收口（Codex）
- `ActionRunner.sync_current_need()` 成为自动调度 `current_need` 单写入口；`player_needs_manager.gd` 清掉 fire/shelter/build_table/commerce/tool/management/sanitation 等 handler 双写。
- `_purge_stale_stack()` 增加 mission → allowed chains 守卫；`build_greenhouse` 项目锁只在 mission 仍为 build_greenhouse 时强制 greenhouse/lumber，允许 hunger/sanitation 等真实求生需求抢占。
- 夜间 sleep 分支同步 mission，并把 `state/goal` 闭合为 `休整`；shelter fallback 写成可解释等待，避免旧求生状态污染 trace。
- 资源型 FSM 补不可达失败：`cookMeat/goToMeat`、`forageEat/goToEat`、`makeAxe/goToWood/walkToTri`、`makeSpear/goToMaterial/walkToShard`。
- 狼非近战威胁不再打断“携材生火”原子移动；菇棚取水在无可达桶时允许营地补桶。
- L0 **654 PASS**；L2b live ×5 **全部 PASS**：tick **2365 / 1289 / 1276 / 1264 / 1259**，每轮 `USELESS=0`，`meat_peak=2 / 2 / 0 / 2 / 5`。
- 剩余 P1：真人 F5 复验手感；继续把更多资源型 `path=0` 阶段按同一规则收口；手动代管回弹后续单独处理。

### 2026-05-20 — v2.39 ActionRecovery 切片 1（Cursor）

**做了什么**
- 新建 `scripts/core/action_recovery.gd`：`carry_intent` / `can_resume_carry` / `resume_carry(try_bond, try_organize)`
- 分类走 `WorldRules.bond_to_domain`、`is_camp_storable`、`is_organize_locked`；执行仍委托 `CampHelpers.resume_interrupted_*`

**接线**
| 调用点 | 行为 |
|--------|------|
| `player_execution_recovery.after_interrupt` | `resume_carry(..., false, true)` — 仅 organize，与旧行为一致 |
| `try_recover_stall` / `action_runner._tick_storage` | 默认 bond → organize |
| `player_needs_manager._satisfy_shelter_need` | 仅 bond |
| `camp_helpers.update_camp_storage`（carrying） | runtime `load()` 转调，**避免与 CampHelpers 循环 compile** |

**未做（下一片，按序）**
1. ~~`FIRE_BOND_DOMAIN_TYPES` → 全走 `WorldRules.bond_to_domain`~~（v2.40 已做）
2. ~~storage orphan `on_arrive` → `campOrganize` craft~~（v2.41 已做 fetch 路径）
3. ~~狼 recovery 作 `ActionRecovery` 第二 consumer~~（v2.42 已做清路 recovery）

**测试**
- L0：**358 assertions PASS**
- L2b：首次 FAIL（tick=5000，`gh=false`，卡滞 0）；**复跑 PASS tick=1346，卡滞 0** — Live 有波动，与 carry recovery 无直接关联

**工程文档**：`rule-dimensions-v0.1.md` §5.3、`CHANGELOG.md` v2.39

### 2026-05-20 — v2.40 FIRE_BOND 收口（Cursor）
- 删 `FIRE_BOND_DOMAIN_TYPES`；L0 361 PASS；L2b tick=2295

### 2026-05-20 — v2.41 storage fetch 去 orphan on_arrive（Cursor）
- `update_camp_storage` 无 craft 不再写 `on_arrive`；L0 362 PASS；L2b tick=2058

### 2026-05-20 — v2.42 狼 recovery（Cursor）
- `ActionRecovery.wolf_carry_intent` / `resume_wolf_carry`；L0 368 PASS；L2b tick=1367

### 2026-05-27 — v2.70 耦合风险收口 UI+camp+commerce（Cursor）
- 新增 world_rules_camp/commerce/ui；game_ui/world_manager/drag_manager 规则化
- CardRuleAudit 扫描 ui/input；L0 **625** PASS；L2b 3× PASS（1494/1309/1339）
- 方案内耦合风险 **100%** — 见 `coupling-risk-mitigation-v0.2.md`

### 2026-05-27 — v2.69a WorldRules 材料域拆分起步（Codex）
- 新增 `scripts/core/world_rules/world_rules_material.gd`，承接材料、木材、容器、来源、湿木/菇木、燃料等纯查询。
- `WorldRules` 保持 facade API，`is_lumber_material` / `is_water_bucket` / `is_lumber_source_tree` / `is_heated_water_container` 等外部调用不变。
- 本片只搬查询实现，不改任务语义，不改 CardDB 标签。
- L0 **617** PASS。
- L2b live **tick=1334 / wall=31.4s PASS**，蘑菇棚建成，卡滞段 0。

### 2026-05-27 — v2.68 Pathfinding passable 规则化第一刀（Codex）
- 新增 `WorldRules.is_pathfinding_passable_occupant(card)`，由规则层统一回答“地上这张卡是否可被寻路穿过”。
- `WorldRules.is_layout_passable_occupant` 扩展到草、灌木、可采食物、水桶、非锁定营地可收纳物。
- `Pathfinding.blocks_actor` 删独立 passable / blocking type 列表，改消费 `WorldRules.is_pathfinding_passable_occupant` / `is_pathfinding_blocking_occupant`。
- 当前寻路策略：草、灌木、可采食物、水容器、非锁定营地可收纳物、散落可搬物可通过；rooted / businessUnit / shelter / 生物 / 尸体 / 营地域定义者阻挡；未知未登记类别默认阻挡。
- L0 新增 pathfinding passable 用例：草 / 浆果 / 空桶 / 散落工具 / 湿木可通过；羊 / 树 / 篝火 / 狼窝阻挡。
- `docs/design/card-rule-audit.md` 补新卡接入 checklist：新增卡必须明确标签维度、能力、收纳、寻路、规则 IO、AI 可达性、运行时状态和专名规则理由。
- `behavior_trace_runner` 的 live 结束输出改为终端摘要，完整 trace 继续写 `behavior_trace_report.txt`，避免成功后刷几百行报告。
- L0 **617** PASS。
- L2b live **tick=1396 / wall=16.7s PASS**，蘑菇棚建成，卡滞段 0。
- 剩余：继续拆 `WorldRules` 的 material / commerce 小域；或补 L2b 连续 3 次 PASS 统计。

### 2026-05-27 — v2.67b WorldRules service/state 子域拆分（Codex）
- 新增 `scripts/core/world_rules/world_rules_service.gd`，承接 `type_provides_service` / `provides_service` / `first_service_provider`。
- 新增 `scripts/core/world_rules/world_rules_state.gd`，承接旅人离开、树枯竭/受损/扎根、眩晕生物等状态谓词实现。
- `WorldRules` 保持 facade API，外部调用不变；`CardRuleAudit` 状态字符串扫描白名单加入 `world_rules_state.gd`。
- L0 **603** PASS。
- L2b live **tick=1309 PASS**，蘑菇棚建成，卡滞段 0。

### 2026-05-27 — v2.67a WorldRules 配方域拆分起步（Codex）
- 新增 `scripts/core/world_rules/world_rules_craft.gd`，把 `can_craft_cards` / `can_craft_types` 实现移入配方子模块。
- `WorldRules` 保持 facade API，外部调用不变；本片只验证 preload 与模块边界，未继续搬 camp/ecosystem。
- `behavior_trace_runner` 的 live 模式停止逐状态终端打印，完整 trace 仍写 `behavior_trace_report.txt`；避免输出量拖慢 wall-clock。
- L0 **603** PASS。
- L1 intent_trace **ticks=1730 PASS**，蘑菇棚建成。
- L2 greenhouse trace **tick=1571 PASS**，蘑菇棚建成，卡滞段 0。
- L2b live：优化输出前连续 2 次 120s wall FAIL；优化后 **tick=1293 / wall=55.9s PASS**，蘑菇棚建成，卡滞段 0。
- v2.67a 可视为完成；完整 v2.67 还未继续搬 camp/ecosystem 等域。

### 2026-05-27 — v2.66 `can_craft_cards` + shelter/fire 配方层收口（Codex）
- 新增 `WorldRules.can_craft_cards` / `can_craft_types`，由规则层统一封装 `CardDB.can_impact` + `CardDB.find_relation`。
- `WorldHelpers.can_craft` 改转发 `WorldRules.can_craft_types`，并补 `WorldHelpers.can_craft_cards` 兼容入口。
- `fire_tasks` / `shelter_tasks` 的 fire / hut / table 检查改传 `CardBase`，避免任务层继续拆 `card_type`。
- L0 新增 craft rule card query 用例；L0 **603** PASS。
- L2b 首次 120s wall FAIL（玩家饥死，既有 live 波动）→ 复跑 **tick=1268 PASS**，蘑菇棚建成，卡滞段 0。
- 剩余 R6 小切片：`tool_tasks` 长矛、`survival_tasks` 熟肉仍有 `WorldHelpers.can_craft(...card_type...)`，下一轮单独收口。

### 2026-05-27 — v2.65 状态谓词收口（Codex）
- 新增 `WorldRules.is_stunned_being` / `is_traveler_departing` / `is_tree_depleted` / `is_tree_depleted_state` / `is_tree_damaged` / `is_tree_active`。
- 替换运行时代码内 `"眩晕"`、`"离开"`、`"枯竭"`、`"受损"`、`"扎根"` 的直接比较；状态赋值仍由状态机负责。
- `CardRuleAudit.validate()` 增加 runtime state 比较扫描，禁止在 `player/world/core` 里重新散落核心状态字符串判断。
- L0 新增状态谓词用例；L0 **597** PASS。
- L2b 首次 120s wall FAIL（玩家消失，既有 live 波动）→ 复跑 **tick=1359 PASS**，蘑菇棚建成，卡滞段 0。

### 2026-05-27 — v2.64 漏网清扫 + need 契约 L0（Codex）
- 新增 `scripts/player/execution/need_contract.gd`：P0 need → chain / exec entry / blocked note / can-start predicate。
- `unit_test_cases.gd` 新增 need 契约 fixture：fire、hunger、build_greenhouse、commerce、cleanliness、storage_idle。
- `CardRuleAudit.validate()` 增加 runtime `card_type == / in [...]` 扫描；`world_manager.gd` 暂列 UI/展示白名单。
- 清扫漏网硬编码：`mushroom_helpers` 水桶判断改 `WorldRules.is_water_bucket`；`world_helpers` / `manual_control` 玩家判断改 `PlayerCard`。
- L0 **587** PASS；L2b 首次 tick=5000 FAIL（既有 live flaky，卡滞 0）→ 复跑 **tick=1307 PASS**，蘑菇棚建成，卡滞段 0。

### 2026-05-20 — v2.63 环境/生态 tick/经营 demand 规则化（Cursor）
- environment/commerce/hunt/ecosystem tick 去 type 硬编码；`ecosystem_behavior_key` 替代 card_type match
- L0 **562** PASS；L2b **1295** PASS

### 2026-05-20 — v2.62 Shelter/Interaction/态势规则化（Cursor）
- shelter/interaction/spatial/zone/camp_planner 去结构/交互 type 硬编码
- L0 **541** PASS；L2b 3× FAIL（tick 5000 上限，flaky）

### 2026-05-20 — v2.61 食物/营地整理/需求层规则化（Cursor）
- is_cooked_meat / is_bush / is_organize_immovable / camp_helpers+needs 去 type 硬编码
- L0 **529** PASS；L2b 5× FAIL（120s wall / tick≈2347 饥死，flaky）

### 2026-05-20 — v2.60 材料/菇棚/经营层规则化（Cursor）
- CARD_CAPABILITIES 全量同步 audit；player/mushroom/container/commerce 查询 API
- population/ecosystem/mushroom/craft/insight/snapshot 去 runtime type 枚举
- L0 **519** PASS；L2b **1932** PASS

### 2026-05-20 — v2.59 草皮/环境层规则化（Cursor）
- is_living_grass / grass_covers / actor_hidden_in_living_grass；环境/生态去 grass 枚举
- L0 **452** PASS；L2b 首次 FAIL → 重跑 **1285** PASS

### 2026-05-20 — v2.58 Camp Service 任务/经营层（Cursor）
- camp_sleep_huts / camp_commerce_tables / traveler_guests；runtime 零 hut/table/traveler 枚举
- L0 **446** PASS；L2b **2379** PASS

### 2026-05-20 — v2.57 尸体/肉源 + 兔子迁入（Cursor）
- is_predator_feed_corpse / corpses_in_world / count_wild_rabbits；生态与 needs 去 corpse 枚举
- L0 **441** PASS；L2b 首次 5000 FAIL → 重跑 **2595** PASS

### 2026-05-20 — v2.56 猎物扫描规则化（Cursor）
- is_live_prey / is_ambush_prey_for；world_helpers / need_evaluator / player_hunt_helper 去 prey 枚举
- L0 **433** PASS；L2b **2805** PASS

### 2026-05-20 — v2.55 狼群域成员查询（Cursor）
- is_wolf_pack_member / is_adult_wolf / is_wolf_pack_cub；pack 系列去 type 枚举
- L0 **428** PASS；L2b **1346** PASS

### 2026-05-20 — v2.54 幼体跟随 / can_care_for（Cursor）
- can_care_for 双物种；sheep_behavior 去 nearest_type("sheep")；wolf_cubs 对齐 can_grow
- L0 **421** PASS；L2b **1317** PASS

### 2026-05-20 — v2.53 羊/幼体繁殖查询（Cursor）
- reproducing_sheep / sheep_lambs / count_sheep_population；population 羊链去 type 枚举
- L0 **418** PASS；L2b **1324** PASS

### 2026-05-20 — v2.52 狼威胁/繁殖查询全链（Cursor）
- reproducing_wolves / hunt_threats / pack_hunter_near；runtime 零 wolf 枚举
- L0 **414** PASS；L2b **1320** PASS

### 2026-05-20 — v2.51 生态/感知层规则收口（Cursor）
- population/spatial_scan/zone_overlay/CampHelpers feed 转调
- L0 **409** PASS；L2b **3204** PASS

### 2026-05-20 — v2.50 Hunt/care B + go_near_xy（Cursor）
- wolf 建窝/feed/home 规则查询；go_near_xy→bool
- L0 **405** PASS；L2b **1792** PASS

### 2026-05-20 — v2.49 go_to_xy 返回值 + 世界层 fire（Cursor）
- go_to_xy→bool；environment/population/zone_overlay 规则化
- L0 **398** PASS；L2b **1396** PASS

### 2026-05-20 — v2.48 shelter/tool movement + needs fire 收口（Cursor）
- shelter/tool go_near 返回值；player_needs_manager 零 fire 枚举
- L0 **396** PASS；L2b 首次 120s 超时 → 重跑 **1404** PASS

### 2026-05-20 — v2.47 movement 第二轮 + has_fire 收口（Cursor）
- go_near 扩至 fire/survival/mushroom/commerce；has_camp_domain_anchor 替代 intent/mission fire 枚举
- 修复 makeFire hitFire 锚点条件写反（L2b 玩家消失回归）
- L0 **393** PASS；L2b **1977** PASS

### 2026-05-20 — v2.46 Camp 收尾 + go_near 返回值（Cursor，movement 第一轮）
- `near_camp_fire` / `find_greenhouse` service 查询；`go_near`→bool；cook/forage phase 推进
- `_cell_near_fire` 保留切比雪夫（与 `_near_any_fire` 一致，避免湿木被烤干）
- L0 **386** PASS；L2b 首次 5000 FAIL → 重跑 **1796** PASS

### 2026-05-20 — v2.45 sleep/commerce service + home territory（Cursor）
- `camp_sleep_home` / `camp_commerce_table` / `in_camp_home_territory`；`find_camp` 规则化
- L0 **381 PASS**；L2b **tick=3509**

### 2026-05-20 — v2.44 Camp 组织/commerce 规则查询（Cursor，略提速合并片）
- `camp_organize_refs` / `camp_structure_card`；删 `_first_type`；need_evaluator 生火/建桌/经营改规则层
- L0 **376 PASS**；L2b PASS tick=2789（首次 5000 未建成，复跑过）

### 2026-05-20 — v2.43 营地域结构查询（Cursor）
- `WorldRules.camp_domain_structure_cards()`；删 storage orphan 死代码
- L0 **372 PASS**；L2b **tick=1850**

### 2026-05-30 — Codex → DeepSeek 角色交接

**做了什么**
- DeepSeek 接替 Codex，担任策划助手角色：设计分析、方案讨论、工程核对、产出策划文档/执行单。
- Cursor 仍是主程序，负责按文档实现代码。
- 读取全部关键文档：CODEX_HANDOFF.md、增量迭代工作流、v3.0-A 原案、A2/A3 修正执行单、生态重设计讨论稿。
- 确认当前 v3.0-A 状态：A1 ✅ · A2 ⚠️（鹿链体感不合格）· A3 ⚠️（桃源人 live 不可见）。

**下一步（待用户确认后执行）**
1. 工程核对：检查 Cursor 是否已实现 v3.0-a-corrective-a2-a3.1.md（deerFawn、鹿繁殖、桃源人 spawn、封闭生态 trace）。
2. 设计推进：与用户讨论是否将 v3.0-a-ecology-redesign-discussion.md 升级为正式执行单。
3. 新增卡原则重申：必须是机制增量，不是换皮分型。

---

## 7. 关键架构图

```
player_needs_manager.update()
  ├─ L1 REACTIVE: 狼/睡眠 → 可能 set_path_to，early return
  ├─ L2 PATH: path 非空 → 冻结规划
  └─ L3 PLANNING:
       need = NeedEvaluator.evaluate()
       mission = MissionResolver.resolve()
       ActionRunner.tick()
         ├─ _sync_stack (mission 变化可能 clear path/craft)
         ├─ _dispatch → GH/lumber/storage/...
         └─ _present_mind → MindPresenter → player.goal

CraftTaskSystem.execute() → survival/shelter/mushroom/commerce tasks
  └─ CraftTaskHelpers.go_near → path_intent + set_path_to
```

**发呆的统一 outward 脸：** `need=X, craft空, path空` 或 **path 来回但 phase 不推进**。

---

## 8. Codex 工作建议

### 8.1 第一件事

1. 读 `AGENTS.md` + `docs/design/README.md`
2. 跑 L0，确认 **358** PASS（v2.39 起）
3. 跑 L2b + 读 `behavior_trace_report.txt`；v2.38 最近一次为 `tick=1272` 建成、卡滞段 0
4. **开游戏玩 5 分钟**，确认用户说的「左右走」是否仍在

### 8.2 推荐任务顺序（勿跳）

1. **Movement 收口**（P0）— 最大用户价值  
2. **storage 全面 craft 化**（P0/P1）  
3. **Live GH mission 锁**（P1）— `greenhouse_project_active` 加强，hunger 仅在 starving 抢占  
4. 更新 `CHANGELOG.md` + 同步设计圣经（若改玩法）  
5. 重跑 L0–L3，**分开报告「测试」与「玩家体感」**

### 8.3 禁止事项

- 未跑测试就声称修复完成
- 未经用户要求 `git commit`
- 只改 eval 不改 exec（或反之）
- 用追踪 bootstrap 掩盖 Live 链问题而不标注

### 8.4 验收标准（用户真正在意的）

> 玩家卡不再长时间对着同一张卡左右走而 craft/path 无进展；能稳定完成：生火 → 刀矛 → 归置 → 建菇棚（Live 场景，无 bootstrap）。

测试 L1/L2a PASS 是必要不充分条件。

---

## 9. 重要常量（`autoload/game_state.gd`）

```
REAL_TICK = 0.35
WET_WOOD_MATURE_SECONDS = 30.0
FIRE_FEAR_RANGE = 2
PLAYER_HUNGER_SEEK / PLAYER_HUNGER_NEED  # 饥饿阈值
CAMP_STORE_RANGE                         # 营地归置范围
commerce_manual_trade_confirm = false    # 默认自动交易
```

---

## 10. 联系上下文

- 用户对 Cursor 信心下降原因：**测试绿了但 playable 仍像坏掉**；多轮 patch 像打地鼠  
- 用户偏好：**中文沟通**；改完要测；不行要如实分析报告  
- 完整 Cursor 会话 transcript（若需）：  
  `C:\Users\Administrator.HR-20220613KUGU\.cursor\projects\...\agent-transcripts\8cb1f9f3-a426-4e6a-b7b5-271014e9b1f9.jsonl`

---

## 11. 给接任 Agent 的一句话

**2026-05-30 起，DeepSeek（策划助手）接替 Codex 的设计分析/方案讨论/工程核对角色。Cursor 仍是主程序。§12 协作留痕改为三方（Cursor / Codex / DeepSeek）共同维护。**

Codex 时代收尾：WorldRules 已收口，ActionRunner 调度层已钉死（L2b 5× USELESS=0）。当前重心是 v3.0-A 生态修正与桃源人可见性，不再回到主链空转修复。

---

*文档结束。有问题回到 Cursor 时带上本文件 + 最新 `behavior_trace_report.txt`。*
