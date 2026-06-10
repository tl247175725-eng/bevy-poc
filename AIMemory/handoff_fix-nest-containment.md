# 鸟巢容纳关系修复

**From**: deepseek-v4
**To**: cursor
**Date**: 2026-06-10
**Priority**: P0（鸟巢在树上不是容纳关系——一格一卡规则下需要改）

## 架构计划
复用现有容纳系统：`host_tree_id` + `in_tree = true`。鸟巢实体依附于树，不占格子，通过树的详情面板查看。

## 架构反馈
容纳系统（in_tree/in_pool/in_ground/host_tree_id）已在 spawn 逻辑中，只需在初始生成时正确设置。

## 智能验收
- 断言：`birdNest` 实体的 `host_tree_id` 不为 None
- 断言：`birdNest` 不与 tree 抢占同一格（`in_tree = true` 意味着不占格）
- 断言：点击 tree 可以看到鸟巢的容纳详情

---

## 修复

`src/initial_spawn.rs` 中 `spawn_initial_world`：

当前：
```rust
let tree1 = w.spawn("tree", px - 10, py);
w.spawn("birdNest", px - 10, py);
```

改为（参考 `spawn_tree_flora` 方式）：
```rust
let tree1 = w.spawn("tree", px - 10, py);
let nest_id = w.spawn("birdNest", px - 10, py);
if let Some(e) = w.entities.get_mut(&nest_id) {
    e.in_tree = true;
    e.host_tree_id = Some(tree1);
}
```

## 涉及文件

| 文件 | 改动 |
|---|---|
| `src/initial_spawn.rs` | spawn birdNest 时设置 in_tree + host_tree_id |

## 验收
- `cargo check` 0 错误
- `cargo test --release` 全 PASS
- `cargo run --release -- --smoke-test` SMOKE: PASS
- 启动游戏 → 鸟巢不显示为独立卡牌，点树可看到鸟巢容纳信息
