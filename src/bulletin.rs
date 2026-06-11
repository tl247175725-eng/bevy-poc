//! Global bulletin board — coarse zone hints updated every N ticks.

use std::collections::HashMap;

use crate::axioms::EntityProfile;
use crate::card_def::CardDef;
use crate::world_rules::{card_has_tag, chebyshev_distance, GRID_HEIGHT, GRID_WIDTH};
use crate::world_state::{Entity, WorldState};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Zone {
    pub center_x: u8,
    pub center_y: u8,
    pub radius: u8,
    pub intensity: u8,
}

#[derive(Debug, Clone, Default)]
pub struct BulletinBoard {
    pub channels: HashMap<String, Vec<Zone>>,
    pub last_update_tick: u64,
}

impl BulletinBoard {
    pub const UPDATE_INTERVAL: u64 = 50;
    const BLOCK_SIZE: u8 = 8;
    const DENSITY_THRESHOLD: usize = 2;
    const ZONE_RADIUS: u8 = 4;

    pub fn needs_update(&self, tick: u64) -> bool {
        tick == 0 || tick.saturating_sub(self.last_update_tick) >= Self::UPDATE_INTERVAL
    }

    pub fn update(&mut self, world: &WorldState) {
        self.channels.clear();
        let blocks_x = (GRID_WIDTH as usize + Self::BLOCK_SIZE as usize - 1) / Self::BLOCK_SIZE as usize;
        let blocks_y =
            (GRID_HEIGHT as usize + Self::BLOCK_SIZE as usize - 1) / Self::BLOCK_SIZE as usize;

        let mut block_counts: HashMap<(usize, usize, &'static str), usize> = HashMap::new();

        for entity in world.entities.values() {
            if entity.in_den || entity.in_burrow {
                continue;
            }
            let Some(def) = world.card_defs.get(&entity.type_name) else {
                continue;
            };
            let bx = (entity.x as usize) / Self::BLOCK_SIZE as usize;
            let by = (entity.y as usize) / Self::BLOCK_SIZE as usize;
            for channel in entity_zone_channels(entity, def) {
                *block_counts.entry((bx, by, channel)).or_default() += 1;
            }
        }

        for &(px, py) in &world.pool_cells {
            let bx = (px as usize) / Self::BLOCK_SIZE as usize;
            let by = (py as usize) / Self::BLOCK_SIZE as usize;
            *block_counts.entry((bx, by, "water_zones")).or_default() += 1;
        }

        for by in 0..blocks_y {
            for bx in 0..blocks_x {
                let center_x = ((bx * Self::BLOCK_SIZE as usize) + Self::BLOCK_SIZE as usize / 2)
                    .min(GRID_WIDTH as usize - 1) as u8;
                let center_y = ((by * Self::BLOCK_SIZE as usize) + Self::BLOCK_SIZE as usize / 2)
                    .min(GRID_HEIGHT as usize - 1) as u8;

                for channel in CHANNEL_NAMES {
                    let count = block_counts.get(&(bx, by, channel)).copied().unwrap_or(0);
                    if count >= Self::DENSITY_THRESHOLD {
                        let intensity = ((count * 100) / (Self::DENSITY_THRESHOLD * 4))
                            .min(100)
                            .max(10) as u8;
                        self.channels
                            .entry(channel.to_string())
                            .or_default()
                            .push(Zone {
                                center_x,
                                center_y,
                                radius: Self::ZONE_RADIUS,
                                intensity,
                            });
                    }
                }
            }
        }

        self.last_update_tick = world.tick_count;
    }

    pub fn nearest_zone_center(
        &self,
        profile: &EntityProfile,
        channel: &str,
        x: u8,
        y: u8,
    ) -> Option<(u8, u8)> {
        if !profile.has_bulletin_access(channel) {
            return None;
        }
        let zones = self.channels.get(channel)?;
        zones
            .iter()
            .min_by_key(|z| {
                let dist = chebyshev_distance(x, y, z.center_x, z.center_y);
                (dist, 255u8 - z.intensity)
            })
            .map(|z| (z.center_x, z.center_y))
    }
}

const CHANNEL_NAMES: &[&str] = &[
    "predator_zones",
    "prey_zones",
    "food_zones",
    "water_zones",
    "corpse_zones",
    "shelter_zones",
];

pub fn seek_target_channel(target_tag: &str) -> &'static str {
    match target_tag {
        "foodSource" | "grass" => "food_zones",
        "herbivore" | "smallPrey" | "largePrey" | "smallHerbivore" => "prey_zones",
        "corpse" => "corpse_zones",
        "predator" | "mesopredator" | "tiger" => "predator_zones",
        _ => "food_zones",
    }
}

