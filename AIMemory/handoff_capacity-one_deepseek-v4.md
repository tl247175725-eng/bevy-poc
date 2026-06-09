# 每格一卡 + 例外规则

**From**: deepseek-v4
**To**: cursor
**Date**: 2026-06-09
**Priority**: P0（设计基准——所有卡默认不叠，只有特定逻辑允许同格）

---

## 规则

| 情况 | 同格 | 说明 |
|---|---|---|
| **默认** | 一格一张 | 狼不叠羊、草不叠草 |
| **群居同种** | 允许，上限 flock_max | `social_structure != None` |
| **容纳** | 允许 | `in_tree/in_pool/in_ground/in_den` |
| **隐藏** | 允许 | `hidden_in_grass` / `in_burrow`（临时状态） |
| **实体+尸体** | 允许 | 尸体是死物，不占活动空间 |
| **捕猎/进食** | 不进入目标格 | 相邻格操作——狼在 (5,3) 杀 (5,4) 的羊 |

---

## P0：compose 改为默认拒绝

`src/axioms/laws.rs` `compose` 函数：

```rust
pub fn compose(cell: &CellSlot, incoming: &EntityProfile) -> Composition {
    if incoming.incorporeal {
        return Allowed;
    }
    
    // 默认：每格一个实体。current_size > 0 即拒绝
    if cell.current_size > 0 {
        // 例外 1：所有 occupant 和 incoming 都是同种群居
        if all_same_flock(cell, incoming) && cell.current_size < incoming.flock_max as u8 {
            return Allowed;
        }
        return Denied;
    }
    
    Allowed
}
```

移除 `max_size` / `density` 概念——不再用"3 只/格"这种虚构容量。

---

## P0：捕猎/进食改为相邻格操作

### 捕猎

`tick_reactive` — `try_consume` 函数。当前逻辑要求捕食者走到猎物格上。改为：

```rust
// 捕食者 (x,y) 在猎物相邻格时即可捕杀
// chebyshev_distance(px, py, tx, ty) == 1 → 可以攻击
```

### 进食

`try_eat_grass` — 当前 `x == gx && y == gy` 才吃。改为 `chebyshev_distance(x,y,gx,gy) == 1`。

---

## P0：CellComposition 简化

`CellSlot` 不再需要 `max_size`：

```rust
pub struct CellSlot {
    pub medium: Medium,
    pub current_count: u8,   // 该格实体数
    pub is_flock: bool,       // 该格是否全是同种群居
    pub flock_type: String,   // 群居类型名
}
```

---

## P0：尸体单元格允许共存

实体和尸体可在同一格。在 compose 中加尸体例外：

```rust
if cell.has_only_corpses() {
    return Allowed;
}
```

---

## P1：CellComposition 默认 max 改为 1

`src/axioms/composition.rs` `default_max_size` 函数移除，或用 1。

---

## P1：烟测试 carry 适配

smoke test 中实体数范围可能需要微调——一格一个后密度更低。

---

## 涉及文件

| 文件 | 改动 |
|---|---|
| `src/axioms/laws.rs` | compose 重写：默认拒绝，例外白名单 |
| `src/axioms/composition.rs` | CellSlot 简化为 current_count，移除 max_size |
| `src/axioms/profile.rs` | compose 签名可能变 |
| `src/world_state.rs` | move_entity / spawn 适配 |
| `src/systems/tick_reactive.rs` | try_consume 改为相邻攻击 |
| `src/systems/tick_herbivore.rs` | try_eat_grass 改为相邻进食 |
| `src/smoke_test.rs` | 实体数阈值可能需微调 |

## 验收

1. `cargo test --release` 全部 PASS
2. `cargo run --release -- --smoke-test` SMOKE: PASS
3. 启动游戏：任何格最多 1 只非群体动物，群居同种可共存，狼在相邻格杀羊
