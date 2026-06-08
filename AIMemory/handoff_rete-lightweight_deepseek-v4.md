# 轻量 Rete：标签索引规则网络

**From**: deepseek-v4
**To**: cursor
**Date**: 2026-06-07
**Priority**: P0（地基，先于新内容）
**设计依据**: `AIMemory/design_rete-rule-index_deepseek-v4.md`

---

## 动机

当前 WorldRules 里每条规则是独立 if/else 链。"有 predator 标签？""有 prey 标签？""距离近？"——这些共同条件被反复检查。200 条规则时无所谓。2000 条时是灾难。

Rete 核心思想：共同条件合并成共享节点，标签变了只重评估受影响的分支。我们的标签索引和空间索引已经天然做了 Rete 的 alpha/beta memory——只缺一层：**共享条件节点**。

---

## 一、RuleIndex 结构

在 WorldRules 建一个 `RuleIndex`：

```rust
struct RuleIndex {
    // 标签 → 依赖它的规则列表（alpha memory）
    // "predator" → [Rule::Hunt, Rule::Stalk, Rule::FleeIfAlone]
    tag_index: HashMap<String, Vec<RuleId>>,

    // 共享条件节点（beta memory）
    // 多个规则共有的非标签条件合并成一个节点
    shared_nodes: Vec<SharedNode>,

    // 所有规则
    rules: Vec<RuleDef>,
}

struct SharedNode {
    id: NodeId,
    condition: SharedCondition,   // "距离 < N" "持武器" "同伴<2"
    children: Vec<NodeId>,        // 子节点
    rules: Vec<RuleId>,           // 该节点可达的规则
}

enum SharedCondition {
    InRange { radius: i32 },
    HasTagOnSelf { tag: String },   // 数值条件："持有武器" = 身上有 tool.weapon
    CountNearby { tag: String, threshold: i32 },
    SeasonIs { season: String },
    // ... 按需扩展
}
```

**规则定义**：

```rust
struct RuleDef {
    id: RuleId,
    name: String,
    required_tags: Vec<String>,        // 直接看 tag_index
    shared_node_path: Vec<NodeId>,     // 经过哪些共享节点
    action: fn(&mut WorldState, EntityId),
}
```

---

## 二、查询流程

当一张卡的标签变了：

```
卡获得 "predator" 标签
  → tag_index["predator"] → [Hunt, Stalk, FleeIfAlone]
  → 对每条规则，走它的 shared_node_path
  → Hunt: InRange(5)? 无 → 跳过
  → Stalk: InRange(5)? 有 → HasTagOnSelf(weapon)? 有 → 触发
  → FleeIfAlone: InRange(5)? 有 → CountNearby(companion, <2)? 有 → 触发
```

**和现在的区别**：

```
现在：每条规则从零开始判断所有条件
改后：标签索引跳过无关规则 → 共享节点跳过重复条件 → 只在末端判断差异条件
```

---

## 三、与现有系统共存

- WorldRules 仍为唯一入口，RuleIndex 是其内部数据结构
- 规则判定函数（`can_hunt`、`is_hunt_target_for` 等）不改——RuleIndex 决定哪些规则需要跑，它们仍做实际判定
- 空间索引（SpatialIndex）仍在——`InRange` 条件走空间索引
- 标签索引（card_defs.ron）仍在——`required_tags` 走已有标签系统

---

## 四、实施步骤

### Step 1：建 RuleIndex 数据结构

在 `world_rules.rs` 内加 `RuleIndex` 和 `SharedNode` 结构体。不注册任何规则——先建容器。

### Step 2：迁移现有规则

把当前 WorldRules 里的规则逐条注册到 RuleIndex。先迁 5 条：

- `predator near prey → hunt`
- `predator near prey + weapon → stalk`  
- `predator near camp + alone → flee_if_alone`
- `prey near grass → graze`
- `herbivore near grass + hungry → eat`

每迁一条，旧 if/else 路径保留——可以双轨并行。

### Step 3：改造 `ecosystem_behavior_key` 匹配

`tick_entity` 里某些行为判定从直接调 WorldRules 函数改为走 RuleIndex 查询。双轨——新路径失败回退旧路径。

### Step 4：验证 + 全部迁移

143+ 断言全绿后，删旧 if/else 路径。

---

## 五、涉及文件

| 文件 | 改动 |
|------|------|
| `src/core/world_rules.rs` | RuleIndex 结构 + tag_index 维护 |
| `src/systems/tick_predator.rs` | hunt 判定改走 RuleIndex |
| `src/systems/tick_herbivore.rs` | graze 判定改走 RuleIndex |
| `src/systems/tick_cover_forager.rs` | flee 判定改走 RuleIndex |

---

## 六、验收

- `cargo test`：143+ 断言不降
- 生态行为与改前一致
- 新增 2 条 RuleIndex 专项断言：tag_index 正确性、共享节点命中率

## 约束

- 不引入外部 crate
- 不碰标签定义
- 不碰渲染/UI
- 旧 if/else 路径保留到 Step 4 才删
- 记 FIX_LOG
