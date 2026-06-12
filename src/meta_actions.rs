//! 元动作 — 不可分解的原子行为
//!
//! 所有复杂行为 = 元动作的序列组合。
//! 元动作之间零耦合，类似四条公理的关系。
//!
//! 每个元动作执行前通过公理验证（compose/traverse/transform），
//! 执行后返回明确的结果。

use crate::spatial_index::EntityId;

/// 不可分解的原子动作
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MetaAction {
    /// 曼哈顿单步移动（dx/dy 不同时非零）
    Move { dx: i16, dy: i16 },
    /// 对目标施加 1 单位打击
    Strike { target: EntityId },
    /// 消耗目标，转化能量
    Consume { target: EntityId },
    /// 两物合并为新物
    Combine { ingredient: EntityId },
    /// 放置携带物到世界
    Release { x: u8, y: u8 },
    /// 维持当前状态 N tick
    Wait { ticks: u64 },
    /// 进入容纳态（不占格）
    Hide { cover_id: EntityId },
    /// 从容纳态退出
    Emerge,
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
}
