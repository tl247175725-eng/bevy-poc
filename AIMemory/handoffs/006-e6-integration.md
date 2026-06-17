# Handoff 006-e6 — 需求匹配引擎集成

## 架构计划

**改什么：** 新建 `src/need_match/engine.rs`，小幅修改 `src/systems/main_tick.rs`（2 文件）
**做什么：** 把 e1-e5 模块串联成可调用的管道

### engine.rs

```rust
use crate::need_match::activation::*;
use crate::need_match::data::*;
use crate::need_match::search::*;
use crate::need_match::execution::*;
use crate::need_match::social::*;

/// 每个 tick：更新需求 → 安全阻断 → 匹配 → 执行
pub fn tick_need_engine(
    needs: &mut [NeedState],
    state: &mut ExecutionState,
    knowledge: &KnowledgeGraph,
    environment: &[(u32, MaterialProperties)],
    delta: f32,
) -> Option<MetaAction> {
    // 1. 需求衰减 + 计算紧迫度
    for need in needs.iter_mut() {
        tick_need(need, delta);
    }
    
    // 2. 安全阻断
    apply_safety_block(needs);
    
    // 3. 如果没有当前计划 → Decide
    if state.intention.is_none() {
        let candidates = search_by_needs(needs, knowledge);
        if let Some(best) = arbitrate(candidates, needs, state.intention_score) {
            *state = build_plan(&best, knowledge);
        }
    }
    
    // 4. 执行下一步
    tick_execution(state, environment)
}
```

### main_tick.rs 修改

在 `batch_uniform_entity_updates` 之后加一行：

```rust
// 每个实体调一次需求引擎（暂时不接——等感知层就位后启用）
// for entity in world.entities.values_mut() {
//     if let Some(action) = tick_need_engine(&mut entity.needs, &mut entity.exec_state, ...) {
//         // 执行 action
//     }
// }
// 当前占位：编译通过即可，实际调用后续 handoff
```

### lib.rs

如果 engine.rs 引用了 MetaAction，需确认 `use crate::meta_actions::MetaAction;` 可访问。

## 架构反馈

**管道顺序 (三步)：**
1. `tick_need` + `apply_safety_block` — 每个 tick 必然执行
2. `search_by_needs` + `arbitrate` — 仅在无计划或计划失败时执行
3. `tick_execution` — 每个 tick 执行

**故意不做的：** 不挂接到 main_tick 的实际循环。挂接需要感知层（MaterialProperties 映射到真实世界实体）先就位。当前只证明模块可串联编译。

## 智能验收

- `cargo check` 零错误
- `cargo test` 全 PASS
- 测试：伪造需求+知识图+空环境 → tick_need_engine 返回 Pause（无匹配候补）
