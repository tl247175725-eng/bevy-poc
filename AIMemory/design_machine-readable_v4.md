# 方寸商国：桃花源记 — 设计规范（机读版）

> 每一条对应一个可执行断言。用于代码审查和自动化测试。

---

## 1. 世界基础规则

### 1.1 一格一卡

```
触发：任何实体尝试进入格 (x,y)
操作：compose 检查 cell.living_count
规则：
  - incorporeal 实体 → 允许（不增加 living_count）
  - 尸体 → 允许
  - living_count == 0 → 允许
  - living_count > 0 → 拒绝（返回 Denied { max: 1 }）
验证：对于任意实体 E 和格 C，C 中非 incorporeal 非尸体实体数 ≤ 1
```

### 1.2 昆虫无实卡

```
触发：waterBug / landBug spawn
操作：in_pool=true（水虫）或 in_ground=true（陆虫）
结果：渲染层跳过（sync_card_visuals 过滤 in_pool || in_ground）
验证：查询 CardVisual 组件，不存在 type_name=="waterBug" 或 "landBug" 的实体
```

### 1.3 不可移动实体

```
触发：yield/shove 逻辑
操作：检查 blocker 标签
  - card_has_tag(def, "rooted") → 不可移动
  - def.is_rooted → 不可移动
  - entity.in_tree → 不可移动
  - entity.in_pool → 不可移动
验证：山、树、石头不被 yield/shove 改变位置
```

---

## 2. 藏 = 容纳

### 2.1 钻进

```
触发：entity 在 cover 格 + drive::Hide 激活
操作：
  1. entity.in_cover = true
  2. entity.host_cover_id = cover_id
  3. cell_composition.vacate_entity（不占格）
  4. cover 卡显示紫藏标
验证：
  - entity 的 CardVisual 不可见
  - entity 所在格 cell_composition living_count 不包含该 entity
  - cover 卡 HideBadge 组件存在
```

### 2.2 感知

```
触发：predator query_near_filtered
操作：
  - 目标在 cover 中 + distance > 1 → 不可感知
  - 目标在 cover 中 + distance == 1 + predator 有 keen_scent → 可感知
验证：狼在 2 格外不能感知草中兔；邻格+嗅觉灵敏可感知
```

### 2.3 释放

```
触发：cover 被破坏（HP==0）或 entity 主动退出
操作：
  - 破坏 → 所有 in_cover 实体弹出到邻格
  - hide:exit_when_safe → 威胁消失 → entity.in_cover = false
  - hide:exit_when_hungry → need:eat 超阈值 → entity.in_cover = false
验证：破坏草皮 → 兔出现在邻格；安全后兔自动出草
```

---

## 3. 禁止硬编码

```
规则：代码中不允许出现 if type_name == "wolf" 式的物种分叉
替代：使用 card_has_tag / card_has_capability / EntityProfile 字段
审查项：grep 'type_name\s*==' src/ | grep -v test | grep -v corpse_type | grep -v "player\|grass\|algae\|waterBug\|landBug"
```

---

## 4. 曼哈顿移动

```
触发：move_toward / flee_from / wander
规则：任何一次移动，dx 和 dy 不能同时非零
  - dx != 0 && dy != 0 → 随机选一个轴
验证：smoke test 中任意实体的相邻两次位置，|Δx|+|Δy| == 1
```

---

## 5. 公理

### 5.1 compose

```
输入：cell_slot, entity_profile
输出：Allowed / Denied
规则：见 1.1
```

### 5.2 traverse

```
输入：entity_profile, from_medium, to_medium
输出：Allowed / Denied
规则：
  - from == to → Allowed
  - is_omnimedium → Allowed
  - native_medium == to → Allowed
  - bridges 包含 (from, to) → Allowed
  - 否则 → Denied
```

### 5.3 perceive

```
输入：observer_profile, target_profile, distance, media_conditions
输出：Detected / Undetected
规则：
  - 跨介质且无 cross_perception → Undetected
  - effective_range = channel.range × keen_mod × visibility_mod × conduction
  - effective_range >= distance → Detected
```

### 5.4 transform

```
输入：source_profile, target_profile, action
输出：Transformation { drawn, received, lost }
规则：received = drawn × efficiency(action)
```

---

## 6. 交互

### 6.1 砸

```
触发：左键拖拽实体碰撞目标
规则：
  - 目标 HP > 0 → HP -= 1（攻击）
  - 目标 HP == 0 → smashes_accumulated += 1；累计 == 2 → 加工出产品（加工）
  - 同一接触不重复计数（必须拉开）
验证：拖石碰石 ×2 → 碎石生成；拖石碰羊 → 羊 HP-1
```

### 6.2 叠

