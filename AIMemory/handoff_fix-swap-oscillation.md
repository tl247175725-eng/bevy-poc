# 修复交换震荡 + 根除斜走

**Priority**: P0

## 根因
1. yield/shove 没检查阻挡者是否可移动——树被挤开
2. yield 太激进——闲逛动物也让路，导致连锁互换震荡
3. yield/shove 后阻挡者自己的 tick 又走一步 → 连续位移看起来像斜走

## 修复

### 1. 阻挡者移动性检查
`try_yield_and_enter` 和 `try_shove` 开头加：

```rust
// 不可移动的实体绝不推开
let Some(blocker_def) = world.card_defs.get(&world.entities[&blocker_id].type_name) else { return false; };
if crate::world_rules::card_has_tag(blocker_def, "rooted")
    || blocker_def.is_rooted
    || world.entities[&blocker_id].in_tree
    || world.entities[&blocker_id].in_pool
{
    return false;
}
```

### 2. yield 触发条件收紧
只在以下情况触发 yield：
- mover 状态 = Fleeing（逃命）且 blocker 状态 = Idle/Wandering
- mover 状态 = Hunting（猎食）且 blocker 状态 = Idle，且 blocker 是猎物同类

**删除**：SeekingFood 触发 yield（觅食动物不该挤开其他动物）

### 3. yield/shove 后设置标记防止回弹
yield 成功后给阻挡者设 `ecology_state = Idle` 并清除 `needs_grazing_tick`，防止同一 tick 内再参与移动。

## 验收
- 树、山不可被挤开
- 无连锁互换震荡
- 游戏中无斜走
