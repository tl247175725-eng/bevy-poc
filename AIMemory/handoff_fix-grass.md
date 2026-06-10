# 草再生修复：解决全图震荡

**Priority**: P0（全场只有 1 棵草，所有动物饿死震荡）

## 根因
`grass_regen.rs` 只在河岸格重生草，且 cap 太小。初始 10 棵草被 29 只食草动物秒吃光后无补充。

## 修复

### 1. 扩大草再生范围
不只河岸，所有陆地空格（terrain = land，无 entity 占用）定期概率再生草。

```rust
// 改为：所有陆地空格
for x in 1..GRID_WIDTH-1 {
    for y in 1..GRID_HEIGHT-1 {
        if count >= cap { break; }
        if terrain_at(world, x, y) == "land" 
           && world.entities_at(x, y).is_empty() 
        {
            world.spawn("grass", x, y);
        }
    }
}
```

### 2. 增大草上限
`LIVING_GRASS_CAP` 从当前值提高到 30+。

### 3. 腐殖土长草加速
humus → grass 时间从 `HUMUS_GRASS_SECONDS` 减半。

## 验收
- 游戏运行 30 秒后 grass > 10
- 动物不再全图震荡
