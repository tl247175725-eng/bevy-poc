//! 元动作 — 不可分解的原子行为（25 变体，7 层覆盖）
//!
//! 所有复杂行为 = 元动作的序列组合。
//! 元动作之间零耦合，类似四条公理的关系。
//!
//! 每个元动作执行前通过公理验证（compose/traverse/transform），
//! 执行后返回明确的结果。
//!
//! 依据 design-philosophy-v5.md §9.2-9.7

use crate::spatial_index::EntityId;

/// 不可分解的原子动作 — 25 变体覆盖 7 层
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MetaAction {
    // === 物理层 (11) ===
    /// 曼哈顿单步移动（dx/dy 不同时非零）
    Move { dx: i16, dy: i16 },
    /// 唯一冲击——战斗/加工同源
    Strike { target: EntityId },
    /// 摄入/消耗
    Consume { target: EntityId },
    /// 属性代数合并，无配方
    Combine { ingredient: EntityId },
    /// 整体→多个部分
    Separate { target: EntityId },
    /// 限制行动自由
    Constrain { target: EntityId },
    /// 解除约束，放到世界
    Release { x: u8, y: u8 },
    /// 主动不行动（原 Wait 改名）
    Pause { ticks: u64 },
    /// 进入容纳态（不占格）
    Hide { cover_id: EntityId },
    /// 从容纳态退出
    Emerge,
    /// 改变物质属性（烹饪/燃烧/冶炼）
    Alter { target: EntityId },

    // === 信息层 (4) ===
    /// 发送语义信息
    Signal { content: String, range: u8 },
    /// 解码语义
    Receive { source: Option<EntityId> },
    /// 推理/比较/判断
    Process { input: String },
    /// 记忆/记录/铭刻
    Store { content: String, target: Option<EntityId> },

    // === 生物层 (3) ===
    /// 产出新实体
    Reproduce { partner: Option<EntityId> },
    /// 父→子传递标签
    Inherit { parent: EntityId },
    /// 生物体内变化（成长/变态/愈合）
    Transform { target: EntityId },

    // === 心智层 (3) ===
    /// 从"知道"到"去做"的跳变
    Decide { intention: String },
    /// 持久性能力传授
    Teach { target: EntityId, skill: String },
    /// 创建持续性关系链接
    Bond { target: EntityId, bond_type: String },

    // === 社会层 (2) ===
    /// 命名/定价/所有权/身份
    Assign { target: EntityId, role: String },
    /// 契约/债务/誓言
    Commit { target: EntityId, obligation: String },

    // === 超自然层 (2) ===
    /// 无前因产出新实体
    Create { template: String, x: u8, y: u8 },
    /// 从因果链彻底移除
    Destroy { target: EntityId },
}

/// 元动作执行结果
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionResult {
    /// 动作成功执行
    Success,
    /// 被公理阻止
    Blocked { reason: String },
    /// 动作对当前上下文无效
    Invalid,
    /// 消耗完成，获得能量
    Consumed { energy_gained: u32 },
    /// 目标被击杀
    Killed { corpse_spawned: bool },
    /// Separate 产物数量
    Separated { parts: u32 },
    /// Bond 建立完成
    Bound,
    /// Commit 生效
    Committed,
    /// Create 完成
    Created,
    /// Destroy 完成
    Destroyed,
}
