# Layer 0 Intent 问题 backlog

> 遇问题先记录再分析。归类：`judgment` | `execution` | `situation` | `display`

---

## 模板

```text
### [YYYY-MM-DD] 标题
- 现象：
- Intent 序列：
- 态势 progress_key：
- 归类：
- 分析：（判断层 / 执行层 / …）
- 状态：open | deferred | fixed
```

---

## 2026-05-20 追踪（判断层实施前 baseline）

### 敲山卡死 · craft 空
- 现象：goal=敲山取石，craft=空，path=0，3000+ tick 不动；从未进入 buildGreenhouse FSM
- Intent 序列：（无，旧架构）
- progress_key：卡在 no_lumber
- 归类：execution + judgment（无判断层时 imperative 挂 goal）
- 分析：备木走 `_ensure_wood_supply` 碎函数，到山边非邻格后无重选
- 状态：open → 用 Intent `prepare_wood` + replan 覆盖；执行层敲山仍待修

### eval 与 need 分裂
- 现象：current_need=build_greenhouse，eval 变 storage_idle / hunger，行为不变
- 归类：judgment
- 分析：无统一 Intent，eval 不能打断假承诺
- 状态：deferred（Intent 层已加 survive_hunger 抢占规则，待 trace 验证）

### 饥死未进食
- 现象：eval=hunger 后仍敲山，最终饥死
- 归类：judgment
- 分析：build_greenhouse satisfy 虚报成功，饥饿未抢占
- 状态：deferred（survive_hunger intent 待验证）

---

## 实施判断层后

### 2026-05-20 Intent trace #1（判断层已接入）

- 现象：Intent 正确选 `prepare_wood`，goal 显示「备木（制斧/砍树）」；craft 走 makeKnife→shapeTri×2 后 craft 空；3392 tick 卡滞于 `prog=no_lumber`；未进入 buildGreenhouse FSM；饥死（trace 未 clamp 饥饿）
- Intent 序列：`prepare_wood`（仅 1 步，缺后续 4 步）
- progress_key：全程 `no_lumber`，未前进
- 归类：**judgment 部分 OK**（意向选对+展示对）；**execution open**（备木链未完成）；**situation open**（progress 不更新）
- 分析：判断层与 eval=build_greenhouse 对齐；执行层 `_ensure_wood_supply` 在 shapeTri 后停滞，态势不前进故 Intent 不重选；REPLAN_STALL_TICKS=90 但 stall 计数在 craft 空+path 空时增长慢，且饥死打断
- 状态：judgment **partial pass**；execution **open**

### 待 trace #2（已加 intent_trace 饥饿 clamp）

- 目的：排除饥死干扰，观察 progress 前进时 Intent 链是否推进
- 命令：`powershell -File tools/run_intent_trace.ps1` 或 `scenes/intent_trace.tscn`

