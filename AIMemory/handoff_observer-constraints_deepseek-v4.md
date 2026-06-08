# Observer 迁移约束清单

**状态**：设计约束，确认后转 handoff
**日期**：2026-06-06
**前提**：Phase 0-4 已完工，143 断言全绿

---

## 一、选型

**方案 B**：WorldState 内自建 SimObserver。在 spawn / move / kill 三个 choke point 发事件，Observer 调已有 `tick_*` 函数。不迁 Bevy ECS。

**不做方案 A**（仿真实体全迁 Bevy ECS）——工作量太大，B 已够用。
**不做方案 C**（只统一日志不改调度）——不是真正的 Observer。

---

## 二、范围：两刀

### 第一刀：OnKill + OnSpawn 日志统一（不改 bucket tick）

- 合并 `ecology_log`（字符串）和 `SimEventQueue`（结构化）为统一 `SimEvent` 枚举
- spawn / kill 时发 SimEvent → Observer 写日志 + 通知 UI
- bucket tick 原样保留

**验收**：143 断言不降。日志内容与 Godot 一致。

### 第二刀：OnMove → 邻居通知（加在 bucket tick 之上，不替换）

- 卡移动时发 `OnMove(entity, old_pos, new_pos)`
- Observer 收到 → 通知新格/旧格邻居（predator 感知 prey 进入范围、prey 感知 predator 靠近）
- **捕食者仍保留每 tick patrol**（解决"都不动但相邻"场景）
- 食草动物寻食可逐步切到 OnMove——羊到新位置后扫周围有无草

**不做**：纯 OnMove 替代全量轮询。狼羊都不动时必须仍能触发 hunt。

**验收**：143 断言不降。assert_20 1000 tick 稳定性不退化。

---

## 三、不做

- 繁殖仍走 FixedTimestep（没触发源）
- 草再生/腐坏/风化 仍走 FixedTimestep（纯时间驱动）
- 不删现有 `tick_*` 函数——Observer 内部调用它们
- 不迁 Bevy ECS

---

## 四、必保断言（每刀跑全量 143，但特别盯这几条）

| 断言 | 为何关键 |
|------|---------|
| assert_06 hunt_range 内狼发起捕猎 | Observer 不能漏掉 hunt 场景 |
| assert_07 猎杀后羊变尸体 | OnKill 触发链条完整 |
| assert_08 fear_range 内羊逃跑 | 邻居通知必须覆盖 flee |
| assert_14 超出 hunt_range 不猎 | 不该触发时不触发 |
| assert_20 1000 tick 稳定性 | tick 顺序变化不能破坏生态自洽 |
| m2_11/14 狼 hunt 相关 | 移动触发 + patrol 都必须覆盖 |

---

## 五、约束

- `card_defs.ron` 不改
- 行为逻辑（`tick_one_predator`/`tick_one_grazer`/`try_hunt` 等）不改
- 每刀跑 `cargo test`，143 不降
- 记 FIX_LOG
