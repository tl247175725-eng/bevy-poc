use crate::spatial_index::EntityId;
use crate::world_state::WorldState;

pub fn tick_entity(world: &mut WorldState, id: EntityId, delta: f32) {
    if world
        .entities
        .get(&id)
        .is_some_and(|e| e.type_name == "player")
    {
        crate::player::tick_player_world(world, id, delta);
        return;
    }
    crate::event_registry::EventRegistry::tick_entity_ecology(world, id, delta);
}
