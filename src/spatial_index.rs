use crate::world_rules::{GRID_HEIGHT, GRID_WIDTH};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntityId(pub u64);

pub struct IndexedEntity {
    pub id: EntityId,
    pub x: u8,
    pub y: u8,
    pub tags: Vec<String>,
}

// ===== 单层 2D 空间索引 =====

type GridBuckets = [[Vec<EntityId>; GRID_WIDTH as usize]; GRID_HEIGHT as usize];

fn empty_grid_buckets() -> GridBuckets {
    std::array::from_fn(|_| std::array::from_fn(|_| Vec::new()))
}

struct SpatialLayer {
    by_tag: HashMap<String, HashSet<EntityId>>,
    grid_buckets: GridBuckets,
    positions: HashMap<EntityId, (u8, u8)>,
    tags: HashMap<EntityId, Vec<String>>,
}

impl SpatialLayer {
    fn new() -> Self {
        Self {
            by_tag: HashMap::new(),
            grid_buckets: empty_grid_buckets(),
            positions: HashMap::new(),
            tags: HashMap::new(),
        }
    }

    fn bucket_push(&mut self, x: u8, y: u8, id: EntityId) {
        self.grid_buckets[y as usize][x as usize].push(id);
    }

    fn bucket_remove(&mut self, x: u8, y: u8, id: EntityId) {
        let bucket = &mut self.grid_buckets[y as usize][x as usize];
        bucket.retain(|&eid| eid != id);
    }

    fn insert(&mut self, id: EntityId, x: u8, y: u8, tags: &[String]) {
        self.positions.insert(id, (x, y));
        self.tags.insert(id, tags.to_vec());
        for tag in tags {
            self.by_tag.entry(tag.clone()).or_default().insert(id);
        }
        self.bucket_push(x, y, id);
    }

    fn remove(&mut self, id: EntityId) {
        if let Some((x, y)) = self.positions.remove(&id) {
            self.bucket_remove(x, y, id);
        }
        if let Some(tags) = self.tags.remove(&id) {
            for tag in tags {
                if let Some(set) = self.by_tag.get_mut(&tag) {
                    set.remove(&id);
                }
            }
        }
    }

    fn move_entity(&mut self, id: EntityId, new_x: u8, new_y: u8) {
        if let Some((old_x, old_y)) = self.positions.insert(id, (new_x, new_y)) {
            if old_x != new_x || old_y != new_y {
                self.bucket_remove(old_x, old_y, id);
                self.bucket_push(new_x, new_y, id);
            }
        } else {
            self.bucket_push(new_x, new_y, id);
        }
    }

    fn query_tag(&self, tag: &str) -> Vec<EntityId> {
        self.by_tag
            .get(tag)
            .map(|s| s.iter().copied().collect())
            .unwrap_or_default()
    }

    fn query_near(&self, x: u8, y: u8, tag: &str, radius: u8) -> Vec<EntityId> {
        let tag_set = match self.by_tag.get(tag) {
            Some(s) => s,
            None => return Vec::new(),
        };

        let min_x = x.saturating_sub(radius);
        let max_x = (x as u16 + radius as u16).min(GRID_WIDTH as u16 - 1) as u8;
        let min_y = y.saturating_sub(radius);
        let max_y = (y as u16 + radius as u16).min(GRID_HEIGHT as u16 - 1) as u8;

        let mut result = Vec::new();
        for gy in min_y..=max_y {
            for gx in min_x..=max_x {
                for &id in &self.grid_buckets[gy as usize][gx as usize] {
                    if !tag_set.contains(&id) {
                        continue;
                    }
                    if let Some(&(ex, ey)) = self.positions.get(&id) {
                        if x.abs_diff(ex).max(y.abs_diff(ey)) <= radius {
                            result.push(id);
                        }
                    }
                }
            }
        }
        result
    }

