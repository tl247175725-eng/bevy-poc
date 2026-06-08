//! Survival tasks — forage / eat.

use crate::spatial_index::EntityId;
use crate::world_state::WorldState;

use super::brain_world::sorted_by_distance;
use super::state::PlayerMind;

pub fn should_forage(mind: &PlayerMind) -> bool {
    super::brain_tags::has_tag(mind, "hungry")
        && mind.affordances.contains_key("forage")
}

pub fn should_not_forage_when_full(mind: &PlayerMind) -> bool {
    !super::brain_tags::has_tag(mind, "hungry")
}

pub fn satisfy_hunger(world: &mut WorldState, player_id: EntityId, mind: &mut PlayerMind) -> bool {
    if !should_forage(mind) {
        return false;
    }
    let (px, py) = {
        let e = &world.entities[&player_id];
        (e.x, e.y)
    };
    if let Some(berry) = sorted_by_distance(world, px, py, "berry").first() {
        world.remove_entity(*berry);
        mind.hunger = (mind.hunger - 35.0).max(0.0);
        mind.goal_text = "觅食完成".into();
        return true;
    }
    if let Some(bush) = sorted_by_distance(world, px, py, "bush").first() {
        mind.goal_text = "前往灌木觅食".into();
        return true;
    }
    false
}

pub fn hunger_priority_when_starving(mind: &PlayerMind) -> bool {
    super::brain_tags::has_tag(mind, "food_seek")
}
