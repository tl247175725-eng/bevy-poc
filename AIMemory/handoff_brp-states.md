# BRP 行为状态暴露

**From**: deepseek-v4
**To**: cursor
**Date**: 2026-06-10
**Priority**: P1（让 AI 看到每张卡的实时行为——消除"你描述→我猜"循环）

## 架构计划
扩展 SimStats 增加 `state_breakdown: HashMap<StateDesc, usize>` 统计每种（类型, 状态）的实体数。如 "sheep:Fleeing=3"。纯诊断，不影响模拟。

## 架构反馈
WorldState 不支持 Reflect 所以不能直接暴露。用统计摘要替代。

---

## 实现

### 扩展 SimStats

```rust
#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct SimStats {
    pub entity_count: usize,
    pub tick_count: u64,
    pub herbivore_count: usize,
    pub predator_count: usize,
    pub deaths: u64,
    // 新增
    pub state_breakdown: Vec<String>,  // ["sheep:Fleeing×3", "wolf:Hunting×2", ...]
    pub top_entities: Vec<String>,     // ["sheep:15", "grass:42", "wolf:3", ...]
    pub interactions_this_tick: u32,   // 本 tick 发生的交互数（捕猎+进食+繁殖）
}
```

### 更新 SimStats 的系统

在 `sync_sim_stats` 中遍历 `world.entities`，统计每种 (type_name × ecology_state) 的组合，格式化为字符串列表。

### 注册

SimStats 已经注册到 BRP。

## 验收
- `curl` 读 SimStats 能看到 state_breakdown 字段
- 包含 "wolf:Hunting"、"sheep:Fleeing"、"sheep:SeekingFood" 等条目
