# 激活玩家 AI

**Priority**: P0（玩家卡完全不行动——tick_brain 从未被调用）

## 根因
`tick_player_in_sim` 函数存在但无调用者。PlayerPlugin 只注册了 UI，没注册 AI 逻辑。

## 修复
在 `main_tick` 中调用玩家 AI：

```rust
// src/systems/main_tick.rs 末尾
crate::player::tick_player_in_sim(world_sim, &mut interaction, &mut events, delta);
```

或直接在 world_state 的 tick_once 中调用：
```rust
crate::player::needs_manager::tick_player(&mut self, player_id, delta, ...);
```

## 验收
- 人卡开始行动——觅食、躲避、休息
