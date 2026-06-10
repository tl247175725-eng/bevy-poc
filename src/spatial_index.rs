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

type GridBuckets = [[Vec<EntityId>; GRID_WIDTH as usize]; GRID_HEIGHT as usize];

fn empty_grid_buckets() -> GridBuckets {
    std::array::from_fn(|_| std::array::from_fn(|_| Vec::new()))
}

pub struct SpatialIndex {
    by_tag: HashMap<String, HashSet<EntityId>>,
    grid_buckets: GridBuckets,
    positions: HashMap<EntityId, (u8, u8)>,
    tags: HashMap<EntityId, Vec<String>>,
}

impl Default for SpatialIndex {
    fn default() -> Self {
        Self::new()
    }
}

impl SpatialIndex {
    pub fn new() -> Self {
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

    pub fn insert(&mut self, entity: &IndexedEntity) {
        self.positions.insert(entity.id, (entity.x, entity.y));
        self.tags.insert(entity.id, entity.tags.clone());
        for tag in &entity.tags {
            self.by_tag.entry(tag.clone()).or_default().insert(entity.id);
        }
        self.bucket_push(entity.x, entity.y, entity.id);
    }

    pub fn remove(&mut self, id: EntityId) {
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

    pub fn move_entity(&mut self, id: EntityId, new_x: u8, new_y: u8) {
        if let Some((old_x, old_y)) = self.positions.insert(id, (new_x, new_y)) {
            if old_x != new_x || old_y != new_y {
                self.bucket_remove(old_x, old_y, id);
                self.bucket_push(new_x, new_y, id);
            }
        } else {
            self.bucket_push(new_x, new_y, id);
        }
    }

    pub fn query_tag(&self, tag: &str) -> Vec<EntityId> {
        self.by_tag
            .get(tag)
            .map(|s| s.iter().copied().collect())
            .unwrap_or_default()
    }

    pub fn query_near(&self, x: u8, y: u8, tag: &str, radius: u8) -> Vec<EntityId> {
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

    pub fn position(&self, id: EntityId) -> Option<(u8, u8)> {
        self.positions.get(&id).copied()
    }

    pub fn has_grass_at(&self, x: u8, y: u8) -> bool {
        self.grid_buckets[y as usize][x as usize].iter().any(|id| {
            self.tags.get(id).is_some_and(|t| {
                t.iter()
                    .any(|tag| tag == "grass" || tag == "foodSource" || tag == "food_source")
            })
        })
    }

    #[cfg(test)]
    fn query_near_full_tag_scan(&self, x: u8, y: u8, tag: &str, radius: u8) -> Vec<EntityId> {
        let mut result = Vec::new();
        let tag_set = match self.by_tag.get(tag) {
            Some(s) => s,
            None => return result,
        };
        for &id in tag_set {
            if let Some(&(ex, ey)) = self.positions.get(&id) {
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
        // Cluster thousands of matches far from the query point; local bucket scan should win clearly.
        for i in 0..3000u64 {
            let x = 28 + (i % 6) as u8;
            let y = 16 + ((i / 6) % 6) as u8;
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
}
