# 移除群卡系统

**Priority**: P0（群卡系统导致无数 bug，现阶段无游戏性价值，砍掉。设计文档标记为待定）

---

## 需要移除的代码

### world_state.rs
- Entity 删除 `herd_count` 字段
- 删除 `spawn_herd` 函数

### tick_reactive.rs
- 删除 `try_merge_same_cell_herd`
- 删除 `try_join_adjacent_herd`  
- 删除 `scatter_herd`
- 删除 `try_form_new_herd`
- 删除所有调用这些函数的地方
- Flock drive 恢复为简单 move_toward 向同类平均位置

### card_visual.rs
- 删除 `GroupCardMarker` 组件
- 删除 `GroupCountText` 组件
- 删除 `GroupLabelText` 组件
- 删除 `flock_groups` 函数
- 删除 `spawn_group_card_visual` 函数
- 删除所有群卡渲染相关逻辑

### selection_info.rs
- 删除 `build_herd_panel` 函数
- 删除 `herd_identity_tags` 函数
- 删除 herd_count 检查

### axioms/profile.rs
- 删除 `SocialStructure` 相关字段（cohesion, separation等）或保留但标记未使用

### axioms/mod.rs
- 移除 herd 相关导出

### card_audit.rs / tag_zh.rs
- 移除 herd 相关标签注册

### card_defs.ron
- 移除所有卡牌的 `flock_*` 标签

---

## 必须保留

- `in_tree` 修复（只对 oak/pine/bamboo/bush）
- `in_pool` 修复（含 waterBug, landBug）
- `in_ground` 含 landBug
- 所有地形修改（barren 替代 edge）
- spawn 碰撞回避
- compose is_flock 逻辑保留（即使群卡砍掉，同种群居规则将来可能复用）

## 验收

- `cargo check` 0 错误
- `cargo run` 游戏启动正常
- 一格一卡，无堆叠
- 无群卡出现
