use crate::terrain::terrain_at;
use crate::world_rules::{GRID_HEIGHT, GRID_WIDTH};
use crate::world_state::{Entity, WorldState};

use super::laws::compose;
use super::profile::{medium_for_cell, EntityProfile, Medium};

pub type MediumAlias = Medium;

#[derive(Clone, Debug)]
pub struct CellSlot {
    pub medium: Medium,
    pub living_count: u8,
    pub corpse_count: u8,
    pub is_flock: bool,
    pub flock_type: String,
}

impl CellSlot {
    pub fn has_only_corpses(&self) -> bool {
        self.living_count == 0 && self.corpse_count > 0
    }
}

pub struct CellComposition {
    pub grid: Box<[[CellSlot; GRID_WIDTH as usize]; GRID_HEIGHT as usize]>,
}

fn empty_slot(medium: Medium) -> CellSlot {
    CellSlot {
        medium,
        living_count: 0,
        corpse_count: 0,
        is_flock: false,
        flock_type: String::new(),
    }
}

pub fn entity_occupies_active_cell(entity: &Entity) -> bool {
    !entity.profile.incorporeal
        && entity.host_tree_id.is_none()
        && !entity.in_pool
        && !entity.in_ground
        && !entity.in_den
        && !entity.in_burrow
        && !entity.hidden_in_grass
}

impl CellComposition {
    pub fn empty() -> Self {
        let mut grid = Box::new(std::array::from_fn(|_| {
            std::array::from_fn(|_| empty_slot("land".into()))
        }));
        for y in 0..GRID_HEIGHT {
            for x in 0..GRID_WIDTH {
                grid[y as usize][x as usize] = empty_slot("land".into());
            }
        }
        Self { grid }
    }

    pub fn from_world(world: &WorldState) -> Self {
        let mut grid = Box::new(std::array::from_fn(|_| {
            std::array::from_fn(|_| empty_slot("land".into()))
        }));

        for y in 0..GRID_HEIGHT {
            for x in 0..GRID_WIDTH {
                let terrain = terrain_at(world, x, y);
                let medium = medium_for_cell(terrain);
                grid[y as usize][x as usize] = empty_slot(medium);
            }
        }

        let mut comp = Self { grid };
        for entity in world.entities.values() {
            comp.occupy_entity(entity.x, entity.y, entity);
        }
        comp
    }

    pub fn refresh_mediums(&mut self, world: &WorldState) {
        for y in 0..GRID_HEIGHT {
            for x in 0..GRID_WIDTH {
                let terrain = terrain_at(world, x, y);
                self.grid[y as usize][x as usize].medium = medium_for_cell(terrain);
            }
        }
    }

    pub fn slot(&self, x: u8, y: u8) -> &CellSlot {
        &self.grid[y as usize][x as usize]
    }

    pub fn slot_mut(&mut self, x: u8, y: u8) -> &mut CellSlot {
        &mut self.grid[y as usize][x as usize]
    }

    pub fn occupy_entity(&mut self, x: u8, y: u8, entity: &Entity) {
        if !entity_occupies_active_cell(entity) {
            return;
        }
        let slot = self.slot_mut(x, y);
        if entity.is_corpse {
            slot.corpse_count = slot.corpse_count.saturating_add(1);
            return;
        }
        if slot.living_count == 0 {
            slot.flock_type = entity.profile.type_name.clone();
            slot.is_flock = entity.profile.social_structure != super::profile::SocialStructure::None;
        }
        slot.living_count = slot.living_count.saturating_add(1);
    }

    pub fn vacate_entity(&mut self, x: u8, y: u8, entity: &Entity) {
        if !entity_occupies_active_cell(entity) {
            return;
        }
        let slot = self.slot_mut(x, y);
        if entity.is_corpse {
            slot.corpse_count = slot.corpse_count.saturating_sub(1);
            return;
        }
        slot.living_count = slot.living_count.saturating_sub(1);
        if slot.living_count == 0 {
            slot.is_flock = false;
            slot.flock_type.clear();
        }
    }

    pub fn occupy(&mut self, x: u8, y: u8, profile: &EntityProfile) {
        if profile.incorporeal {
            return;
        }
        let is_corpse = profile.type_name.ends_with("Corpse");
        let slot = self.slot_mut(x, y);
        if is_corpse {
            slot.corpse_count = slot.corpse_count.saturating_add(1);
            return;
        }
        if slot.living_count == 0 {
            slot.flock_type = profile.type_name.clone();
        }
        slot.living_count = slot.living_count.saturating_add(1);
    }

    pub fn vacate(&mut self, x: u8, y: u8, profile: &EntityProfile) {
        if profile.incorporeal {
            return;
        }
        let is_corpse = profile.type_name.ends_with("Corpse");
        let slot = self.slot_mut(x, y);
        if is_corpse {
            slot.corpse_count = slot.corpse_count.saturating_sub(1);
            return;
        }
        slot.living_count = slot.living_count.saturating_sub(1);
        if slot.living_count == 0 {
            slot.is_flock = false;
            slot.flock_type.clear();
        }
    }

    pub fn can_occupy(&self, x: u8, y: u8, profile: &EntityProfile) -> bool {
        matches!(
            compose(self.slot(x, y), profile),
            super::laws::Composition::Allowed { .. }
        )
    }
}