    fn query_radius_all(&self, x: u8, y: u8, radius: u8) -> Vec<EntityId> {
        let min_x = x.saturating_sub(radius);
        let max_x = (x as u16 + radius as u16).min(GRID_WIDTH as u16 - 1) as u8;
        let min_y = y.saturating_sub(radius);
        let max_y = (y as u16 + radius as u16).min(GRID_HEIGHT as u16 - 1) as u8;

        let mut result = Vec::new();
        for gy in min_y..=max_y {
            for gx in min_x..=max_x {
                for &id in &self.grid_buckets[gy as usize][gx as usize] {
                    if let Some(&(ex, ey)) = self.positions.get(&id) {
                        if x.abs_diff(ex).max(y.abs_diff(ey)) <= radius {
                            result.push(id);
                        }
                    }
                }
            }
        }
        result
    }

    fn position(&self, id: EntityId) -> Option<(u8, u8)> {
        self.positions.get(&id).copied()
    }

    fn has_grass_at(&self, x: u8, y: u8) -> bool {
        self.grid_buckets[y as usize][x as usize].iter().any(|id| {
            self.tags.get(id).is_some_and(|t| {
                t.iter()
                    .any(|tag| tag == "grass" || tag == "foodSource" || tag == "food_source")
            })
        })
    }
}

// ===== 分层 2D 空间索引 =====

/// 分层 2D 空间索引：每 Z 层一个独立的 SpatialLayer。
/// 只有活跃层消耗内存。Z=0 层始终存在（兼容性）。
pub struct SpatialIndex {
    layers: HashMap<i16, SpatialLayer>,
    /// 快速反向查询：EntityId → Z 层
    entity_z: HashMap<EntityId, i16>,
}

impl Default for SpatialIndex {
    fn default() -> Self {
        Self::new()
    }
}

impl SpatialIndex {
    pub fn new() -> Self {
        let mut layers = HashMap::new();
        layers.insert(0, SpatialLayer::new());
        Self {
            layers,
            entity_z: HashMap::new(),
        }
    }

    fn layer(&self, z: i16) -> &SpatialLayer {
        self.layers.get(&z).unwrap_or_else(|| {
            // 回退到 Z=0 层（兼容性保证）
            self.layers.get(&0).expect("Z=0 layer must exist")
        })
    }

    fn layer_or_insert(&mut self, z: i16) -> &mut SpatialLayer {
        self.layers.entry(z).or_insert_with(SpatialLayer::new)
    }

    pub fn insert(&mut self, entity: &IndexedEntity) {
        let z = self.entity_z.get(&entity.id).copied().unwrap_or(0);
        let layer = self.layer_or_insert(z);
        layer.insert(entity.id, entity.x, entity.y, &entity.tags);
    }

    pub fn insert_at_z(&mut self, entity: &IndexedEntity, z: i16) {
        self.entity_z.insert(entity.id, z);
        let layer = self.layer_or_insert(z);
        layer.insert(entity.id, entity.x, entity.y, &entity.tags);
    }

    pub fn remove(&mut self, id: EntityId) {
        let z = self.entity_z.remove(&id).unwrap_or(0);
        if let Some(layer) = self.layers.get_mut(&z) {
            layer.remove(id);
        }
    }

    pub fn move_entity(&mut self, id: EntityId, new_x: u8, new_y: u8) {
        let z = self.entity_z.get(&id).copied().unwrap_or(0);
        if let Some(layer) = self.layers.get_mut(&z) {
            layer.move_entity(id, new_x, new_y);
        }
    }

    /// 实体移动到新 (x,y,z) 坐标。Z 层变更时自动迁移数据。
    pub fn move_entity_3d(&mut self, id: EntityId, new_x: u8, new_y: u8, new_z: i16) {
        let old_z = self.entity_z.get(&id).copied().unwrap_or(0);
        if old_z != new_z {
            // 保存旧层的 tags 数据用于迁移
            let tags: Option<Vec<String>> = self.layers
                .get(&old_z)
                .and_then(|layer| layer.tags.get(&id))
                .cloned();
            // 从旧层移除
            if let Some(old_layer) = self.layers.get_mut(&old_z) {
                old_layer.remove(id);
            }
            // 加入新层（含 tags 迁移）
            self.entity_z.insert(id, new_z);
            let new_layer = self.layer_or_insert(new_z);
            if let Some(ref t) = tags {
                new_layer.insert(id, new_x, new_y, t);
            } else {
                new_layer.move_entity(id, new_x, new_y);
            }
        } else {
            self.move_entity(id, new_x, new_y);
        }
    }

