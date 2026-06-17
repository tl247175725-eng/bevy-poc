# Handoff 005-c — 扩展元动作枚举至 25 个

## 架构计划

**改什么：** `src/meta_actions.rs`（1 文件）
**为什么：** 当前 8 变体 → 25 变体完整定义。纯枚举，零行为逻辑。
**依据：** `design-philosophy-v5.md` §9.2-9.7

### 元动作枚举（25 变体）

```rust
pub enum MetaAction {
    // === 物理层 (11) ===
    Move { dx: i16, dy: i16 },           // 曼哈顿单步
    Strike { target: EntityId },          // 唯一冲击——战斗/加工同源
    Consume { target: EntityId },         // 摄入/消耗
    Combine { ingredient: EntityId },     // 属性代数合并，无配方
    Separate { target: EntityId },        // 整体→多个部分
    Constrain { target: EntityId },       // 限制行动自由
    Release { x: u8, y: u8 },            // 解除约束，放到世界
    Pause { ticks: u64 },                // 主动不行动（原 Wait 改名）
    Hide { cover_id: EntityId },          // 进入容纳态
    Emerge,                               // 从容纳态退出
    Alter { target: EntityId },           // 改变物质属性（烹饪/燃烧/冶炼）

    // === 信息层 (4) ===
    Signal { content: String, range: u8 },    // 发送语义信息
    Receive { source: Option<EntityId> },     // 解码语义
    Process { input: String },                // 推理/比较/判断
    Store { content: String, target: Option<EntityId> }, // 记忆/记录/铭刻

    // === 生物层 (3) ===
    Reproduce { partner: Option<EntityId> },  // 产出新实体
    Inherit { parent: EntityId },             // 父→子传递标签
    Transform { target: EntityId },           // 生物体内变化（成长/变态/愈合）

    // === 心智层 (3) ===
    Decide { intention: String },             // 从"知道"到"去做"的跳变
    Teach { target: EntityId, skill: String },// 持久性能力传授
    Bond { target: EntityId, bond_type: String }, // 创建持续性关系链接

    // === 社会层 (2) ===
    Assign { target: EntityId, role: String }, // 命名/定价/所有权/身份
    Commit { target: EntityId, obligation: String }, // 契约/债务/誓言

    // === 超自然层 (2) ===
    Create { template: String, x: u8, y: u8 }, // 无前因产出新实体
    Destroy { target: EntityId },              // 从因果链彻底移除
}
```

### 变动说明

| 旧 | 新 |
|---|---|
| `Wait { ticks }` | → `Pause { ticks }`（语义更精确：主动不行动） |
| 保留 Move/Strike/Consume/Combine/Release/Hide/Emerge | 不变 |
| 新增 17 个变体 | Separate/Constrain/Alter/Signal/Receive/Process/Store/Reproduce/Inherit/Transform/Decide/Teach/Bond/Assign/Commit/Create/Destroy |

### ActionResult 更新

添加新结果变体：
- `Separated { parts: u32 }` — Separate 产物
- `Bound` — Bond 建立完成
- `Committed` — Commit 生效
- `Created` — Create 完成
- `Destroyed` — Destroy 完成

### 不做的

- 不实现任何元动作的执行逻辑（纯枚举定义）
- 不修改 ActionResult 原有变体
- 元动作 name 用英文（注释中文说明）

## 架构反馈

**设计哲学一致性：**
- 25 个变体覆盖 7 层（物理/信息/生物/心智/社会/超自然）+ 2 底层机制 ✅
- 多 AI 独立检验收敛 ✅
- Elapse/Chance 不在 MetaAction 枚举中（它们是底层机制，在 tick 循环里）✅

## 智能验收

- `cargo check` 零错误
- `cargo test` 全 PASS
- 枚举定义完整（25 变体，7 层覆盖）
