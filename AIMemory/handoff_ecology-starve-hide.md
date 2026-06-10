# 草地/饿死/藏容纳 — 批量落地

**From**: deepseek-v4
**To**: cursor
**Date**: 2026-06-10
**Priority**: P0

## 架构计划
全部标签驱动，复用现有系统。草改存量 HP 用 compose 已有逻辑。饿死用公理 transform(Starve)。藏用容纳体系（host_cover_id + in_cover）。

## 架构反馈
新增 `in_cover` 状态布尔。饿死在公理层完成。

## 智能验收
- 草有 HP 存量，不被一口吃光
- 动物会饿死且尸体匹配种类
- 兔子藏入草后不可见，草显示紫藏标

---

## P0：草和灌木改为存量消耗

### 草皮
- HP = 4（一口 -1）
- 再生：每 15 tick +1 HP（湿地周边 2 格内）
- 生成位置：湿地周边 2 格内，不在顶排
- 湿地面积扩大 3×

### 灌木
- HP = 8
- 再生：每 30 tick +1 HP

### 实现
`try_eat_grass` 改为 `hp -= 1` 而不是 `remove_entity`。HP 归零才移除。`grass_regen` 改为在湿地周边给草 +1 HP（不满时）。

---

## P0：饿死逻辑

### 公理层
`tick_environment` 或独立 `tick_starvation` 系统：

```
每 tick：实体 fed_today == false → starve_days += (1 / TICKS_PER_DAY)
starve_days >= max_starve → transform(Starve) → kill → 尸体
```

`fed_today` 每天结束时重置。

### 标签 `max_starve:N`

| 动物 | 天数 |
|---|---|
| 鼠、竹鼠、鱼 | 2 |
| 兔、鸡、狐崽、狼崽、羊羔、鹿崽 | 3 |
| 狐 | 4 |
| 羊、鹿、人、村民、野牛崽 | 5 |
| 狼 | 6 |
| 野牛 | 7 |
| 水蝽、陆虫 | 1 |

---

## P0：藏 = 容纳

草和灌木有 `cover.small` 标签。动物通过 `drive:hide` 钻入：
- 动物设置 `in_cover = true`，`host_cover_id = cover_card_id`
- `entity_occupies_active_cell` 对 in_cover 实体返回 false（不占格）
- 草/灌木显示紫藏标

### Entity 新增字段
```rust
pub in_cover: bool,
pub host_cover_id: Option<EntityId>,
```

### 加入渲染过滤
`sync_card_visuals` 和 `stack_indices` 过滤 `in_cover` 实体。

---

## 涉及文件

| 文件 | 改动 |
|---|---|
| `assets/card_defs.ron` | 全动物加 max_starve；草 HP=4 灌木 HP=8 |
| `src/systems/grass_regen.rs` | 草存量再生 + 湿地绑定 |
| `src/systems/tick_herbivore.rs` | try_eat_grass 改为 HP-1 |
| `src/systems/tick_environment.rs` | tick_starvation |
| `src/world_state.rs` | Entity 加 in_cover / host_cover_id / starve_days |
| `src/card_visual.rs` | 过滤 in_cover 实体 |
| `src/axioms/composition.rs` | entity_occupies_active_cell 排除 in_cover |
| `AIMemory/design_ecology-chase-sprint_deepseek-v4.md` | 更新 |
