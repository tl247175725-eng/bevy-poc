use crate::world_rules::GRID_WIDTH;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntityId(pub u64);

pub struct IndexedEntity {
    pub id: EntityId,
    pub x: u8,
    pub y: u8,
    pub tags: Vec<String>,
}

pub struct SpatialIndex {
  by_tag: HashMap<String, HashSet<EntityId>>,
  by_cell: HashMap<u32, HashSet<EntityId>>,
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
            by_cell: HashMap::new(),
            positions: HashMap::new(),
            tags: HashMap::new(),
        }
    }

    fn cell_key(x: u8, y: u8) -> u32 {
        (y as u32) * (GRID_WIDTH as u32) + (x as u32)
    }

    pub fn insert(&mut self, entity: &IndexedEntity) {
        self.positions.insert(entity.id, (entity.x, entity.y));
        self.tags.insert(entity.id, entity.tags.clone());
        for tag in &entity.tags {
            self.by_tag.entry(tag.clone()).or_default().insert(entity.id);
        }
        self.by_cell
            .entry(Self::cell_key(entity.x, entity.y))
            .or_default()
            .insert(entity.id);
    }

    pub fn remove(&mut self, id: EntityId) {
        if let Some((x, y)) = self.positions.remove(&id) {
            if let Some(cell) = self.by_cell.get_mut(&Self::cell_key(x, y)) {
                cell.remove(&id);
            }
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
            if let Some(cell) = self.by_cell.get_mut(&Self::cell_key(old_x, old_y)) {
                cell.remove(&id);
            }
        }
        self.by_cell
            .entry(Self::cell_key(new_x, new_y))
            .or_default()
            .insert(id);
    }

    pub fn query_tag(&self, tag: &str) -> Vec<EntityId> {
        self.by_tag
            .get(tag)
            .map(|s| s.iter().copied().collect())
            .unwrap_or_default()
    }

    pub fn query_near(&self, x: u8, y: u8, tag: &str, radius: u8) -> Vec<EntityId> {
        let mut result = Vec::new();
        let tag_set = match self.by_tag.get(tag) {
            Some(s) => s,
            None => return result,
        };
        for &id in tag_set {
            if let Some(&(ex, ey)) = self.positions.get(&id) {
                let dx = x.abs_diff(ex);
                let dy = y.abs_diff(ey);
                if dx.max(dy) <= radius {
                    result.push(id);
                }
            }
        }
        result
    }

    pub fn position(&self, id: EntityId) -> Option<(u8, u8)> {
        self.positions.get(&id).copied()
    }

    pub fn has_grass_at(&self, x: u8, y: u8) -> bool {
        let cell = Self::cell_key(x, y);
        self.by_cell.get(&cell).is_some_and(|ids| {
            ids.iter().any(|id| {
                self.tags.get(id).is_some_and(|t| {
                    t.iter()
                        .any(|tag| tag == "grass" || tag == "foodSource" || tag == "food_source")
                })
            })
        })
    }
}
