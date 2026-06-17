# Handoff 026 — 动物世界生成

## 架构计划

**改什么：** `src/initial_spawn.rs`（1 文件）
**做什么：** 在 `spawn_concentric_world()` 之后，按栖息地环散落动物

### 生成逻辑

每种动物按标签中的 `habitat:*` 自动匹配到对应环。按卡定义中的 `quantity` 控制总数，按栖息环面积比例分配。

```rust
pub fn spawn_animals(world: &mut WorldState, card_defs: &[CardDef]) {
    let animal_cards: Vec<&CardDef> = card_defs.iter()
        .filter(|d| d.tags.iter().any(|t| t == "animal"))
        .collect();

    for animal_def in &animal_cards {
        // 从标签中提取栖息地偏好
        let habitats = extract_habitats(&animal_def.tags); // "forest" / "grassland" / "aquatic"
        let total = animal_def.quantity as usize;

        // 按栖息环面积比例分配个体
        let ring_counts = distribute_by_ring(total, &habitats);
        
        for (ring, count) in ring_counts {
            for _ in 0..count {
                let (x, y) = random_cell_in_ring(ring);
                spawn_card_at(world, animal_def, x, y, 1); // 每格 1 只
            }
        }
    }
}
```

### 栖息环映射

```
habitat:aquatic    → 深潭(环1) + 浅水(环2)
habitat:wetland    → 湿地(环3)
habitat:grassland  → 草原(环4)
habitat:forest     → 森林(环5)
habitat:mountain   → 山麓(环6)
```

### 不做的

- 不修改已有植物撒落逻辑
- 每格每物种限 1 个实体（动物不能堆叠在同一个格上）
- 水生动物的 z 坐标在水下

## 智能验收

- `cargo check` 零错误
- `cargo test` 全 PASS
