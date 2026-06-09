# 群聚系统 — 分离/聚合/对齐 + 群卡视觉

**From**: deepseek-v4
**To**: cursor
**Date**: 2026-06-09
**Priority**: P1（架构——解决卡牌叠堆问题的正确方案）

---

## 背景

当前多个同种动物会挤在同一格。之前修了 size→1 缓解了，但本质上缺少群聚动力学。真正的解决方式是 Boids 三力引擎 + 群卡视觉表现。

## 设计文档

见 `AIMemory/design_flock-system_deepseek-v4.md`。

---

## 架构

```
tick_reactive 中 drive:flock 分支
  │
  ├→ 在 flock_range 内找同种邻居
  ├→ 计算 separation（推开最近的）
  ├→ 计算 cohesion（向重心靠拢）
  ├→ 计算 alignment（对齐方向）
  ├→ 三力加权合成 → 本次移动方向
  ├→ 检查 predator 信号 → 按 flock_alert 模式散开
  └→ 检查群规模 → 超阈值分裂
```

群卡视觉由 `sync_card_visuals` / 新增 `group_card_render` 负责：同种 ≥3 只同格 → 渲染群卡而非单卡。

---

## P0：EntityProfile 新增字段

```rust
// axioms/profile.rs

pub struct EntityProfile {
    // ... 现有字段 ...

    // 群聚参数
    pub social_structure: SocialStructure,  // flock / pack / herd / none
    pub flock_cohesion: f32,               // 0.0-1.0
    pub flock_separation: f32,             // 0.0-1.0
    pub flock_range: u8,                   // 感知范围（格）
    pub flock_max: u8,                     // 自然上限
    pub flock_split_threshold: f32,        // 分裂阈值倍率
    pub flock_alert: AlertMode,            // startle / scatter / stampede / school / none
    pub flock_alert_range: u8,             // 惊散传播范围
}

pub enum SocialStructure {
    Flock,   // 无中心聚合
    Pack,    // 巢穴中心
    Herd,    // 松散对齐
    None,    // 独居
}

pub enum AlertMode {
    Startle,
    Scatter,
    Stampede,
    School,
    None,
}
```

所有新字段默认值（无标签时）：
- `social_structure: None`, `cohesion: 0.0`, `separation: 0.0`, `range: 0`, `max: 1`, `split_threshold: 1.5`, `alert: None`, `alert_range: 0`

---

## P0：标签解析

`parse_flock_params(tags) -> (SocialStructure, f32, f32, u8, u8, f32, AlertMode, u8)`

标签格式：
```
social_structure:flock
flock_cohesion:0.8
flock_separation:0.3
flock_range:4
flock_max:8
flock_alert:startle
flock_alert_range:3
```

`parse_flock_params` 逻辑：
1. 查找 `social_structure:xxx` 标签 → 确定结构类型
2. 按上表解析各参数，每种结构有默认 fallback
3. None 类型 → 所有参数归零

---

## P0：修改 tick_reactive — drive:flock 分支

当前 `drive:flock` 只是向同类平均位置 `move_toward`。扩展为 Boids 三力：

