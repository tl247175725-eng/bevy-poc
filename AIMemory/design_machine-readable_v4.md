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

*每条规则对应一个测试或审查 grep。*
