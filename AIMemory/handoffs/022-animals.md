# Handoff 022 — 动物模型 + 底座卡牌棋子

## 架构计划

**改什么：** 新建 `src/render/animals.rs` + 修改 `mod.rs`（2 文件）
**做什么：** 13 种动物 mesh + 黑色圆柱底座 + 顶点色条纹/斑点

### 动物模型（参考 Low World Forest Animals Kit 面数级别）

每种动物 = 躯体(拉长椭球) + 头(稍小球) + 4腿(柱体) + 尾 + 特征部件
全部合并为单个 Mesh。纯顶点色。

```rust
// 中国亚热带 + 失落世界动物群
fn generate_tiger_mesh()     -> Mesh { /* 躯干+头+腿+尾+耳。~1200面。橙底+噪声黑条纹 */ }
fn generate_leopard_mesh()   -> Mesh { /* 参考虎但稍小。~1000面。黄底+噪声黑斑 */ }
fn generate_dhole_mesh()     -> Mesh { /* 豺——如小狼。~800面。棕红色 */ }
fn generate_rhino_mesh()     -> Mesh { /* 粗躯干+短粗腿+尖角。~800面。灰 */ }
fn generate_stegodon_mesh()  -> Mesh { /* 大象体型+弯曲长牙。~1500面。灰棕 */ }
fn generate_tapir_mesh()     -> Mesh { /* 粗躯干+短腿+短鼻。~700面。黑白双色 */ }
fn generate_deer_mesh()      -> Mesh { /* 细躯干+长腿+角。~1100面。棕+白腹 */ }
fn generate_boar_mesh()      -> Mesh { /* 粗躯干+短腿+獠牙。~800面。深棕 */ }
fn generate_bear_mesh()      -> Mesh { /* 粗躯干+粗腿。~1000面。黑 */ }
fn generate_rabbit_mesh()    -> Mesh { /* 小球+长耳+短尾。~400面。棕灰 */ }
fn generate_crocodile_mesh() -> Mesh { /* 长扁躯干+短腿+长尾。~700面。绿灰 */ }
fn generate_monkey_mesh()    -> Mesh { /* 小躯干+长臂+尾。~500面。棕 */ }
fn generate_peacock_mesh()   -> Mesh { /* 鸟身+扇形尾羽。~600面。蓝绿 */ }
```

### 底座

```rust
/// 黑色圆柱底座。高度 0.15，半径 0.25
fn generate_base_mesh() -> Mesh {
    // 简单圆柱：顶面+底面+侧面
    // 侧面顶点色：黑色 [0.1, 0.1, 0.1]
    // 顶面：稍浅灰——刻数字的位置
}
```

### 条纹/斑点——Perlin 噪声顶点色

```rust
/// 简化 Perlin 噪声（hash→平滑）
fn simple_perlin(x: f32, y: f32, seed: u32) -> f32 { /* 3D 噪声采样 */ }

/// 虎纹：在橙色底上，噪声 > 0.5 的区域顶点色加深
fn apply_tiger_stripes(mesh: &mut Mesh, seed: u32) {
    // 取每个顶点位置 → perlin(x*4, y*4, seed) → >0.5 → 颜色 ×0.3
}
/// 鹿腹白斑：perlin(x*2, y*8) > 0.6 → 偏白
/// 豹斑：perlin(x*6, y*6) > 0.65 → 黑
```

## 架构反馈

- 参考 Low World Forest Animals 面数级别(400-2100面) ✅
- 纯顶点色+Perlin噪声驱动条纹 ✅
- 底座统一黑色圆柱 ✅
- 每种动物独立 mesh → GPU 实例化（同种群 = 1 draw call）✅

## 智能验收

- `cargo check` 零错误
- `cargo test` 全 PASS
- 每个生成函数返回非空 mesh