```rust
fn execute_flock(world: &mut WorldState, id: EntityId, x: u8, y: u8, profile: &EntityProfile) {
    let neighbors = world.query_near_filtered(x, y, &profile.type_name, 
        profile.flock_range, id);
    
    if neighbors.len() < 2 {
        // 没有足够邻居 → 回到普通 seek/wander
        return;
    }

    // 1. Separation — 推开最近的邻居
    let mut sep_dx: i16 = 0;
    let mut sep_dy: i16 = 0;
    for &nid in &neighbors {
        let (nx, ny) = world.spatial_index.position(nid).unwrap_or((x, y));
        let dist = crate::world_rules::chebyshev_distance(x, y, nx, ny) as i16;
        if dist <= 2 && dist > 0 {
            sep_dx += (x as i16 - nx as i16) * 2;
            sep_dy += (y as i16 - ny as i16) * 2;
        }
    }
    sep_dx = sep_dx.clamp(-1, 1);
    sep_dy = sep_dy.clamp(-1, 1);

    // 2. Cohesion — 向邻居重心
    let (avg_x, avg_y) = average_position(world, &neighbors);
    let coh_dx = (avg_x as i16 - x as i16).signum();
    let coh_dy = (avg_y as i16 - y as i16).signum();

    // 3. 加权合成
    // 向心力 = cohesion_weight × cohesion_dir
    // 分离力 = separation_weight × separation_dir
    // 实际的 dx,dy = 加权和 → clamp 到 {-1,0,1} → Manhattan 移动

    // 4. 超上限 → cohesion 减弱
    let count = neighbors.len() as u8 + 1;
    let coh_mult = if count > profile.flock_max { 
        0.5 
    } else { 
        1.0 
    };

    // 5. 三力加权
    let total_x = (coh_dx as f32 * profile.flock_cohesion * coh_mult)
                + (sep_dx as f32 * profile.flock_separation);
    let total_y = (coh_dy as f32 * profile.flock_cohesion * coh_mult)
                + (sep_dy as f32 * profile.flock_separation);

    // 6. Manhattan 移动
    let (final_dx, final_dy) = if total_x.abs() >= total_y.abs() {
        (total_x.signum() as i16, 0)
    } else {
        (0, total_y.signum() as i16)
    };
    // 如果分离力强于聚合 → 分离方向优先
    let (final_dx, final_dy) = if sep_dx != 0 || sep_dy != 0 {
        (sep_dx, sep_dy)  // 分离优先
    } else {
        (final_dx, final_dy)
    };

    let gx = (x as i16 + final_dx).clamp(0, GRID_WIDTH as i16 - 1) as u8;
    let gy = (y as i16 + final_dy).clamp(0, GRID_HEIGHT as i16 - 1) as u8;
    if (gx, gy) != (x, y) {
        move_toward(world, id, x, y, gx, gy);
    }
}
```

---

## P0：惊散逻辑

在 `tick_reactive` 中，如果当前 entity 的 `flock_alert` 不是 None 且有 predator 在 `flock_alert_range` 内，则：

```
match alert_mode:
  startle:  随机方向走 2-3 格，cohesion 暂时减弱 3 tick
  scatter:  反向于 predator 方向走 5-8 格，cohesion 归零 10 tick
  stampede: 所有邻居朝同一方向（远离 predator）走 10+ 格
  school:   围绕 predator 分成两半——左半向左、右半向右
  none:     不触发
```

实现方式：在 `Entity` 上临时设置 `scatter_timer: i8`（剩余散开 tick 数）。散开期间 `flock_cohesion` 归零。

---

## P1：群卡视觉

当同种 ≥3 只实体在同一格时，渲染层绘制群卡而非单卡。

### 视觉规格

```
┌─────────────┐
│ 群     (5)   │  ← 绿色圆圈 + 白色数字，右上角
│             │
│    羊        │  ← 白色，居中
└─────────────┘
  动物底色（CardDef.color）
```

### 实现方式

`sync_card_visuals` 或新的 `group_card_render` 系统：

1. 在每帧遍历所有卡牌，按 `(x, y, type_name)` 聚合
2. 同格同种 ≥3 只 → 渲染一张群卡
3. 群卡 spawn 为特殊的 CardVisual + GroupCardMarker 组件
4. 原独立卡牌 hide（不 despawn），底层实体仍然存在
5. 数字变化时更新群卡上的计数字段

### 群卡组件

```rust
#[derive(Component)]
pub struct GroupCardMarker {
    pub count: u8,
    pub member_ids: Vec<EntityId>,  // 底层独立实体ID
}
```

---

## 涉及文件

| 文件 | 改动 |
|---|---|
| `src/axioms/profile.rs` | EntityProfile 新增群聚字段 + parse_flock_params |
| `src/systems/tick_reactive.rs` | drive:flock 分支 → Boids 三力引擎 + 惊散 + 分裂 |
| `src/world_state.rs` | Entity 加 `scatter_timer: i8` |
| `src/card_visual.rs` | 新增 `group_card_render` → 群卡视觉 |
| `AIMemory/design_flock-system_deepseek-v4.md` | 已有 |
| `assets/card_defs.ron` | 所有群居动物加 flock 标签 |

## 验收

1. `cargo test --release` 全 PASS
2. `cargo run --release -- --smoke-test` SMOKE: PASS
3. 启动游戏：
   - 3+ 只羊同格 → 显示群卡，绿圈标注数量
   - 狼靠近 → 羊惊散，群卡消失、单卡散开
   - 分散后安全 → 逐渐重聚
   - 狼 pack 围绕巢穴活动
   - 虎完全独居，从不聚合

## 约束

- 不碰公理层
- 不改 tick_reactive 的非 flock 分支
- 不碰 Bevy plugin 注册逻辑（在需要用到的 module 中自行加）