    pub fn query_tag(&self, tag: &str) -> Vec<EntityId> {
        let mut result = Vec::new();
        for layer in self.layers.values() {
            result.extend(layer.query_tag(tag));
        }
        result
    }

    /// 标签过滤查询（兼容层：仅在 Z=0 层查询）
    pub fn query_near(&self, x: u8, y: u8, tag: &str, radius: u8) -> Vec<EntityId> {
        self.layer(0).query_near(x, y, tag, radius)
    }

    /// 按 Z 层查询（精确匹配，不 fallback）
    pub fn query_near_z(&self, x: u8, y: u8, z: i16, tag: &str, radius: u8) -> Vec<EntityId> {
        match self.layers.get(&z) {
            Some(layer) => layer.query_near(x, y, tag, radius),
            None => Vec::new(),
        }
    }

    /// 无标签过滤的半径查询（兼容层：仅在 Z=0 层查询）
    pub fn query_radius_all(&self, x: u8, y: u8, radius: u8) -> Vec<EntityId> {
        self.layer(0).query_radius_all(x, y, radius)
    }

    /// 按 Z 层的无过滤半径查询（精确匹配，不 fallback）
    pub fn query_radius_all_z(&self, x: u8, y: u8, z: i16, radius: u8) -> Vec<EntityId> {
        match self.layers.get(&z) {
            Some(layer) => layer.query_radius_all(x, y, radius),
            None => Vec::new(),
        }
    }

    pub fn position(&self, id: EntityId) -> Option<(u8, u8)> {
        let z = self.entity_z.get(&id).copied().unwrap_or(0);
        self.layer(z).position(id)
    }

    /// 返回 (x, y, z) 完整坐标
    pub fn position_3d(&self, id: EntityId) -> Option<(u8, u8, i16)> {
        let z = self.entity_z.get(&id).copied().unwrap_or(0);
        self.layer(z).position(id).map(|(x, y)| (x, y, z))
    }

    pub fn has_grass_at(&self, x: u8, y: u8) -> bool {
        self.layer(0).has_grass_at(x, y)
    }

    /// 按 Z 层查询草地
    pub fn has_grass_at_z(&self, x: u8, y: u8, z: i16) -> bool {
        self.layer(z).has_grass_at(x, y)
    }

    // ── 测试辅助 ──

