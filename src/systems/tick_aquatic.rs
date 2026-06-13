//! 水生系统——已掏空，等待需求匹配引擎。

use crate::world_state::WorldState;

/// 水生 tick——已移除（藻类再生、水生迁移及固着滤食行为将在新系统中重新实现）
pub fn tick_aquatic(_world: &mut WorldState, _delta: f32) {}
