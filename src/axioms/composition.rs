use crate::terrain::terrain_at;
use crate::world_rules::{GRID_HEIGHT, GRID_WIDTH};
use crate::world_state::WorldState;

use super::laws::compose;
use super::profile::{medium_for_cell, EntityProfile, Medium};

pub type MediumAlias = Medium;

#[derive(Clone, Debug)]
pub struct CellSlot {
    pub medium: Medium,
    pub max_size: u8,
    pub current_size: u8,
}

pub struct CellComposition {
    pub grid: Box<[[CellSlot; GRID_WIDTH as usize]; GRID_HEIGHT as usize]>,
}

impl CellComposition {
    pub fn empty() -> Self {
        let mut grid = Box::new(std::array::from_fn(|_| {
            std::array::from_fn(|_| CellSlot {
                medium: "land".into(),
                max_size: 3,
                current_size: 0,
            })
        }));
        for y in 0..GRID_HEIGHT {
            for x in 0..GRID_WIDTH {
                let medium = "land".to_string();
                grid[y as usize][x as usize] = CellSlot {
                    medium: medium.clone(),
                    max_size: default_max_size(&medium),
                    current_size: 0,
                };
            }
        }
        Self { grid }
    }

    pub fn from_world(world: &WorldState) -> Self {
        let mut grid = Box::new(std::array::from_fn(|_| {
            std::array::from_fn(|_| CellSlot {
                medium: "land".into(),
                max_size: 3,
                current_size: 0,
            })
        }));

        for y in 0..GRID_HEIGHT {
            for x in 0..GRID_WIDTH {
                let terrain = terrain_at(world, x, y);
                let medium = medium_for_cell(terrain);
                let max_size = default_max_size(&medium);
                grid[y as usize][x as usize] = CellSlot {
                    medium,
                    max_size,
                    current_size: 0,
                };
            }
        }

        let mut comp = Self { grid };
        for entity in world.entities.values() {
            comp.occupy(entity.x, entity.y, &entity.profile);
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

    pub fn occupy(&mut self, x: u8, y: u8, profile: &EntityProfile) {
        if profile.incorporeal {
            return;
        }
        let slot = self.slot_mut(x, y);
        slot.current_size = slot.current_size.saturating_add(profile.size);
    }

    pub fn vacate(&mut self, x: u8, y: u8, profile: &EntityProfile) {
        if profile.incorporeal {
            return;
        }
        let slot = self.slot_mut(x, y);
        slot.current_size = slot.current_size.saturating_sub(profile.size);
    }

    pub fn can_occupy(&self, x: u8, y: u8, profile: &EntityProfile) -> bool {
        matches!(
            compose(self.slot(x, y), profile),
            super::laws::Composition::Allowed { .. }
        )
    }
}

fn default_max_size(medium: &str) -> u8 {
    match medium {
        "water" => 2,
        "underground" => 1,
        "canopy" => 2,
        _ => 3,
    }
}
