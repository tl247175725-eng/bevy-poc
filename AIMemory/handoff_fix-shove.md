# 紧急修复：挤过踢飞卡牌

**Priority**: P0（石、人、尸体全被踢飞到地图边缘）

## 根因

`movement.rs` `try_shove` 第 74-89 行：查找推送方向上**最远**的空位。阻者被一路推到边界，穿过整个地图。

## 修复

改为只推**最近**的一个空位：从 `gx+step_dx, gy+step_dy` 开始，第一个空位就停下。

```rust
// 修改后：只推一格
let push_x = gx as i16 + step_dx;
let push_y = gy as i16 + step_dy;
if push_x >= 0 && push_y >= 0 
    && push_x < GRID_WIDTH as i16 && push_y < GRID_HEIGHT as i16 
{
    let ux = push_x as u8;
    let uy = push_y as u8;
    if world.cell_composition.slot(ux, uy).living_count == 0
        && !is_blocked_for(world, ux, uy, Some(blocker_id.0))
    {
        world.move_entity(blocker_id, ux, uy) == MoveResult::Moved
            && world.move_entity(mover_id, gx, gy) == MoveResult::Moved
    }
}
```

## 验收
- 石不被踢飞到地图边缘
- 尸体不被踢飞
- 人卡不被踢飞
- 挤过仍正常工作（推一格放行）
