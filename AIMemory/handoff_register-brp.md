# 注册 BRP 自定义组件

**From**: deepseek-v4
**To**: cursor
**Date**: 2026-06-10
**Priority**: P1（让 AI 读游戏运行时状态——减少"你启动→描述→我猜"循环）

## 架构计划
BRP 已运行。注册自定义组件后 AI 可通过 curl 读取 `CardVisual`、`SimWorld` 等游戏特有数据。

## 架构反馈
无。

---

## 实现

### 1. 注册自定义组件类型

`src/plugins/mod.rs` 在 `.add_plugins()` 之前或之后：

```rust
use bevy::ecs::component::Component;

app.register_type::<crate::card_visual::CardVisual>();
app.register_type::<crate::card_visual::CardIconText>();
app.register_type::<crate::card_visual::CardNameText>();
app.register_type::<crate::card_visual::GroupCardMarker>();
app.register_type::<crate::render::move_animation::MoveAnimating>();
```

### 2. CardVisual 等结构体加 Reflect derive

```rust
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct CardVisual { ... }

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct CardIconText;

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct GroupCardMarker { ... }
```

### 3. 注册 SimWorld 资源

```rust
app.register_type::<crate::grid_render::SimWorld>();
```

`SimWorld` 包装了 `WorldState`，加 Reflect derive：

```rust
// grid_render.rs
#[derive(Resource, Reflect, Deref, DerefMut)]
#[reflect(Resource)]
pub struct SimWorld(pub WorldState);
```

但 `WorldState` 内部有 `HashMap` 等不支持 Reflect 的类型，所以 SimWorld 无法完全序列化。跳过这步——用更简单的方式：把 `tick_count`、`entities.len()` 等关键统计量暴露为 Bevy Resource。

### 4. 替代方案：暴露统计资源

新建 `SimStats`：

```rust
// sim_events.rs
#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct SimStats {
    pub entity_count: usize,
    pub tick_count: u64,
    pub herbivore_count: usize,
    pub predator_count: usize,
    pub deaths: u64,
}
```

在 `main_tick` 更新 SimStats。这样 AI 可以读到核心指标而不需要序列化整个 HashMap。

## 验收
- `cargo check` 0 错误
- 游戏运行时 `curl` 可读到 `CardVisual` 和 `SimStats`
