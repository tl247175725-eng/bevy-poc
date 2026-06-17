use crate::spatial_index::EntityId;
use crate::world_state::WorldState;

pub fn tick_entity(world: &mut WorldState, id: EntityId, delta: f32) {
    if world
        .entities
        .get(&id)
        .is_some_and(|e| {
            world.card_defs.get(&e.type_name)
                .is_some_and(|def| crate::world_rules::card_has_tag(def, "role:player"))
        })
    {
        crate::player::tick_player_world(world, id, delta);
        return;
    }
    crate::event_registry::EventRegistry::tick_entity_ecology(world, id, delta);
}
