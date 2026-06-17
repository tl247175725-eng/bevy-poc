//! 需求匹配引擎数据结构 — 纯数据定义，无行为逻辑
//!
//! 依据 design-philosophy-v5.md §3、§8.2

use std::collections::HashMap;

use crate::meta_values::{
    CURIOSITY_DECAY_RATE, HYDRATION_BASELINE_RATIO, SOCIAL_DECAY_RATE,
};
use crate::spatial_index::EntityId;

// ===== 需求状态 =====

/// 运行时需求状态 — 每个实体持有一组 NeedState
#[derive(Debug, Clone)]
pub struct NeedState {
    /// 需求类型
    pub kind: NeedKind,
    /// 当前水平 (0 = 完全满足, 1 = 完全匮乏)
    pub current: f32,
    /// 基线阈值（低于此值 → 激活）
    pub baseline: f32,
    /// 当前紧迫度 = sigmoid(baseline - current)
    pub urgency: f32,
    /// 是否被安全阻断
    pub blocked: bool,
    /// 每 tick 衰减速率（需求向匮乏方向递增）
    pub decay_rate: f32,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NeedKind {
    /// 进食
    Nutrition,
    /// 饮水
    Hydration,
    /// 安全（可阻断其他需求）
    Safety,
    /// 休息
    Rest,
    /// 社交
    Social,
    /// 探索/学习
    Curiosity,
}

/// 按需求类型的默认衰减速率
pub fn default_decay_rate(kind: &NeedKind) -> f32 {
    match kind {
        NeedKind::Nutrition => crate::meta_values::nutrition_decay_per_tick(1.0),
        NeedKind::Hydration => crate::meta_values::nutrition_decay_per_tick(1.5),
        NeedKind::Safety => crate::meta_values::nutrition_decay_per_tick(0.5),
        NeedKind::Rest => 0.3,
        NeedKind::Social => SOCIAL_DECAY_RATE,
        NeedKind::Curiosity => CURIOSITY_DECAY_RATE,
    }
}

// ===== 知识条目 =====

/// 知识条目 — "如何满足需求"的可执行描述
#[derive(Debug, Clone)]
pub struct KnowledgeEntry {
    pub id: KnowledgeId,
    /// 人类可读名（如"生篝火"）
    pub name: String,
    /// 功能前提：对材料属性的要求（不写死具体材料）
    pub functional_prerequisites: Vec<PropertyRequirement>,
    /// 分解步骤
    pub decomposition: Vec<DecompositionStep>,
    /// 产出效果
    pub effects: Vec<EffectDescriptor>,
    /// 来源
    pub source: KnowledgeSource,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KnowledgeId(pub u64);

/// 属性需求 — 对物体抽象属性的约束
#[derive(Debug, Clone)]
pub struct PropertyRequirement {
    /// 属性名: "flammability", "hardness", "edge"
    pub property: String,
    /// 比较操作
    pub operator: CompareOp,
    /// 阈值
    pub threshold: f32,
    /// 需要多少量
    pub quantity_needed: f32,
    /// 标签值（可选）— property=="has_tag" 时查目标实体是否携带此标签
    pub tag_value: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CompareOp {
    GreaterThan,
    LessThan,
    GreaterOrEqual,
    Equal,
    /// 属性存在即可（如 edge:present）
    Present,
}

/// 分解步骤 — 知识条目的执行序列
#[derive(Debug, Clone)]
pub enum DecompositionStep {
    /// 获取满足某组属性的物体
    Acquire {
        requirements: Vec<PropertyRequirement>,
    },
    /// 执行元动作
    Act {
        action: String,
        target: Option<String>,
    },
    /// 对已获取的物体执行操作
    Combine {
        ingredient_indices: Vec<usize>,
    },
}

/// 效果描述 — 知识条目的产出
#[derive(Debug, Clone)]
pub struct EffectDescriptor {
    /// 满足的需求类型
    pub satisfies: NeedKind,
    /// 满足程度 (0–1)
    pub magnitude: f32,
}

/// 知识来源
#[derive(Debug, Clone)]
pub enum KnowledgeSource {
    /// 常识（物种/文化预设）
    CommonSense,
    /// 被 Teach 传授
    Taught,
    /// 实践改良
    Practiced,
    /// "玩"发现
    Discovered,
}

// ===== 知识图 =====

#[derive(Debug, Clone)]
pub struct KnowledgeGraph {
    pub entries: HashMap<KnowledgeId, KnowledgeEntry>,
    pub next_id: u64,
}

impl KnowledgeGraph {
    /// 创建空知识图
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            next_id: 1,
        }
    }

    /// 添加知识条目（自动分配 id）
    pub fn add(&mut self, mut entry: KnowledgeEntry) -> KnowledgeId {
        let kid = KnowledgeId(self.next_id);
        self.next_id += 1;
        entry.id = kid.clone();
        self.entries.insert(kid.clone(), entry);
        kid
    }
}

// ===== 社会记忆目录 =====

/// 社会记忆目录 — 已知其他实体擅长的知识
#[derive(Debug, Clone)]
pub struct SocialDirectory {
    /// EntityId → 该实体擅长的知识条目列表
    pub expertise: HashMap<EntityId, Vec<KnowledgeId>>,
}

// ===== 候选行动（匹配产出） =====

/// 候选行动 — 需求匹配引擎的输出
#[derive(Debug, Clone)]
pub struct CandidateAction {
    pub knowledge_id: KnowledgeId,
    /// 覆盖了哪些激活需求
    pub matched_needs: Vec<NeedKind>,
    /// 可达成度 (0–1)
    pub achievability: f32,
    /// 预估风险 (0–1)
    pub risk: f32,
    pub source: CandidateSource,
}

#[derive(Debug, Clone)]
pub enum CandidateSource {
    /// A 方向：从知识图匹配
    KnowledgeGraph,
    /// B 方向：从环境感知推导
    EnvironmentPerceive,
    /// B 方向进阶：本质查询引擎组合
    EssenceCombine,
}

// ===== 基线推导函数 =====

/// baseline_nutrition = mass × metabolism_rate
pub fn baseline_nutrition(mass_kg: f32, metabolism_rate: f32) -> f32 {
    mass_kg * metabolism_rate
}

/// baseline_hydration = mass × 0.03 (~30ml/kg 基准)
pub fn baseline_hydration(mass_kg: f32) -> f32 {
    mass_kg * HYDRATION_BASELINE_RATIO
}

/// baseline_safety = 1.0（始终追求安全，紧迫度来自感知威胁）
pub fn baseline_safety() -> f32 {
    1.0
}