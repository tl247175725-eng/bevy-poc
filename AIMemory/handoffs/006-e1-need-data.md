# Handoff 006-e1 — 需求匹配引擎数据结构

## 架构计划

**改什么：** 新建 `src/need_match/mod.rs` + `src/need_match/data.rs`（2 文件）
**为什么：** 需求匹配引擎需要统一数据结构，先定义后实现
**依据：** `design-philosophy-v5.md` §3、§8.2 B 层生命/心智、本质查询引擎架构

### data.rs — 所有数据结构

```rust
// === 需求状态 ===
#[derive(Debug, Clone)]
pub struct NeedState {
    pub kind: NeedKind,          // 需求类型
    pub current: f32,            // 当前水平 (0=完全满足, 1=完全匮乏)
    pub baseline: f32,           // 基线阈值（低于此值→激活）
    pub urgency: f32,            // 当前紧迫度 = sigmoid(baseline - current)
    pub blocked: bool,           // 是否被安全阻断
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NeedKind {
    Nutrition,    // 进食
    Hydration,    // 饮水
    Safety,       // 安全（可阻断其他需求）
    Rest,         // 休息
    Social,       // 社交
    Curiosity,    // 探索/学习
}

// === 知识条目 ===
#[derive(Debug, Clone)]
pub struct KnowledgeEntry {
    pub id: KnowledgeId,
    pub name: String,                           // 人类可读名（如"生篝火"）
    pub functional_prerequisites: Vec<PropertyRequirement>, // 功能前提
    pub decomposition: Vec<DecompositionStep>,  // 分解步骤
    pub effects: Vec<EffectDescriptor>,          // 产出效果
    pub source: KnowledgeSource,                // 来源
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KnowledgeId(pub u64);

#[derive(Debug, Clone)]
pub struct PropertyRequirement {
    pub property: String,      // 属性名: "flammability", "hardness", "edge"
    pub operator: CompareOp,   // >, <, >=, ==
    pub threshold: f32,        // 阈值
    pub quantity_needed: f32,  // 需要多少
}

#[derive(Debug, Clone)]
pub enum CompareOp {
    GreaterThan,
    LessThan,
    GreaterOrEqual,
    Equal,
    Present,  // 属性存在即可（如 edge:present）
}

#[derive(Debug, Clone)]
pub enum DecompositionStep {
    /// 获取满足某组属性的物体
    Acquire { requirements: Vec<PropertyRequirement> },
    /// 执行元动作
    Act { action: String, target: Option<String> },
    /// 对已获取的物体执行操作
    Combine { ingredient_indices: Vec<usize> },
}

#[derive(Debug, Clone)]
pub struct EffectDescriptor {
    pub satisfies: NeedKind,     // 满足的需求类型
    pub magnitude: f32,          // 满足程度 (0-1)
}

#[derive(Debug, Clone)]
pub enum KnowledgeSource {
    CommonSense,     // 常识（物种/文化预设）
    Taught,          // 被 Teach 传授
    Practiced,       // 实践改良
    Discovered,      // "玩"发现
}

// === 知识图 ===
#[derive(Debug, Clone)]
pub struct KnowledgeGraph {
    pub entries: HashMap<KnowledgeId, KnowledgeEntry>,
    pub next_id: u64,
}

// === 社会记忆目录 ===
#[derive(Debug, Clone)]
pub struct SocialDirectory {
    /// EntityId → 已知该实体擅长的知识条目列表
    pub expertise: HashMap<EntityId, Vec<KnowledgeId>>,
}

// === 候选行动（匹配产出） ===
#[derive(Debug, Clone)]
pub struct CandidateAction {
    pub knowledge_id: KnowledgeId,
    pub matched_needs: Vec<NeedKind>,   // 覆盖了哪些激活需求
    pub achievability: f32,             // 可达成度 (0-1)
    pub risk: f32,                      // 预估风险 (0-1)
    pub source: CandidateSource,
}

#[derive(Debug, Clone)]
pub enum CandidateSource {
    KnowledgeGraph,        // A 方向：从知识图匹配
    EnvironmentPerceive,   // B 方向：从环境感知推导
    EssenceCombine,        // B 方向进阶：本质查询引擎组合
}
```

### 基线推导函数

```rust
/// baseline_nutrition = metabolism_rate × mass × species_factor
pub fn baseline_nutrition(mass_kg: f32, metabolism_rate: f32) -> f32 {
    mass_kg * metabolism_rate
}

/// baseline_hydration = mass × species_hydration_factor
pub fn baseline_hydration(mass_kg: f32) -> f32 {
    mass_kg * 0.03  // ~30ml/kg 基准
}

/// baseline_safety = 1.0（始终追求安全，紧迫度来自感知威胁）
pub fn baseline_safety() -> f32 { 1.0 }
```

### mod.rs

```rust
pub mod data;
pub use data::*;
```

### lib.rs 注册

```rust
pub mod need_match;
```

## 架构反馈

**设计哲学一致性：**
- 需求从 NeedState 表达，基线从 mass/metabolism 推导 ✅
- 知识条目功能前提使用属性需求，不写死具体材料 ✅
- 三重过滤（功能→可行→风险）对应 CandidateAction 的三个字段 ✅
- 社会记忆目录独立于知识图 ✅
- 知识来源四档对应常识/传授/实践/玩 ✅

## 智能验收

- `cargo check` 零错误
- `cargo test` 全 PASS
- 纯数据结构定义，无任何行为逻辑