fn entity_zone_channels<'a>(entity: &Entity, def: &CardDef) -> Vec<&'a str> {
    let mut out = Vec::new();
    if entity.is_corpse || card_has_tag(def, "corpse") {
        out.push("corpse_zones");
    }
    if card_has_tag(def, "predator") || card_has_tag(def, "mesopredator") {
        out.push("predator_zones");
    }
    if card_has_tag(def, "herbivore")
        || card_has_tag(def, "smallPrey")
        || card_has_tag(def, "smallHerbivore")
        || card_has_tag(def, "largePrey")
        || card_has_tag(def, "omnivore.small")
    {
        out.push("prey_zones");
    }
    if card_has_tag(def, "foodSource") || card_has_tag(def, "grass") || entity.type_name == "grass"
    {
        out.push("food_zones");
    }
    if card_has_tag(def, "aquatic") || entity.type_name.contains("fish") {
        out.push("water_zones");
    }
    if matches!(
        entity.type_name.as_str(),
        "bush" | "hut" | "oak" | "pine" | "wolfDen" | "foxDen"
    ) || card_has_tag(def, "cover.small")
    {
        out.push("shelter_zones");
    }
    out
}

pub fn maybe_update(world: &mut WorldState) {
    let tick = world.tick_count;
    if !world.bulletin_board.needs_update(tick) {
        return;
    }
    let mut board = std::mem::take(&mut world.bulletin_board);
    board.update(world);
    world.bulletin_board = board;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spatial_index::EntityId;
    use crate::world_state::empty_world;

    #[test]
    fn bulletin_updates_every_fifty_ticks() {
        let board = BulletinBoard::default();
        assert!(board.needs_update(0));
        assert!(board.needs_update(50));
        assert!(!board.needs_update(25));
    }

    #[test]
    fn dense_food_cluster_posts_food_zone() {
        let mut world = empty_world();
        for i in 0..4u64 {
            world.spawn("grass", 10 + i as u8, 10);
        }
        world.tick_count = 50;
        let mut board = BulletinBoard::default();
        board.update(&world);
        assert!(board.channels.get("food_zones").is_some_and(|z| !z.is_empty()));
    }

    #[test]
    fn bulletin_food_access_enables_zone_seek() {
        let mut world = empty_world();
        for i in 0..4u64 {
            world.spawn("grass", 20 + i as u8, 12);
        }
        world.tick_count = 50;
        let mut board = BulletinBoard::default();
        board.update(&world);

        let sheep = world.spawn("sheep", 5, 5);
        let profile = world.entities[&sheep].profile.clone();
        assert!(profile.has_bulletin_access("food_zones"));
        let target = board.nearest_zone_center(&profile, "food_zones", 5, 5);
        assert!(target.is_some());
        let (tx, ty) = target.unwrap();
        assert!(chebyshev_distance(5, 5, tx, ty) > 6);
    }

    #[test]
    fn no_bulletin_tag_denies_channel_access() {
        let mut world = empty_world();
        let grass = world.spawn("grass", 5, 5);
        let profile = world.entities[&grass].profile.clone();
        assert!(!profile.has_bulletin_access("food_zones"));
        let _ = grass;
        let _ = EntityId(0);
    }
}
