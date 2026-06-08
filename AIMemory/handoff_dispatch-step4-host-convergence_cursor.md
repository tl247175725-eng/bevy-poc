# 调度改造 Step 4：宿主收敛（den / scavenge）

**From**: cursor | **To**: cursor（自执行）  
**Date**: 2026-05-30 | **Parent**: proposition 6 步表 Step 4  
**前置**: Step3 能力管线 DONE（779）

---

## 目标

`ecosystem_manager` 狼/狐平行 API 收敛为 actor 统一入口；灌木窝 vs 草窝由 `support_den` + 材料能力（`world_rules.actor_den_build_mode`）区分。

| 统一 API | 职责 |
|----------|------|
| `den_for(actor)` | `home_for_actor` |
| `should_build_den(actor)` | `should_actor_build_den` |
| `update_den_work(actor, real_delta)` | 草窝 `_update_grass_den_work` / 灌木 `_update_bush_den_work` |
| `update_build_den(actor, real_delta)` | 仅灌木占灌丛（草窝筑巢仍在 grass work 内） |
| `try_scavenge(actor, real_delta)` | 清腐（`capability.scavenge`） |
| `find_meat_source` / `take_meat_from_source` | 狼取肉（管线 den_work） |

**保留薄 wrapper**（兼容旧调用/单测）：`den_for_wolf/fox`、`wolf_should_build_den`、`update_wolf_den_work`、`update_fox_*`、`try_fox_scavenge`、`find_wolf_meat_source`、`take_meat_for_wolf`。

**本步不碰**：`wolf_attack`/`fox_attack`、`nearest_wolf`、`should_wolf_leave`（Step5/6 或后续）。

---

## world_rules 新增

- `DEN_BUILD_MODE_GRASS` / `DEN_BUILD_MODE_BUSH`
- `actor_den_build_mode(actor)` — `predator`+`return_home` → 草窝；`mesopredator` → 灌木窝
- `designated_den_builder_for_actor(actor)`、`should_actor_build_den(actor)`

---

## 文件

| 文件 | 操作 |
|------|------|
| `world_rules.gd` | den build mode + should build |
| `ecosystem_manager.gd` | 统一 API + 私有 `_update_grass_*` / `_update_bush_*` / `_build_bush_den` |
| `capability_behavior_pipeline.gd` | 调用 `den_for` / `update_den_work` / `try_scavenge` / `find_meat_source` |
| `wolf_behavior.gd` / `fox_behavior.gd` | `den_for` |
| `unit_test_cases.gd` | `_test_host_den_convergence` |

---

## 验收

- L0 全绿（断言计数 ≥779，含 `_test_host_den_convergence`）
- 狼窝单测仍过（`update_den_work` 与 `update_wolf_den_work` 等价）
- 管线 den_work / scavenge 无 `den_for_wolf` 直连（除 wrapper 定义处）

---

## 子任务（自拆）

1. [x] world_rules：`actor_den_build_mode` + `should_actor_build_den`
2. [x] ecosystem_manager：统一 API + 实现下沉
3. [x] pipeline + behaviors 改调用
4. [x] `_test_host_den_convergence`
5. [ ] L0 headless → 更新 #回复 / CHANGELOG / fix-log / work.log（本环境无 Godot CLI，待本地 `tools/run_unit_tests.ps1`）

---

## #回复

**代码**：Step4 宿主收敛已合入。`_update_bush_den_work`：叼肉无窝时仍走 `_build_bush_den`（对齐原 `update_fox_build_den` 分支）。

**L0**：本机未检测到 Godot，未跑 headless；请本地跑 `powershell -File tools/run_unit_tests.ps1`，预期断言 ≥780（+`_test_host_den_convergence`）。
