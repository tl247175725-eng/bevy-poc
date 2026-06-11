# 全局公告栏系统

**From**: deepseek-v4
**To**: cursor
**Date**: 2026-06-11
**Priority**: P1（智能实体不再原地打转——有全局方向感）

## 架构计划
新增 BulletinBoard 资源，N tick 更新一次（全局扫描）。实体通过标签匹配按需查阅对应频道。不改变现有感知系统（6 格本地感知不变），公告栏提供更大方向感。

## 架构反馈
复用现有标签系统和 spatial_index，新增资源层。

## 智能验收
- 羊饿时能朝食物区域方向移动（非仅本地 6 格扫描）
- 公告栏 50 tick 更新一次，其余 tick 只读（无锁冲突）
- smoke test PASS

---

## 实现

### 1. BulletinBoard 结构

```rust
// src/bulletin.rs (新建)

pub struct Zone {
    pub center_x: u8,    // 区域中心
    pub center_y: u8,
    pub radius: u8,      // 区域半径（格）
    pub intensity: u8,   // 密度/活跃度 0-100
}

pub struct BulletinBoard {
    pub channels: HashMap<String, Vec<Zone>>,  // channel_name → zones
    pub last_update_tick: u64,
}

impl BulletinBoard {
    const UPDATE_INTERVAL: u64 = 50;
    
    pub fn needs_update(&self, tick: u64) -> bool {
        tick - self.last_update_tick >= Self::UPDATE_INTERVAL
    }
}
```

### 2. 频道定义

```
predator_zones   → 捕食者的位置（密度 = 该区域捕食者数量）
prey_zones       → 猎物的位置
food_zones       → 食物源的位置
water_zones      → 水域的位置
corpse_zones     → 尸体/腐肉的位置
shelter_zones    → 遮蔽物（草/灌木）的区域
```

### 3. 更新逻辑

每 50 tick：扫描全图，按 8×8 区块统计各标签实体密度 → 写入对应频道。

```
将地图划分为 8×8 区块（当前 36×24 → 约 4×3 个区块）
每区块统计各类型实体数
密度 > 阈值 → 加入对应频道
```

### 4. 实体查阅

```rust
// 在 tick_reactive 的决策阶段
if let Some(board) = &world.bulletin_board {
    // 兔：查 food_zones + corpse_zones
    if profile.has_bulletin_access("food_zones") {
        if let Some(food_zones) = board.channels.get("food_zones") {
            for zone in food_zones {
                let dist = chebyshev_distance(x, y, zone.center_x, zone.center_y);
                // 向食物区域方向偏转
                if dist <= zone.radius * 2 {
                    // 该区域在当前方向，增加 seek 倾向
                }
            }
        }
    }
}
```

### 5. 标签控制访问

| 标签 | 可查频道 |
|---|---|
| `bulletin:food` | food_zones |
| `bulletin:predator` | predator_zones |
| `bulletin:prey` | prey_zones |
| `bulletin:water` | water_zones |
| `bulletin:full` | 全部频道（人类） |
| 无标签 | 无访问（昆虫、植物） |

### 6. 集成到 tick_reactive

在 `active_drives` 生成后、`execute_drive` 前，查询公告栏调整 drive 优先级或增加方向性。

```
// 如果一个局部的 seek 得分为 0（附近没食物），但公告栏显示北边有食物区
// → 降低 seek 的"放弃门槛"，让动物愿意走更远
```

## 涉及文件

| 文件 | 改动 |
|---|---|
| `src/bulletin.rs` | 新建：BulletinBoard + 更新逻辑 |
| `src/world_state.rs` | WorldState 新增 bulletin_board 字段 |
| `src/systems/tick_reactive.rs` | 决策阶段读公告栏 |
| `src/axioms/profile.rs` | 新增 has_bulletin_access |
| `assets/card_defs.ron` | 动物加 bulletin:* 标签 |

## 验收
- `cargo check` 0 错误
- `cargo test --release` 全 PASS
- `cargo run --release -- --smoke-test` PASS
- 公告栏 50 tick 更新一次，实体按标签查阅对应频道
