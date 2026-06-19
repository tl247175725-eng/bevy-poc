# 十二、工程实战笔记

> Bevy 0.15 开发中踩过的坑和验证过的方案。每次重大技术决策收束后追加。

---

## 1. WGSL 自定义材质管线

### ✅ 正确做法

- **Uniform 必须打包**：多个字段不能各自 `#[uniform(0)]`，必须放进一个 `#[derive(ShaderType)]` 结构体
- **文件加载 shader 用 `@group(2)`**：`#{MATERIAL_BIND_GROUP}` 宏只在嵌入 shader 时有效，文件加载的 shader 用硬编码 `@group(2) @binding(0)`
- **Material struct 最小化**：`#[uniform(0)] uniforms: MyUniforms`，其余交由 `Material` trait 处理
- **调试口诀**：如果材质渲染为透明/品红 → 看终端 `ERROR bevy_render::render_resource::pipeline_cache`——它会精确指出 WGSL 第几行出错

### ❌ 已知坑

- GLB 文件通过 `SceneRoot` 加载在 Bevy 0.15 存在兼容问题（透明模型）
- `@import bevy_pbr::mesh_view_bindings::view` → `view.world_position` 访问相机位置有效

---

## 2. 3D 轨道相机

### ✅ 正确做法

**三层解耦**：旋转(yaw/pitch/radius)、缩放(radius clamp)、平移(focus offset)各自独立计算，最后统一应用 Transform。

- 球坐标 → 笛卡尔：`pos = focus + Vec3(r*cos(pitch)*sin(yaw), r*sin(pitch), r*cos(pitch)*cos(yaw))`
- 天空球边界：作为最后一层独立检查，不混入旋转/缩放/平移逻辑
- 软刹车：`lerp` 过渡替代 `clamp` 硬截断

### ❌ 已知坑

- 射线投射实现"缩放到光标"引入 bug（焦点飞出天空球）→ 保持简单的焦点补偿即可
- 硬截断导致视角抽搐 → 用投影+平滑过渡

---

## 3. 天空着色

### ✅ 正确做法

**三重预设平滑混合**（CPU 端，48×96 顶点球体）：
```
白天预设（天顶深蓝/地平线浅蓝）
晚霞预设（天顶紫蓝/地平线橙红）
夜晚预设（天顶极深蓝/地平线暗蓝）
→ sun_elevation 作为 smoothstep 权重混合
→ view_dir.y 作为空间插值（天顶↔地平线）
```

- CPU 端顶点色方案稳定可靠，无 shader 编译风险
- smoothstep 是图形学灵魂——生硬的"是/否"变成平滑渐变

---

## 4. 程序化云层

### ✅ 正确做法

- **Simplex 2D 梯度噪声**（非 hash 噪声）→ 相邻像素平滑连续
- 高度遮罩：`smoothstep(0.2, 0.4, view.y) * (1.0 - smoothstep(0.7, 0.85, view.y))` → 云只在中高空
- 时间偏移：`view_dir.xz + time * 0.01` → 缓缓飘移
- 密度阈值随太阳高度变化：`threshold = 0.35 - sun_elev * 0.1` → 夜间消散

### ❌ 已知坑

- Hash 噪声（`fract(sin(x)*big)`）产生"雪花/方块"→ 只适合离散效果，不适合云
- 不限制高度 → 云糊满全球
- 不用 smoothstep → 边缘像贴纸

---

## 5. 太阳/月亮 3D 实体

### ✅ 正确做法

- **程序化球体 + 自定义 WGSL 材质** > GLB 外部模型（Bevy 0.15 兼容性）
- 太阳：Flat Shading（dpdx/dpdy 计算面法线）+ Fresnel 颜色渐变 + emissive_intensity 驱动 Bloom
- 月亮：3D FBM 噪声模拟坑洼 + 边缘微光
- **Bloom 后处理**替代假光晕球：摄像机 `hdr:true` + `BloomSettings::default()`

### ❌ 已知坑

- 大透明球体做光晕 → 硬边缘穿帮 → 删除，纯靠 Bloom
- 多个 halo 实体用 `get_single_mut()` 限制 → 改用 `iter_mut()`

---

## 6. 星星

### ✅ 正确做法

- 独立的 `PointList` mesh：600个 hash 随机位置白点，mesh 原点设为天空球心
- 可见性控制用 `Visibility::Visible/Hidden`（太阳低于地平线→可见）
- **不要用 `Transform::scale` 做淡入淡出**—— scale 从 0→1 会使点从中心"绽放开"

---

## 7. 渐进式开发

### ✅ 正确做法

- Step1 → Step2 → Step3 独立 binary 推进
- 每步只加一个可验收的视觉/功能变化
- 旧 binary 保留不删——可随时回退对比
- `cargo run --bin step3_skysun` 有明确的启动命令
