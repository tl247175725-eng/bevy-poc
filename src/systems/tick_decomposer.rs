//! Dung production, decomposer spawn, and fertility cycling.

use crate::ecology_log::{card_display_name, eco_log};
use crate::world_rules::{card_has_tag, chebyshev_distance, ecology_was_fed_today, is_being};
use crate::world_state::WorldState;

pub fn upgrade_cell_fertility(world: &mut WorldState, x: u8, y: u8) {
    let slot = world.cell_composition.slot_mut(x, y);
    if slot.tags.iter().any(|t| t == "fertility:high") {
        return;
    }
    if let Some(idx) = slot.tags.iter().position(|t| t == "fertility:low") {
        slot.tags[idx] = "fertility:normal".into();
        return;
    }
    if let Some(idx) = slot.tags.iter().position(|t| t == "fertility:normal") {
        slot.tags[idx] = "fertility:high".into();
        return;
    }
    slot.tags.push("fertility:normal".into());
}

fn cell_has_soil_tag(world: &WorldState, x: u8, y: u8, tag: &str) -> bool {
    world
        .cell_composition
        .slot(x, y)
        .tags
        .iter()
        .any(|t| t == tag)
}

/// Called at day rollover before `fed_today` is cleared.
pub fn tick_dung_from_fed_animals(world: &mut WorldState) {
    let fed: Vec<(u8, u8, String)> = world
        .entities
        .values()
        .filter(|e| !e.is_corpse)
        .filter_map(|e| {
            let def = world.card_defs.get(&e.type_name)?;
            if !is_being(def) || !ecology_was_fed_today(e, def) {
                return None;
            }
            Some((e.x, e.y, e.type_name.clone()))
        })
        .collect();
    for (x, y, type_name) in fed {
        if world.has_tag_at(x, y, "fertilizer") {
            continue;
        }
        world.spawn("dung", x, y);
        eco_log(
            world,
            format!("{} 排泄 → 粪堆", card_display_name(world, &type_name)),
        );
    }
}

pub fn tick_decomposer(world: &mut WorldState) {
    spawn_decomposers(world);
    decompose_dung(world);
}

fn spawn_decomposers(world: &mut WorldState) {
    if world.tick_count % 40 != 0 {
        return;
    }
    for entity in world.entities.values() {
        let is_dung = world.card_defs.get(&entity.type_name)
            .is_some_and(|d| card_has_tag(d, "fertilizer"));
        if is_dung && !entity.is_corpse {
            let (x, y) = (entity.x, entity.y);
            if world.count_by_tag("decomposer") == 0
                && !world.has_tag_at(x, y, "decomposer")
            {
                world.spawn("dungBeetle", x, y);
                eco_log(world, "粪甲虫被粪堆吸引");
                break;
            }
        }
    }
    if world.count_type("earthworm") > 0 {
        return;
    }
    for y in 1..crate::world_rules::GRID_HEIGHT - 1 {
        for x in 1..crate::world_rules::GRID_WIDTH - 1 {
            if cell_has_soil_tag(world, x, y, "soil:rich")
                && !world.has_tag_at(x, y, "decomposer")
            {
                world.spawn("earthworm", x, y);
                eco_log(world, "蚯蚓出现在沃土");
                return;
            }
        }
    }
}

fn decompose_dung(world: &mut WorldState) {
    if world.tick_count % 15 != 0 {
        return;
    }
    fn is_decomposer(world: &WorldState, e: &crate::world_state::Entity) -> bool {
        world.card_defs.get(&e.type_name)
            .is_some_and(|d| card_has_tag(d, "decomposer"))
    }
    let decomposers: Vec<_> = world
        .entities
        .values()
        .filter(|e| is_decomposer(world, e) && e.in_ground)
        .map(|e| (e.id, e.x, e.y))
        .collect();

    for &(_decomposer_id, dx, dy) in &decomposers {
        let target = world
            .entities
            .values()
            .find(|e| {
                world.card_defs.get(&e.type_name)
                    .is_some_and(|d| card_has_tag(d, "fertilizer"))
                    && !e.is_corpse
                    && chebyshev_distance(dx, dy, e.x, e.y) <= 1
            })
            .map(|e| e.id);
        let Some(dung_id) = target else {
            continue;
        };
        let (dx, dy, remove) = {
            let dung = world.entities.get_mut(&dung_id).expect("dung");
            dung.hp = (dung.hp - 1).max(0);
            (dung.x, dung.y, dung.hp <= 0)
        };
        if remove {
            world.remove_entity(dung_id);
            upgrade_cell_fertility(world, dx, dy);
            eco_log(world, "粪堆分解 → 地块肥力提升");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world_state::empty_world;

    #[test]
    fn dung_decomposition_upgrades_fertility() {
        let mut world = empty_world();
        world.ensure_map_ecology();
        let x = 10u8;
        let y = 10u8;
        world.cell_composition.slot_mut(x, y).tags = smallvec::smallvec![
            "soil:dry".into(),
            "fertility:low".into()
        ];
        let dung = world.spawn("dung", x, y);
        world.spawn("dungBeetle", x, y);
        world.entities.get_mut(&dung).unwrap().hp = 1;
        decompose_dung(&mut world);
        assert!(!world.entities.contains_key(&dung));
        let tags = &world.cell_composition.slot(x, y).tags;
        assert!(tags.iter().any(|t| t == "fertility:normal"));
    }

    #[test]
    fn fed_sheep_leaves_dung_at_day_end() {
        let mut world = empty_world();
        let sheep = world.spawn("sheep", 8, 8);
        let def = world.card_defs.get("sheep").cloned().unwrap();
        if let Some(e) = world.entities.get_mut(&sheep) {
            e.fed_today = true;
            e.fed = true;
        }
        tick_dung_from_fed_animals(&mut world);
        assert!(world.count_type("dung") > 0);
        let _ = def;
    }
}
