use crate::ecology_log::{card_display_name, eco_log};
use crate::game_constants::TICKS_PER_DAY;
use crate::interaction::finalize_prey_kill;
use crate::spatial_index::EntityId;
use crate::world_rules::{ecology_was_fed_today, is_being, parse_max_starve};
use crate::world_state::WorldState;

pub fn tick_starvation(world: &mut WorldState) {
    if world.tick_count == 0 {
        return;
    }
    let day = world.tick_count / TICKS_PER_DAY;
    if day <= world.last_processed_day {
        return;
    }
    world.last_processed_day = day;

    let beings: Vec<(EntityId, String)> = world
        .entities
        .values()
        .filter(|e| !e.is_corpse)
        .filter_map(|e| {
            world
                .card_defs
                .get(&e.type_name)
                .filter(|d| is_being(d))
                .map(|_| (e.id, e.type_name.clone()))
        })
        .collect();

    for (id, type_name) in beings {
        let Some(def) = world.card_defs.get(&type_name).cloned() else {
            continue;
        };
        let fed = world
            .entities
            .get(&id)
            .is_some_and(|e| ecology_was_fed_today(e, &def));
        if !fed {
            if let Some(e) = world.entities.get_mut(&id) {
                e.starve_days += 1;
            }
        }
        let starve_days = world.entities.get(&id).map(|e| e.starve_days).unwrap_or(0);
        let max_starve = parse_max_starve(&def);
        if starve_days >= max_starve {
            let name = world
                .entities
                .get(&id)
                .map(|e| card_display_name(world, &e.type_name))
                .unwrap_or_else(|| type_name.clone());
            eco_log(
                world,
                format!("{name} 饿死（{starve_days}/{max_starve} 天未进食）"),
            );
            finalize_prey_kill(world, id, None, "starvation");
            continue;
        }
        if let Some(e) = world.entities.get_mut(&id) {
            e.fed_today = false;
            e.fed = false;
            e.meat_fed_today = 0;
            e.scavenge_today = 0;
        }
    }
}
