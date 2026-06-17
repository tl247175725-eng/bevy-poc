# Handoff 019 — 植被模型

## 架构计划

**改什么：** 新建 `src/render/vegetation.rs`，修改 `src/render/mod.rs`（2 文件）
**做什么：** 每种植物一个 mesh 生成函数，纯 Bevy 0.15，纯顶点色

### 树

```rust
/// 楠木——半球冠 + 直干。冠半径 2.5m，干高 6m。~40 面。
fn generate_nanmu_mesh() -> Mesh { /* 半球 + 圆柱 */ }

/// 樟树——扁椭球冠 + 粗干 + 外展枝。冠宽 7m × 高 4m。~50 面。
fn generate_camphor_mesh() -> Mesh { /* 扁椭球 + 圆柱 + 4 根短枝 */ }

/// 马尾松——尖锥冠 + 直干。锥底 4m，高 8m。~30 面。
fn generate_pine_mesh() -> Mesh { /* 尖锥 + 圆柱 */ }

/// 毛竹——细柱节段 + 顶弯垂带小叶。高 10m。~20 面。
fn generate_bamboo_mesh() -> Mesh { /* 节段圆柱 + 弯头 + 小叶三角 */ }
```

### 灌木

```rust
/// 杜鹃——3-5 个扁球叠堆。~20 面。
fn generate_azalea_mesh() -> Mesh { /* 小球簇 */ }
```

### 草/芦苇

```rust
/// 芦苇——细柱(高3m) + 顶椭球穗。~12 面。
fn generate_reed_mesh() -> Mesh { /* 细柱 + 小椭球顶 */ }

/// 芒草——细三角柱集群(高5m)，每株 6 面。
fn generate_miscanthus_mesh() -> Mesh { /* 三角面片簇 */ }
```

### 水生

```rust
/// 莲花——扁圆盘 + 中心小凸起。~10 面。
fn generate_lotus_mesh() -> Mesh { /* 扁平八边盘 + 半球心 */ }

/// 水草——弯曲线条带。~8 面。
fn generate_waterweed_mesh() -> Mesh { /* 波浪形扁带 */ }
```

### 关键参数

```rust
// 树干颜色
const BARK_NANMU: [f32; 3] = [0.4, 0.35, 0.28];    // 浅灰棕
const BARK_CAMPHOR: [f32; 3] = [0.22, 0.18, 0.14];   // 深黑褐
const BARK_PINE: [f32; 3] = [0.35, 0.20, 0.15];       // 红棕
const BAMBOO_COLOR: [f32; 3] = [0.55, 0.65, 0.35];    // 黄绿

// 叶色
const LEAF_BROADLEAF: [f32; 3] = [0.18, 0.35, 0.12];  // 深绿
const LEAF_PINE: [f32; 3] = [0.10, 0.25, 0.10];        // 暗绿
const LEAF_BAMBOO: [f32; 3] = [0.35, 0.50, 0.20];      // 黄绿
const LEAF_FLOWER: [f32; 3] = [0.75, 0.40, 0.35];      // 杜鹃花-粉红
```

## 架构反馈

- 同构真实形态特征+商业做法参考 ✅
- 纯代码生成，无外部模型 ✅
- 顶点色区分物种 ✅
- 每个函数返回单个 Mesh，后续按 card_def 的 quantity 做 GPU instancing ✅

## 智能验收

- `cargo check` 零错误
- `cargo test` 全 PASS
- 每个函数生成非空 mesh
