//! 公告板——已掏空，等待需求匹配引擎。

use crate::world_state::WorldState;
use std::collections::HashMap;

/// 已掏空的公告板数据
#[derive(Debug, Clone, Default)]
pub struct BulletinBoard {
    pub channels: HashMap<String, Vec<(u8, u8)>>,
    pub tick_since_update: u64,
}

impl BulletinBoard {
    pub fn nearest_zone_center(&self, _tags: &[String], _channel: &str, _x: u8, _y: u8) -> Option<(u8, u8)> {
        None
    }
}

/// 更新公告板——已移除
pub fn maybe_update(_world: &WorldState) {}

/// 频道区域寻的——已移除
pub fn seek_target_channel(_tag: &str) -> &str { "" }