    #[cfg(test)]
    fn query_near_full_tag_scan(&self, x: u8, y: u8, tag: &str, radius: u8) -> Vec<EntityId> {
        let mut result = Vec::new();
        let layer = self.layer(0);
        let tag_set = match layer.by_tag.get(tag) {
            Some(s) => s,
            None => return result,
        };
        for &id in tag_set {
            if let Some(&(ex, ey)) = layer.positions.get(&id) {
                if x.abs_diff(ex).max(y.abs_diff(ey)) <= radius {
                    result.push(id);
                }
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    fn indexed(id: u64, x: u8, y: u8, tags: &[&str]) -> IndexedEntity {
        IndexedEntity {
            id: EntityId(id),
            x,
            y,
            tags: tags.iter().map(|t| (*t).to_string()).collect(),
        }
    }

    #[test]
    fn query_near_scans_local_bucket_only() {
        let mut idx = SpatialIndex::new();
        idx.insert(&indexed(1, 5, 5, &["sheep"]));
        idx.insert(&indexed(2, 20, 20, &["sheep"]));
        let near = idx.query_near(5, 5, "sheep", 2);
        assert_eq!(near.len(), 1);
        assert_eq!(near[0], EntityId(1));
    }

    #[test]
    fn grid_buckets_track_move_and_remove() {
        let mut idx = SpatialIndex::new();
        let id = EntityId(1);
        idx.insert(&indexed(1, 5, 5, &["grass"]));
        assert!(idx.has_grass_at(5, 5));
        idx.move_entity(id, 8, 8);
        assert!(!idx.has_grass_at(5, 5));
        assert!(idx.has_grass_at(8, 8));
        idx.remove(id);
        assert!(!idx.has_grass_at(8, 8));
        assert!(idx.query_tag("grass").is_empty());
    }

    #[test]
    fn query_near_is_faster_than_full_tag_scan_at_scale() {
        let mut idx = SpatialIndex::new();
        for i in 0..3000u64 {
            let x = 24 + (i % 6) as u8;
            let y = 12 + ((i / 6) % 6) as u8;
            idx.insert(&indexed(i, x, y, &["grass"]));
        }
        idx.insert(&indexed(3000, 10, 7, &["grass"]));

        let baseline_start = Instant::now();
        for _ in 0..100 {
            let near = idx.query_near_full_tag_scan(10, 7, "grass", 6);
            assert_eq!(near.len(), 1);
        }
        let baseline = baseline_start.elapsed();

        let optimized_start = Instant::now();
        for _ in 0..100 {
            let near = idx.query_near(10, 7, "grass", 6);
            assert_eq!(near.len(), 1);
        }
        let optimized = optimized_start.elapsed();

        assert!(
            optimized * 10 < baseline,
            "optimized {:?} should be <1/10 of baseline {:?}",
            optimized,
            baseline
        );
    }

    // ── Z 轴测试 ──

    #[test]
    fn layers_isolate_z() {
        let mut idx = SpatialIndex::new();
        // Z=0: sheep at (5,5)
        idx.insert_at_z(&indexed(1, 5, 5, &["sheep"]), 0);
        // Z=-1: sheep at (5,5) underground
        idx.insert_at_z(&indexed(2, 5, 5, &["sheep"]), -1);

        // Z=0 查询只看到 Z=0 的实体
        let near0 = idx.query_near_z(5, 5, 0, "sheep", 2);
        assert_eq!(near0.len(), 1);
        assert_eq!(near0[0], EntityId(1));

        // Z=-1 查询只看到 Z=-1 的实体
        let near_m1 = idx.query_near_z(5, 5, -1, "sheep", 2);
        assert_eq!(near_m1.len(), 1);
        assert_eq!(near_m1[0], EntityId(2));

        // 兼容层 query_near 只查 Z=0
        let near_compat = idx.query_near(5, 5, "sheep", 2);
        assert_eq!(near_compat.len(), 1);
        assert_eq!(near_compat[0], EntityId(1));
    }

    #[test]
    fn move_entity_3d_switches_layer() {
        let mut idx = SpatialIndex::new();
        idx.insert_at_z(&indexed(1, 5, 5, &["sheep"]), 0);

        // 迁移到地下层
        idx.move_entity_3d(EntityId(1), 5, 5, -1);

        // Z=0 不再有该实体
        assert!(idx.query_near_z(5, 5, 0, "sheep", 2).is_empty());
        // Z=-1 有该实体
        assert_eq!(idx.query_near_z(5, 5, -1, "sheep", 2).len(), 1);
    }

    #[test]
    fn default_layer_is_zero() {
        let mut idx = SpatialIndex::new();
        // 旧式 insert（不指定 Z）→ 默认 Z=0
        idx.insert(&indexed(1, 3, 3, &["grass"]));
        assert_eq!(idx.query_near(3, 3, "grass", 1).len(), 1);
        assert_eq!(idx.query_near_z(3, 3, 0, "grass", 1).len(), 1);
        // 非 0 层为空
        assert!(idx.query_near_z(3, 3, -1, "grass", 1).is_empty());
    }

    #[test]
    fn position_3d_returns_z() {
        let mut idx = SpatialIndex::new();
        idx.insert_at_z(&indexed(1, 7, 3, &["sheep"]), 2);
        assert_eq!(idx.position_3d(EntityId(1)), Some((7, 3, 2)));
    }
}
