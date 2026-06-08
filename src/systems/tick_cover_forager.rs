use crate::card_def::CardDef;
use crate::game_constants::{FIELD_MOUSE_REPRODUCE_MIN_MICRO, RABBIT_WOLF_FEAR_DIST};
use crate::spatial_index::EntityId;
use crate::systems::movement::{flee_from, move_toward, nearest_of, wander};
use crate::world_rules::{mark_ecology_fed, wolves_near};
use crate::world_state::{EcologyState, WorldState};

pub fn tick_cover_forager(world: &mut WorldState, id: EntityId, def: &CardDef, _delta: f32) {
    let type_name = def.type_name.clone();
    let (x, y, in_burrow) = {
        let e = &world.entities[&id];
        (e.x, e.y, e.in_burrow)
    };
    if in_burrow {
        world.entities.get_mut(&id).unwrap().ecology_state = EcologyState::Burrowed;
        return;
    }

    let wolves = wolves_near(world, x, y, RABBIT_WOLF_FEAR_DIST);
    let _ = crate::rule_index::dual_track_flee_if_alone(crate::rule_index::rule_index(), world, id);
    if !wolves.is_empty() {
        if world.has_tag_at(x, y, "bush") {
            world.entities.get_mut(&id).unwrap().in_burrow = true;
            world.entities.get_mut(&id).unwrap().ecology_state = EcologyState::Burrowed;
            return;
        }
        if let Some((wid, _)) = nearest_of(world, x, y, &wolves) {
            let (wx, wy) = world.spatial_index.position(wid).unwrap_or((x, y));
            flee_from(world, id, x, y, wx, wy);
            return;
        }
    }

    if type_name == "fieldMouse" || type_name == "fieldMousePup" {
        tick_field_mouse(world, id, x, y, def);
    } else if type_name == "bambooRat" {
        tick_bamboo_rat(world, id, x, y, def);
    }
}

fn tick_field_mouse(world: &mut WorldState, id: EntityId, x: u8, y: u8, def: &CardDef) {
    let micro = world.bush_microfauna.get(&(x, y)).copied().unwrap_or(0);
    if world.has_tag_at(x, y, "bush") && micro >= FIELD_MOUSE_REPRODUCE_MIN_MICRO {
        if let Some(mouse) = world.entities.get_mut(&id) {
            mark_ecology_fed(mouse, def);
            mouse.ecology_state = EcologyState::SeekingFood;
        }
        world
            .bush_microfauna
            .entry((x, y))
            .and_modify(|m| *m = (*m - 1).max(0));
        return;
    }
    let bush = world
        .spatial_index
        .query_near(x, y, "bush", 4)
        .first()
        .and_then(|bid| world.spatial_index.position(*bid));
    if let Some((bx, by)) = bush {
        move_toward(world, id, x, y, bx, by);
    } else {
        wander(world, id, x, y, world.tick_count);
        world.entities.get_mut(&id).unwrap().ecology_state = EcologyState::Idle;
    }
}

fn tick_bamboo_rat(world: &mut WorldState, id: EntityId, x: u8, y: u8, def: &CardDef) {
    if world.has_tag_at(x, y, "underground") {
        if let Some(rat) = world.entities.get_mut(&id) {
            mark_ecology_fed(rat, def);
            rat.ecology_state = EcologyState::SeekingFood;
        }
        return;
    }
    let grass = world.spatial_index.query_near(x, y, "grass", 2).first().copied();
    if let Some(grass_id) = grass {
        let (gx, gy) = world.spatial_index.position(grass_id).unwrap_or((x, y));
        move_toward(world, id, x, y, gx, gy);
    } else {
        wander(world, id, x, y, world.tick_count);
    }
}
