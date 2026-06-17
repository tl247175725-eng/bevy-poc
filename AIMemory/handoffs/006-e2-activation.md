# Handoff 006-e2 — 需求激活逻辑

## 架构计划

**改什么：** `src/need_match/data.rs` + 新建 `src/need_match/activation.rs`（2 文件）
**依据：** `design-philosophy-v5.md` §3.2、§6.5 防振荡

### activation.rs

```rust
/// 需求激活阈值——紧迫度超过此值视为"激活"
pub const URGENCY_ACTIVATION_THRESHOLD: f32 = 0.3;

/// 安全阻断阈值——安全紧迫度超过此值阻断所有非安全需求
pub const SAFETY_BLOCK_THRESHOLD: f32 = 0.7;

/// 每个 tick 衰减需求，计算紧迫度
/// urgency = sigmoid((baseline - current) / baseline)
pub fn tick_need(need: &mut NeedState, delta: f32) {
    // 需求值向 1.0（完全匮乏）衰减
    need.current = (need.current + need.decay_rate * delta).min(1.0);
    // sigmoid 紧迫度
    let raw = (need.current - need.baseline) / need.baseline;
    need.urgency = 1.0 / (1.0 + (-raw * 5.0).exp());
}

/// 安全阻断逻辑
pub fn apply_safety_block(needs: &mut [NeedState]) {
    let safety_urgent = needs.iter()
        .any(|n| n.kind == NeedKind::Safety && n.urgency > SAFETY_BLOCK_THRESHOLD);
    if safety_urgent {
        for need in needs.iter_mut() {
            if need.kind != NeedKind::Safety {
                need.blocked = true;
            }
        }
    } else {
        for need in needs.iter_mut() {
            need.blocked = false;
        }
    }
}
```

### data.rs 补充

NeedState 加 `decay_rate: f32` 字段。默认值按需求类型初始化：
- Nutrition: 0.5,  Hydration: 0.7,  Safety: 0.2
- Rest: 0.3,  Social: 0.1,  Curiosity: 0.15

## 智能验收

- `cargo check` 零错误
- 测试：sigmoid(0)=0.0, sigmoid(0.5)=~0.92, sigmoid(1.0)=~0.99
- 测试：safety_block → 非安全需求 blocked=true
