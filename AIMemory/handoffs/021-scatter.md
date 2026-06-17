# Handoff 021 — 植物散落 + GPU 实例化

## 架构计划

**改什么：** 新建 `src/render/scatter.rs`，修改 `src/render/mod.rs`（2 文件）
**做什么：** 读棋盘数据 → 泊松盘采样散落植物 → GPU 实例化渲染

### 核心逻辑

```rust
/// 泊松盘采样——在 1.0×1.0 单位格内生成自然分布的点
/// seed: 从格子坐标派生，保证每次生成相同结果
/// density: 每格几个点
fn poisson_disk_sample(seed: u64, density: u32) -> Vec<(f32, f32)> {
    // 标准泊松盘算法：
    // 1. 生成候选点（随机位置）
    // 2. 检查每个候选点与已接受点的最小距离 min_dist = 1.0 / density.sqrt()
    // 3. 满足距离要求的放入结果集
    // 4. 直到格内无法再塞入新点或达到 density 上限
}

/// 依据卡的类型和 quantity 决定散落参数
fn scatter_params(card_type: &str, quantity: u32) -> ScatterConfig {
    match card_type {
        "nanmu_tree"     => ScatterConfig { density: density_from_qty(quantity, 0, 30, 60, 1, 4, 8), scale_range: (0.85, 1.15) },
        "camphor_tree"   => ScatterConfig { density: density_from_qty(quantity, 0, 25, 50, 1, 3, 6), scale_range: (0.88, 1.12) },
        "pine_forest"    => ScatterConfig { density: density_from_qty(quantity, 0, 20, 40, 1, 3, 5), scale_range: (0.80, 1.10) },
        "bamboo"         => ScatterConfig { density: density_from_qty(quantity, 0, 20, 40, 1, 2, 4), scale_range: (0.90, 1.10) },
        "reed"           => ScatterConfig { density: density_from_qty(quantity, 0, 50, 100, 2, 6, 12), scale_range: (0.85, 1.15) },
        "miscanthus"     => ScatterConfig { density: density_from_qty(quantity, 0, 100, 200, 3, 8, 16), scale_range: (0.80, 1.20) },
        "lotus"          => ScatterConfig { density: density_from_qty(quantity, 0, 10, 20, 1, 2, 3), scale_range: (0.90, 1.10) },
        "waterweed"      => ScatterConfig { density: density_from_qty(quantity, 0, 50, 100, 2, 5, 10), scale_range: (0.85, 1.10) },
        "azalea"         => ScatterConfig { density: density_from_qty(quantity, 0, 15, 30, 1, 2, 4), scale_range: (0.85, 1.15) },
        _ => ScatterConfig { density: 0, scale_range: (1.0, 1.0) },
    }
}

/// quantity → density 映射：低(<t1)=>d1, 中(t1-t2)=>d2, 高(>t2)=>d3
fn density_from_qty(qty: u32, _lo: u32, t1: u32, t2: u32, d1: u32, d2: u32, d3: u32) -> u32 {
    if qty == 0 { 0 } else if qty < t1 { d1 } else if qty < t2 { d2 } else { d3 }
}

struct ScatterConfig {
    density: u32,
    scale_range: (f32, f32),
}

/// 遍历 32×32 格 → 找叠附植物卡 → 泊松采样 → 收集实例数据
fn collect_plant_instances(world: &WorldState, card_defs: &[CardDef]) -> HashMap<String, Vec<PlantInstance>> {
    // plant_type → [(x, y, z, scale, rotation)]
}

struct PlantInstance {
    x: f32, y: f32, z: f32,
    scale: f32,
    rotation_y: f32,
}

/// 每种植物→生成一个 instanced Entity（Bevy 自动 GPU 实例化）
fn spawn_plant_layer(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    plant_type: &str,
    instances: &[PlantInstance],
    base_mesh: Handle<Mesh>,
    color: [f32; 3],
)
```

### 关键参数

```rust
const POISSON_MIN_DIST: f32 = 0.15;  // 树之间最小间距（格内归一化坐标）
const PERLIN_SCALE: f32 = 4.0;        // 颜色噪声频率
```

### 颜色噪声微调

每棵树顶点色 = 基础色 × (1.0 + perlin_noise(x, y) × 0.1)
→ 同一片林子里有深有浅。不改 mesh，只改实例颜色。

## 架构反馈

- 泊松盘生成自然分布 ✅
- seed 来自格坐标→确定性，每次生成相同 ✅
- quantity → density 自动映射—后端驱动 ✅
- 每种植物 = 1 次 GPU instanced draw call（~9 次全图）✅
- 树被砍了→quantity 变→density 变→下帧实例数变 ✅

## 智能验收

- `cargo check` 零错误
- `cargo test` 全 PASS
