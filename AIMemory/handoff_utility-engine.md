# 效用打分引擎

**From**: deepseek-v4
**To**: cursor
**Date**: 2026-06-11
**Priority**: P0（核心设计——替代固定优先级，需求动态变化）

**参考设计**: `AIMemory/design_human-readable_v4.md` 第七部分

## 架构计划
保留现有 drive 执行层不变。替换 `active_drives` 中的固定优先级选择为动态效用打分：每 tick 所有可能的 drive 同时评分，选最高分执行。need 值随时间衰减累积。

## 架构反馈
现有 `drive:seek(priority=N)` 中的 `priority` 字段退役，改为 `need:eat(rate=2,curve=steep)` 驱动。EntityProfile 新增 need 值字段。

## 智能验收
- 饿时 seek 分高，饱时 seek 分低
- 狼在远处不压过快饿死的羊
- smoke test 行为不退化

---

## 实现

### 1. EntityProfile 新增 need 字段

```rust
pub struct EntityProfile {
    // ... 现有字段 ...
    pub needs: SmallVec<[NeedState; 4]>, // 需求运行时状态
}

pub struct NeedState {
    pub kind: String,       // "eat", "safety", "rest", "explore"
    pub current: f32,       // 当前紧迫度 (0.0-100.0)
    pub decay_rate: f32,    // 每 tick 增长率
    pub curve: NeedCurve,   // 响应曲线
}

pub enum NeedCurve {
    Steep,  // 饥饿型——低时几乎无感，高时指数飙升
    Flat,   // 好奇心型——持续平稳
    Sharp,  // 恐惧型——阈值跳跃
}
```

### 2. need 标签解析

```
need:eat(rate=0.5,curve=steep)    → NeedState { kind:"eat", decay_rate:0.5, curve:Steep }
need:safety(rate=1.0,curve=sharp) → NeedState { kind:"safety", decay_rate:1.0, curve:Sharp }
need:rest(rate=0.2,curve=flat)    → NeedState { kind:"rest", decay_rate:0.2, curve:Flat }
```

### 3. 每 tick 衰减

在 `tick_reactive` 开头：

```rust
// need 自然增长
for need in &mut profile.needs {
    need.current = (need.current + need.decay_rate).min(100.0);
}
// safety 随 predator 距离调整
if predator_nearby {
    // 距离越近增长越快
    safety.current += (6.0 - distance as f32) * 3.0;
}
```

### 4. drive 打分

```rust
fn score_drive(need: &NeedState, drive: &ActiveDrive) -> f32 {
    let raw = match need.curve {
        NeedCurve::Steep => (need.current / 100.0).powi(3), // 低时不急，高时暴涨
        NeedCurve::Flat => need.current / 100.0,
        NeedCurve::Sharp => if need.current > 60.0 { need.current / 100.0 } else { 0.0 },
    };
    raw * drive.range_attenuation() // 距离衰减
}
```

### 5. 选择最高分

```rust
// 替代原有的 .max_by_key(|d| d.priority)
let best = drives.iter()
    .map(|d| (d, score_drive_for(d)))
    .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
    .map(|(d, _)| d);
```

### 6. 动量防震荡

```rust
// 如果上次选中的 drive 和这次最高分差距 < 10%，保留上次选择
if let Some(last) = last_drive {
    let last_score = score_drive_for(last);
    if best_score - last_score < 0.1 * best_score {
        return last; // 不切换
    }
}
```

---

## 涉及文件

| 文件 | 改动 |
|---|---|
| `src/axioms/profile.rs` | NeedState + 解析 |
| `src/systems/tick_reactive.rs` | need 衰减 + 打分 + 选择 |
| `assets/card_defs.ron` | 动物 need 标签替换 priority |
| `AIMemory/design_machine-readable_v4.md` | 更新测试映射 |

## 验收
- `cargo test --release` 全 PASS
- `cargo run --release -- --smoke-test` PASS
- 同一只羊：饱时闲逛，饿时主动找草，快饿死时冒险跨过狼区域找草
