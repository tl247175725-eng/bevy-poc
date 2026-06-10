# Spatial Query 网格化优化

**From**: deepseek-v4
**To**: cursor
**Date**: 2026-06-11
**Priority**: P1（一劳永逸解决 spatial query 瓶颈）

## 架构计划
spatial_index 已有 positions + by_tag 两层索引。加第三层 grid_buckets：每格存该格内所有实体列表。query_near 不再扫全量 tag 集，只查当前格 + 周边 (range×2+1)²个格。

## 架构反馈
spatial_index 是 HashMap 组合，加网格层不改动现有索引结构。向后兼容。

## 智能验收
- `query_near` 查 100 次的平均耗时 < 当前 1/10
- smoke test PASS
- 所有现有测试 PASS

---

## 实现

### 1. spatial_index 加 grid_buckets

```rust
pub struct SpatialIndex {
    positions: HashMap<EntityId, (u8, u8)>,
    by_tag: HashMap<String, HashSet<EntityId>>,
    grid_buckets: [[Vec<EntityId>; GRID_WIDTH as usize]; GRID_HEIGHT as usize], // 新增
}
```

### 2. insert / remove / move_entity 同步维护 grid_buckets

```rust
fn insert(&mut self, entity: &IndexedEntity) {
    // ... 现有 by_tag + positions 逻辑 ...
    self.grid_buckets[entity.y as usize][entity.x as usize].push(entity.id);
}

fn remove(&mut self, id: EntityId) {
    if let Some(&(x, y)) = self.positions.get(&id) {
        self.grid_buckets[y as usize][x as usize].retain(|&eid| eid != id);
    }
    // ... 现有逻辑 ...
}
```

### 3. query_near 改为网格查

```rust
pub fn query_near(&self, x: u8, y: u8, tag: &str, radius: u8) -> Vec<EntityId> {
    let tag_set = match self.by_tag.get(tag) {
        Some(s) => s,
        None => return vec![],
    };
    let mut result = Vec::new();
    
    let min_x = x.saturating_sub(radius);
    let max_x = (x + radius).min(GRID_WIDTH - 1);
    let min_y = y.saturating_sub(radius);
    let max_y = (y + radius).min(GRID_HEIGHT - 1);
    
    for gy in min_y..=max_y {
        for gx in min_x..=max_x {
            for &id in &self.grid_buckets[gy as usize][gx as usize] {
                if tag_set.contains(&id) {
                    if let Some(&(ex, ey)) = self.positions.get(&id) {
                        if x.abs_diff(ex).max(y.abs_diff(ey)) <= radius {
                            result.push(id);
                        }
                    }
                }
            }
        }
    }
    result
}
```

## 涉及文件

| 文件 | 改动 |
|---|---|
| `src/spatial_index.rs` | grid_buckets 字段，query_near 重写，insert/remove/move 维护 |

## 验收
- `cargo check` 0 错误
- `cargo test --release` 全 PASS
- `cargo run --release -- --smoke-test` PASS
