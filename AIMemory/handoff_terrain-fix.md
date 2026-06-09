# 地图修正

## 1. 水潭整体下移 1 格

`src/terrain_ecology.rs` 第 105 行：
```rust
self.pool_source = (GRID_WIDTH / 2, max_y as u8);
```
`max_y = GRID_HEIGHT - 2 = 22`。改成 `max_y - 1 = 21`，即水潭中心从 y=22 移到 y=21（整体上移 1 格）。

如果"往下"是指屏幕往下（y 增大），那就 `max_y + 1 = 23`。先试 `max_y - 1`，不对再改。

同时修改对应测试 `pool_source_bottom_center` 的断言值。

## 2. 边界地格全部放山卡

地图最外一圈（x=0, x=GRID_WIDTH-1, y=0, y=GRID_HEIGHT-1）已经是 `edge` 地形。在 `src/initial_spawn.rs` 中遍历这圈格子，每个 spawn 一张 `mountain` 卡：

```rust
// 在 spawn_initial_world 中加
for x in 0..GRID_WIDTH {
    w.spawn("mountain", x, 0);
    w.spawn("mountain", x, GRID_HEIGHT - 1);
}
for y in 1..GRID_HEIGHT - 1 {
    w.spawn("mountain", 0, y);
    w.spawn("mountain", GRID_WIDTH - 1, y);
}
```

## 验收

启动游戏 → 水潭位置下移，地图四边全是山。