```
触发：右键拖拽实体放置到目标
规则：
  - 半透明幽灵，无碰撞
  - 查标签组合规则表 → 有匹配 → 组合变新卡或加标签
  - 无匹配 → 弹回
验证：篝火叠水格 → 木炭；叠不兼容目标 → 弹回
```

---

## 7. 生态

### 7.1 腐败

```
触发：非山石类实体进入死亡/切断/潮湿状态
规则：
  - decay 阶段递增（每 N tick）
  - 潮湿 → 加速；干燥 → 减速；寒冷 → 冻结
  - severe → remove → spawn humus
验证：尸体在湿地 → 30 tick 变腐殖土；在旱地 → 60 tick
```

### 7.2 饿死

```
触发：每 tick tick_starvation
规则：
  - fed_today == false → starve_days += 1/TICKS_PER_DAY
  - starve_days >= max_starve → finalize_prey_kill → 尸体
验证：smoke test 中 predation_ticks > 0 且 herbivore 移动 > 0
```

### 7.3 衰老

```
触发：每 tick
规则：age > max_age → finalize_prey_kill（自然死亡）
验证：长期 smoke test 中存在非捕食、非饿死的死亡
```

### 7.4 草存量

```
触发：动物 eat grass
规则：grass.hp -= 1；hp == 0 → remove
验证：同一草被吃 4 次才消失
```

---

## 8. 全局验证（smoke test 必须通过）

```
规则：
  1. entity count in [30, 900]
  2. herbivore tick count >= 100/1000 ticks
  3. predation_ticks > 0
  4. max_tick_ms < 15.0
  5. no entity out of bounds
  6. predators > 0 and herbivores > 0
  7. no NaN coordinates
  8. event queue pending < 256
```

---

---

## 9. 行为契约

### 9.1 move_entity 返回值检查

```
契约：所有调用 world.move_entity() 的代码必须检查返回值
违规：丢弃 MoveResult（如 wander、flee_from 曾犯）
检测：grep 'world\.move_entity' src/ | grep -v '== MoveResult|!= MoveResult'
审查文件：movement.rs, tick_environment.rs, ui_interaction.rs, interaction/mod.rs
```

### 9.2 spawn 必须注册 tag-dictionary

```
契约：card_defs.ron 中每个新标签必须已在 tag-dictionary.md 中注册
检测：对比 card_defs.ron 中的标签与 tag-dictionary.md 中的注册表
```

### 9.3 compose 调用前必须 check

```
契约：不能假设 compose 返回 Allowed。所有 spawn/move 前必须调用 can_occupy 或 traverse
违规：直接设 entity.x/entity.y 不经过 move_entity
检测：grep 'entity\.x\s*=' src/ （仅允许在 move_entity 函数体内）
```

### 9.4 Cursor 提交前必须

```
契约：
  1. cargo check 0 错误
  2. cargo test --release 全 PASS
  3. cargo run --release -- --smoke-test PASS
  4. git commit + git push
违规：只 commit 不 push
```

---

## 10. 测试映射

| 机读断言 | 对应测试 |
|---|---|
| 1.1 一格一卡 | smoke_test entity count check |
| 1.2 昆虫无实卡 | m2_22b_landbug_kill_no_corpse_panic |
| 1.3 不可移动实体 | movement::rooted_mountain_not_displaced_by_yield |
| 2.1 钻进 | m2_03_rabbit_hides_in_grass |
| 2.3 释放 | cross_layer_initial_bird_nest_contained_in_tree |
| 4. 曼哈顿 | movement::manhattan_step_never_diagonal |
| 4. 避让 | movement::higher_priority_yield_enters_blocked_cell |
| 4. 互换 | movement::face_to_face_swap_via_yield |
| 6.1 砸 | assert_07_hunt_produces_corpse |
| 7.2 饿死 | smoke_test predation_ticks > 0 |
| 7.4 草存量 | 待添加 |
| 7.1 腐败 | 待添加 |
| 5.x 公理 | 通过 smoke_test + unit tests 间接覆盖 |

---

## 变更记录

| 日期 | 变更 | 原因 |
|---|---|---|
| 2026-06-10 | 初版 | 设计讨论完成 |
| 2026-06-11 | 补标签字典 | 防止 Cursor 发明不存在的标签 |
| 2026-06-11 | 补行为契约 | move_entity 返回值被忽略是反复出现的 bug |
| 2026-06-11 | 补测试映射 | 链接断言和实际测试 |
| 2026-06-11 | 机读版与 tag-dictionary 分离 | 机读版管断言，字典管合法标签集 |

---

*此文件与人读版 design_human-readable_v4.md、标签字典 tag-dictionary.md 三件套互配。修改任一需同步另二。*
