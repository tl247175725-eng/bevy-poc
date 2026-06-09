//! Simulation events → log + world FX queue.

use bevy::prelude::*;

use crate::sim_clock::SimClock;
use crate::spatial_index::EntityId;

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
) {
    for event in queue.drain() {
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
