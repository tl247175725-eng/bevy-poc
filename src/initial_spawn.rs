//! 初始生成——已掏空。

use crate::world_state::WorldState;

/// 生成初始世界——空壳，无实体，无地形
pub fn spawn_initial_world() -> WorldState {
    WorldState::from_card_defs_file(crate::assets_util::card_defs_path())
}

/// 初始卡牌数
pub fn initial_card_count() -> usize { 0 }
