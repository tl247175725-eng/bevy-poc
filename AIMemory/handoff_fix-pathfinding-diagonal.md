# 修复曼哈顿：路径结果加过滤

**Priority**: P0

## 根因
`movement.rs` 第 206-213 行：`find_path` 返回的路径第一步未经过 `manhattan_step`，可能产生斜线移动。

## 修复
```rust
// 修改前（第 208-209 行）
let pdx = nx as i16 - x as i16;
let pdy = ny as i16 - y as i16;

// 修改后
let pdx = nx as i16 - x as i16;
let pdy = ny as i16 - y as i16;
let (pdx, pdy) = manhattan_step(pdx, pdy, world.rng_coin_for(id.0 ^ 0x5A5A));
```

## 验收
- 游戏中不再出现斜线移动
