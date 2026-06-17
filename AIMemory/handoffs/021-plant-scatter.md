# Handoff 021 — 植物散落 + 实例化渲染

## 架构计划

**改什么：** 新建 `src/render/scatter.rs`，修改 `src/render/mod.rs`（2 文件）
**依据：** 世界生成（016）已按概率把植物撒到各格。现在把数据→画面。
**商业参考：** FrostyGrass GPU instancing 模式

### 核心逻辑

```rust
/// 扫描 32×32 格，为每格的植物创建渲染实体
pub fn scatter_vegetation(
    world: &WorldState,
    card_defs: &[CardDef],
    meshes: &HashMap<String, Handle<Mesh>>, // 预生成的植物 mesh
    commands: &mut Commands,
) {
    for x in 0..32 {
        for y in 0..32 {
            let cell = world.get_cell(x, y, 0); // z=0 地表
            for card in cell.stack() {
                match card.type_name.as_str() {
                    "nanmu_tree" => spawn_instances(x, y, card.quantity, "nanmu", commands),
                    "camphor_tree" => spawn_instances(x, y, card.quantity, "camphor", commands),
                    "pine_forest" => spawn_instances(x, y, card.quantity, "pine", commands),
                    "bamboo" => spawn_instances(x, y, card.quantity, "bamboo", commands),
                    "reed" => spawn_instances(x, y, card.quantity, "reed", commands),
                    "miscanthus" => spawn_instances(x, y, card.quantity, "miscanthus", commands),
                    "lotus" => spawn_instances(x, y, card.quantity, "lotus", commands),
                    "azalea" => spawn_instances(x, y, card.quantity, "azalea", commands),
                    _ => {}
                }
            }
        }
    }
}

/// 在指定格内撒 quantity 个植物实例
/// 每格内随机偏移位置（避免排成直线）
fn spawn_instances(x: u8, y: u8, quantity: u32, plant_type: &str, commands: &mut Commands) {
    let mesh_handle = get_mesh_for(plant_type);
    let material = get_material_for(plant_type);
    for i in 0..quantity.min(MAX_PER_CELL) {
        let (ox, oy) = random_offset(x, y, i); // 格内随机偏移
        let scale = 0.8 + rand_scale(i);         // 微变大小
        commands.spawn((
            Mesh3d(mesh_handle.clone()),
            MeshMaterial3d(material.clone()),
            Transform::from_xyz(ox, oy, get_z_height(x, y)).with_scale(Vec3::splat(scale)),
            VegetationTag,
        ));
    }
}
```

### 关键参数

```rust
const MAX_PER_CELL: u32 = 50; // 每格每种植物最多 50 实例
const CELL_SIZE: f32 = 1.0;    // 一格 = 1.0 渲染单位
```

### GPU 实例化——Bevy 0.15 自动

所有同类型植物共享同一个 `Handle<Mesh>` + `Handle<StandardMaterial>` → Bevy 0.15 的 GPU-driven rendering 自动将它们合并为一次 GPU 调用。

### 性能预估

3000-5000 植物实例 × 共享 9 种 mesh = 9 次 draw calls。GPU < 1ms。

## 架构反馈

- 数据来源：`WorldState` 的卡片堆（已有数据）✅
- 同类 mesh 共享 → Bevy 0.15 GPU 自动实例化 ✅
- 格内随机偏移 → 避免网格感 ✅

## 智能验收

- `cargo check` 零错误
- `cargo test` 全 PASS
