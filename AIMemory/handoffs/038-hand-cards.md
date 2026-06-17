# Handoff 038 — 手牌系统框架：五种操作 + 时间跳过快速 tick

## 三柱强制检查

| 柱子 | 用哪个 |
|---|---|
| 标签 | 不涉及新标签 |
| 元数值 | `TICKS_PER_DAY`（快速 tick 循环）、`HAND_SIZE_MAX`（新增，手牌上限） |
| 元动作 | Strike / Combine / Separate / Release（映射到手牌操作） |
| 公理 | 不涉及新公理——手牌操作复用已有的 Strike/Consume/Combine 公理 |

## 架构计划

**改什么：** 新建 `src/hand_cards.rs` + 新建 `src/systems/fast_tick.rs` + 改 `src/lib.rs`（3 文件）

**为什么：** 建立玩家手牌交互框架。五种操作（砸/拿/抽/叠/跳时间）的数据结构和执行骨架，具体规则后续填充。

### 设计决策（策划已确认）

- **五种手牌操作**：砸（Strike/加工）、拿（抓起放置）、抽（从叠中分离一张）、叠（放到格/卡上）、跳过时间（世界快进）
- **手牌 = 有限机会**：玩家持有有限数量的手牌，使用即消耗
- **框架不填规则**：不定义"砸石头出什么"、"叠什么组合出什么"，只做骨架
- **快速 tick = 简化版 main_tick**：跳过感知（最贵的 Phase 1），其他照跑

### 改动 1：新建 `src/hand_cards.rs`

```rust
//! 手牌系统——玩家干预世界的有限机会

use crate::spatial_index::EntityId;

/// 手牌类型
#[derive(Debug, Clone)]
pub enum HandCard {
    /// 砸——对目标施加一次力（Strike）或一次加工（Alter）
    /// 前提：手上拿着正确工具（或空手）+ 目标实体
    Strike,

    /// 拿——抓起一张卡到空中，选择放置位置
    PickUp,

    /// 抽——从叠加的多单元卡中分离一张（Separate）
    Separate,

    /// 叠——把手中卡放到格子或另一张卡上（Combine）
    Combine,

    /// 跳过时间——世界快进 N 天
    TimeSkip { days: u32 },
}

/// 玩家手牌槽
#[derive(Debug, Clone)]
pub struct PlayerHand {
    pub cards: Vec<HandCard>,
    pub max_size: usize,
}

impl PlayerHand {
    pub fn new(max_size: usize) -> Self {
        Self { cards: Vec::new(), max_size }
    }

    /// 添加手牌（满了返回 false）
    pub fn add(&mut self, card: HandCard) -> bool {
        if self.cards.len() >= self.max_size { return false; }
        self.cards.push(card);
        true
    }

    /// 使用第 index 张手牌（消耗）
    pub fn use_card(&mut self, index: usize) -> Option<HandCard> {
        if index < self.cards.len() {
            Some(self.cards.remove(index))
        } else {
            None
        }
    }

    pub fn is_full(&self) -> bool { self.cards.len() >= self.max_size }
    pub fn is_empty(&self) -> bool { self.cards.is_empty() }
    pub fn count(&self) -> usize { self.cards.len() }
}

/// 手牌操作执行结果
#[derive(Debug)]
pub enum HandCardResult {
    /// 操作成功
    Success,
    /// 被公理阻止
    Blocked { reason: String },
    /// 无效操作
    Invalid,
    /// 时间跳过完成
    TimeSkipComplete { days_skipped: u32, events_count: usize },
}
```

### 改动 2：新建 `src/systems/fast_tick.rs`

```rust
//! 快速 tick——时间跳过用的简化版 main_tick
//! 跳过 Phase 1（感知），其他阶段照跑

use crate::world_state::WorldState;
use crate::meta_values::{TICKS_PER_DAY, TICK_SECONDS};

/// 快速 tick：跳过感知，只做需求衰减+决策+执行+应用
pub fn fast_tick(world: &mut WorldState) {
    world.tick_delta = TICK_SECONDS;
    world.tick_count += 1;
    world.elapsed += TICK_SECONDS;

    // 跳过 Phase 1 感知（最贵）

    // Phase 2: 需求衰减
    for entity in world.entities.values_mut() {
        for need in &mut entity.needs {
            crate::need_match::activation::tick_need(need, TICK_SECONDS);
        }
    }

    // Phase 3: 安全阻断
    for entity in world.entities.values_mut() {
        crate::need_match::activation::apply_safety_block(&mut entity.needs);
    }

    // Phase 4-6: 决策+执行+应用（复用 main_tick 的逻辑）
    // 简化：不重建环境，用空环境——动物在快进期间"凭记忆行动"
    let entity_ids: Vec<crate::spatial_index::EntityId> = world.entities.keys().copied().collect();
    for &eid in &entity_ids {
        let Some(entity) = world.entities.get_mut(&eid) else { continue; };
        if entity.execution.intention.is_none()
            || crate::need_match::execution::is_plan_failed(&entity.execution)
        {
            if let Some(action) = crate::need_match::engine::tick_need_engine(
                &mut entity.needs,
                &mut entity.execution,
                &entity.knowledge,
                &[],  // 空环境——快进时不做感知搜索
                TICK_SECONDS,
                (entity.x, entity.y),
            ) {
                world.pending_actions.push((entity.id, action));
            }
        }
    }

    // 应用
    let pending = std::mem::take(&mut world.pending_actions);
    for (entity_id, action) in pending {
        crate::systems::main_tick::apply_meta_action_public(world, entity_id, action);
    }
}

/// 执行时间跳过：快进 N 天
pub fn execute_time_skip(world: &mut WorldState, days: u32) -> usize {
    let total_ticks = days as u64 * TICKS_PER_DAY;
    let mut event_count = 0usize;

    for _ in 0..total_ticks {
        fast_tick(world);
        event_count += world.drain_pending_events().len();
    }

    event_count
}
```

### 改动 3：`src/lib.rs` 注册模块

```rust
pub mod hand_cards;
```

### 注意

`apply_meta_action` 在 `main_tick.rs` 中是私有函数。快速 tick 需要调用它。两个选择：
1. 提取为 `pub fn apply_meta_action_public`
2. 或者把 fast_tick 直接放在 main_tick.rs 里

**选方案 1**——在 main_tick.rs 加一个公开包装函数 `pub fn apply_meta_action_public(world, entity_id, action)`。

## 本体变更

- [ ] 无新标签/元数值/公理变更——纯框架

## 架构反馈

1. **框架和内容分离**：手牌骨架不含具体规则。加工配方、叠加组合等后续填充
2. **快速 tick 自动覆盖未来系统**：任何挂到 main_tick 的新系统（经济、社交等）都走同一个 tick 循环——快速 tick 自动包含
3. **手牌数量是有限资源**：实现了 PlayerHand 的 max_size 限制和 use_card 消耗逻辑

## 智能验收

- [ ] `cargo check` 零错误
- [ ] `cargo test` 全 PASS
- [ ] `hand_cards.rs` 编译通过，HandCard 枚举包含五种操作
- [ ] `fast_tick.rs` 编译通过，`execute_time_skip` 可被调用
- [ ] 新增测试：PlayerHand 的 add/use_card/is_full 基本操作
- [ ] 新增测试：execute_time_skip 跳过 1 天后 tick_count 增加 TICKS_PER_DAY
