//! Simulation events → log + world FX queue.

use bevy::prelude::*;

use crate::sim_clock::SimClock;
use crate::spatial_index::EntityId;
use std::collections::HashMap;

use crate::world_rules::card_has_tag;
use crate::world_state::{EcologyState, WorldState};

/// Move animation request — queued by `move_entity`, consumed by render layer.
#[derive(Debug, Clone, Event)]
pub struct MoveAnimEvent {
    pub entity_id: EntityId,
    pub from_x: u8,
    pub from_y: u8,
    pub to_x: u8,
    pub to_y: u8,
    pub duration_per_step: f32,
}

/// All move animations from the last tick have finished playing.
#[derive(Debug, Clone, Event)]
pub struct MoveAnimationsComplete;

/// Tracks in-flight move tweens so sim ticks wait for visuals.
#[derive(Resource, Default)]
pub struct MoveAnimPlayback {
    pub in_progress: bool,
    pub pending_completions: u32,
}

impl MoveAnimPlayback {
    pub fn begin_batch(&mut self, count: u32) {
        if count > 0 {
            self.in_progress = true;
            self.pending_completions = count;
        }
    }

    pub fn note_completion(&mut self) -> bool {
        if self.pending_completions > 0 {
            self.pending_completions -= 1;
        }
        if self.pending_completions == 0 {
            self.in_progress = false;
            true
        } else {
            false
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SimEvent {
    Spawn {
        entity_id: crate::spatial_index::EntityId,
        type_name: String,
        x: u8,
        y: u8,
        is_corpse: bool,
    },
    Despawn {
        type_name: String,
        x: u8,
        y: u8,
    },
    Move {
        entity_id: crate::spatial_index::EntityId,
        type_name: String,
        from: (u8, u8),
        to: (u8, u8),
    },
    Kill {
        predator: String,
        prey: String,
        x: u8,
        y: u8,
    },
    Hunt { predator: String, prey: String, x: u8, y: u8 },
    Reproduce { species: String, x: u8, y: u8 },
    Death { name: String, x: u8, y: u8 },
    Migrate { name: String, from: (u8, u8), to: (u8, u8) },
    Harvest { product: String, x: u8, y: u8 },
    Impact { source: String, target: String, x: u8, y: u8 },
    Generic(String),
}

/// Marker entity carrying [`SimStats`] for BRP `bevy/query` (resources are not queryable in 0.15).
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct SimStatsBrpMarker;

/// Core sim metrics exposed to BRP (WorldState itself is not Reflect-safe).
#[derive(Resource, Component, Reflect, Default, Clone)]
#[reflect(Resource, Component)]
pub struct SimStats {
    pub entity_count: usize,
    pub tick_count: u64,
    pub herbivore_count: usize,
    pub predator_count: usize,
    pub deaths: u64,
    /// Active `(type, ecology_state)` counts, e.g. `"sheep:Fleeing×3"`.
    pub state_breakdown: Vec<String>,
    /// Per-type population, e.g. `"sheep:15"`.
    pub top_entities: Vec<String>,
    /// Hunt/kill/harvest/reproduce events in the last sim tick.
    pub interactions_this_tick: u32,
}

pub fn setup_sim_stats_brp(
    mut commands: Commands,
    sim: Res<crate::grid_render::SimWorld>,
    mut stats: ResMut<SimStats>,
) {
    sync_sim_stats(&sim.0, &mut stats);
    commands.spawn((SimStatsBrpMarker, stats.clone()));
}

pub fn mirror_sim_stats_brp(stats: Res<SimStats>, mut q: Query<&mut SimStats, With<SimStatsBrpMarker>>) {
    if let Ok(mut mirror) = q.get_single_mut() {
        *mirror = stats.clone();
    }
}

fn ecology_state_label(state: EcologyState) -> &'static str {
    match state {
        EcologyState::Idle => "Idle",
        EcologyState::SeekingFood => "SeekingFood",
        EcologyState::Fleeing => "Fleeing",
        EcologyState::Hunting => "Hunting",
        EcologyState::Patrolling => "Patrolling",
        EcologyState::Burrowed => "Burrowed",
        EcologyState::InDen => "InDen",
        EcologyState::Scavenging => "Scavenging",
        EcologyState::Wandering => "Wandering",
    }
}

pub fn count_tick_interactions(events: &[SimEvent]) -> u32 {
    events
        .iter()
        .filter(|e| {
            matches!(
                e,
                SimEvent::Kill { .. }
                    | SimEvent::Hunt { .. }
                    | SimEvent::Reproduce { .. }
                    | SimEvent::Harvest { .. }
            )
        })
        .count() as u32
}

pub fn sync_sim_stats(world: &WorldState, stats: &mut SimStats) {
    stats.entity_count = world.entities.len();
    stats.tick_count = world.tick_count;
    stats.herbivore_count = world
        .entities
        .values()
        .filter(|e| {
            world
                .card_defs
                .get(&e.type_name)
                .is_some_and(|d| card_has_tag(d, "herbivore") || card_has_tag(d, "omnivore.small"))
        })
        .count();
    stats.predator_count = world
        .entities
        .values()
        .filter(|e| {
            world.card_defs.get(&e.type_name).is_some_and(|d| {
                card_has_tag(d, "predator") || card_has_tag(d, "mesopredator")
            })
        })
        .count();

    let mut state_counts: HashMap<String, usize> = HashMap::new();
    let mut type_counts: HashMap<String, usize> = HashMap::new();
    for e in world.entities.values() {
        *type_counts.entry(e.type_name.clone()).or_default() += 1;
        if e.ecology_state != EcologyState::Idle {
            let key = format!(
                "{}:{}",
                e.type_name,
                ecology_state_label(e.ecology_state)
            );
            *state_counts.entry(key).or_default() += 1;
        }
    }

    let mut state_breakdown: Vec<String> = state_counts
        .iter()
        .map(|(key, count)| format!("{key}×{count}"))
        .collect();
    state_breakdown.sort();
    stats.state_breakdown = state_breakdown;

    let mut top_entities: Vec<String> = type_counts
        .iter()
        .map(|(type_name, count)| format!("{type_name}:{count}"))
        .collect();
    top_entities.sort();
    stats.top_entities = top_entities;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world_state::empty_world;

    #[test]
    fn state_breakdown_lists_active_ecology_states() {
        let mut world = empty_world();
        let sheep_a = world.spawn("sheep", 5, 5);
        let sheep_b = world.spawn("sheep", 7, 5);
        let wolf = world.spawn("wolf", 6, 6);
        world.entities.get_mut(&sheep_a).unwrap().ecology_state = EcologyState::Fleeing;
        world.entities.get_mut(&sheep_b).unwrap().ecology_state = EcologyState::SeekingFood;
        world.entities.get_mut(&wolf).unwrap().ecology_state = EcologyState::Hunting;

        let mut stats = SimStats::default();
        sync_sim_stats(&world, &mut stats);

        assert!(stats.state_breakdown.iter().any(|s| s == "sheep:Fleeing×1"));
        assert!(stats.state_breakdown.iter().any(|s| s == "sheep:SeekingFood×1"));
        assert!(stats.state_breakdown.iter().any(|s| s == "wolf:Hunting×1"));
        assert!(stats.top_entities.iter().any(|s| s == "sheep:2"));
    }
}

#[derive(Resource, Default)]
pub struct SimEventQueue {
    pending: Vec<SimEvent>,
}

impl SimEventQueue {
    pub fn push(&mut self, event: SimEvent) {
        self.pending.push(event);
    }

    pub fn pending_len(&self) -> usize {
        self.pending.len()
    }

    pub fn drain(&mut self) -> Vec<SimEvent> {
        std::mem::take(&mut self.pending)
    }
}

#[derive(Resource, Default)]
pub struct WorldFxQueue {
    pending: Vec<FxMessage>,
}

#[derive(Debug, Clone)]
pub struct FxMessage {
    pub text: String,
    pub x: u8,
    pub y: u8,
}

impl WorldFxQueue {
    pub fn push(&mut self, msg: FxMessage) {
        self.pending.push(msg);
    }

    pub fn drain(&mut self) -> Vec<FxMessage> {
        std::mem::take(&mut self.pending)
    }
}

pub fn drain_sim_events(
    mut queue: ResMut<SimEventQueue>,
    mut fx: ResMut<WorldFxQueue>,
    mut clock: ResMut<SimClock>,
    mut stats: ResMut<crate::session_report::TickStats>,
    mut sim_stats: ResMut<SimStats>,
) {
    for event in queue.drain() {
        if matches!(event, SimEvent::Death { .. }) {
            sim_stats.deaths += 1;
        }
        stats.record_sim_event(&event);
        let (line, fx_msg) = format_event(&event);
        if !line.is_empty() {
            clock.push_log(line);
        }
        if let Some(msg) = fx_msg {
            fx.push(msg);
        }
    }
}

fn format_event(event: &SimEvent) -> (String, Option<FxMessage>) {
    match event {
        SimEvent::Spawn { is_corpse: true, type_name, x, y, .. } => (
            format!("尸体出现：{type_name} @ ({x},{y})"),
            None,
        ),
        SimEvent::Spawn { .. } | SimEvent::Despawn { .. } | SimEvent::Move { .. } => {
            (String::new(), None)
        }
        SimEvent::Kill { predator, prey, x, y } => (
            format!("{predator}捕杀{prey}"),
            Some(FxMessage {
                text: "捕猎".into(),
                x: *x,
                y: *y,
            }),
        ),
        SimEvent::Hunt { predator, prey, x, y } => (
            format!("{predator} 捕猎 {prey} @ ({x},{y})"),
            Some(FxMessage {
                text: "捕猎".into(),
                x: *x,
                y: *y,
            }),
        ),
        SimEvent::Reproduce { species, x, y } => (
            format!("{species} 繁殖 @ ({x},{y})"),
            None,
        ),
        SimEvent::Death { name, x, y } => (
            format!("{name} 死亡 @ ({x},{y})"),
            Some(FxMessage {
                text: "×".into(),
                x: *x,
                y: *y,
            }),
        ),
        SimEvent::Migrate { name, from, to } => (
            format!("{name} 迁移 ({},{}) → ({},{})", from.0, from.1, to.0, to.1),
            None,
        ),
        SimEvent::Harvest { product, x, y } => (
            format!("采收 {product} @ ({x},{y})"),
            Some(FxMessage {
                text: product.clone(),
                x: *x,
                y: *y,
            }),
        ),
        SimEvent::Impact { source, target, x, y } => (
            format!("{source} 砸 {target} @ ({x},{y})"),
            Some(FxMessage {
                text: "砸".into(),
                x: *x,
                y: *y,
            }),
        ),
        SimEvent::Generic(s) => (s.clone(), None),
    }
}

pub fn emit_hunt(
    queue: &mut SimEventQueue,
    predator: &str,
    prey: &str,
    x: u8,
    y: u8,
) {
    queue.push(SimEvent::Hunt {
        predator: predator.to_string(),
        prey: prey.to_string(),
        x,
        y,
    });
}
