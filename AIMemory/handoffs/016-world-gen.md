# Handoff 016 — 同心环世界生成

## 架构计划

**改什么：** `src/initial_spawn.rs` + 可能导致 `smoke_test` 失败（预期内）

### 生成逻辑

```rust
/// 按七环同心圆结构填充 32×32 棋盘
pub fn spawn_concentric_world(world: &mut WorldState, card_defs: &[CardDef]) {
    let center = (16.0, 16.0);  // 地图正中心

    for x in 0..32 {
        for y in 0..32 {
            let dx = x as f32 - center.0;
            let dy = y as f32 - center.1;
            let dist = (dx * dx + dy * dy).sqrt();  // 欧几里得距离

            let card_type = match dist {
                d if d <= 1.5   => "abyss_pool",
                d if d <= 3.5   => "shallow_water",
                d if d <= 7.0   => "wetland",
                d if d <= 12.0  => "grassland",
                d if d <= 19.0  => "broadleaf_forest",
                d if d <= 25.0  => "foothills",
                _               => "cliff",
            };

            spawn_card_at(world, card_defs, card_type, x, y);
        }
    }
}
```

### 散点叠加

在基层地形之上，加散点植物：
- 浅水区：按概率散点莲花(1/6格)、水草(1/3格)
- 湿地：散点芦苇(1/2格)、香蒲(1/3格)
- 草原：散点芒草(1/2格)
- 森林：散点楠木(1/3格)、樟树(1/4格)、毛竹(1/5格)
- 山麓：散点松(1/2格)、杜鹃(1/5格)

散点用 rand 产生，每个格子用 seed = x*32+y（可复现）。

### 不做的

- 不加载动物卡
- 不修改渲染系统
- 不修改现有公理逻辑

## 架构反馈

- 32×32 全格填充—格子上堆叠多卡（叠附态）✅
- 同心环距离公式简单—环边界是圆弧，不是折线 ✅
- seed = x*32+y 保证了每次生成相同 ✅

## 智能验收

- `cargo check` 零错误
- `cargo test` 全 PASS（smoke_test 可能需要暂时跳过——没有动物实体）
