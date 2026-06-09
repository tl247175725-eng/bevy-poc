pub mod causality;
pub mod composition;
pub mod laws;
pub mod profile;

use std::collections::HashMap;

pub use causality::{CausalEvent, CausalStorage};
pub use composition::{CellComposition, CellSlot};
pub use laws::{
    compose, perceive, transform, traverse, Composition, Perception, TransformAction,
    Transformation, Traversal,
};
pub use profile::{ChannelDef, DriveBehavior, DriveDef, EntityProfile, Medium};

use crate::card_def::CardDef;
use crate::spatial_index::EntityId;
use crate::terrain::terrain_at;
use crate::world_state::{Entity, WorldState};

/// Causal abstraction engine — all laws operate on precomputed `EntityProfile`.
pub struct AxiomEngine;

impl AxiomEngine {
    pub fn build_profile(
        entity_id: EntityId,
        type_name: &str,
        tags: &[String],
        hp: i32,
        world: &WorldState,
        x: u8,
        y: u8,
    ) -> EntityProfile {
        let terrain = terrain_at(world, x, y);
        let current_medium = profile::medium_for_cell(terrain);
        let native_medium = profile::parse_native_medium(tags, type_name);
        let mut bridges = profile::parse_bridges(tags, type_name);
        if tags.iter().any(|t| t == "volant") {
            if !bridges.iter().any(|(f, t)| f == "land" && t == "air") {
                bridges.push(("land".into(), "air".into()));
            }
        }

        EntityProfile {
            entity_id,
            type_name: type_name.to_string(),
            size: profile::parse_size(tags, type_name),
            incorporeal: tags.iter().any(|t| t == "capability.incorporeal"),
            native_medium,
            bridges,
            is_omnimedium: tags.iter().any(|t| t == "bridge:omnimedium"),
            channels: profile::parse_channels(tags, type_name),
            cross_perception: profile::parse_cross_perception(tags),
            visibility_mod: profile::parse_visibility_mod(tags),
            keen_eyed_mod: profile::parse_keen_eyed_mod(tags),
            energy: profile::parse_energy(tags, hp),
            efficiencies: profile::parse_efficiencies(tags),
            drives: profile::parse_drives(tags),
            move_speed: profile::parse_move_speed(tags),
            current_medium,
        }
    }

    pub fn build_profile_from_entity(entity: &Entity, def: &CardDef, world: &WorldState) -> EntityProfile {
        let mut tags = def.tags.clone();
        if !tags.iter().any(|t| t == &entity.type_name) {
            tags.push(entity.type_name.clone());
        }
        Self::build_profile(
            entity.id,
            &entity.type_name,
            &tags,
            def.hp,
            world,
            entity.x,
            entity.y,
        )
    }

    pub fn update_profile_dynamic(profile: &mut EntityProfile, entity: &Entity, def: &CardDef) {
        let mut tags = def.tags.clone();
        if !tags.iter().any(|t| t == &entity.type_name) {
            tags.push(entity.type_name.clone());
        }
        profile.energy = profile::parse_energy(&tags, entity.hp.max(0));
    }

    pub fn compose(cell: &CellSlot, incoming: &EntityProfile) -> Composition {
        laws::compose(cell, incoming)
    }

    pub fn traverse(profile: &EntityProfile, from: &Medium, to: &Medium) -> Traversal {
        laws::traverse(profile, from, to)
    }

    pub fn perceive(
        observer: &EntityProfile,
        target: &EntityProfile,
        distance: u8,
        observer_medium_conduction: &[(String, f32)],
        target_medium_conduction: &[(String, f32)],
    ) -> Perception {
        laws::perceive(
            observer,
            target,
            distance,
            observer_medium_conduction,
            target_medium_conduction,
        )
    }

    pub fn transform(
        source: &EntityProfile,
        target: &EntityProfile,
        action: TransformAction,
    ) -> Transformation {
        laws::transform(source, target, action)
    }

    pub fn trace(storage: &mut CausalStorage, event: CausalEvent) {
        storage.push(event);
    }

    pub fn default_medium_conductions() -> HashMap<Medium, Vec<(String, f32)>> {
        let mut m = HashMap::new();
        m.insert(
            "water".into(),
            vec![
                ("visual".into(), 0.5),
                ("olfactory".into(), 1.2),
                ("auditory".into(), 1.5),
            ],
        );
        m.insert(
            "land".into(),
            vec![
                ("visual".into(), 1.0),
                ("olfactory".into(), 1.0),
                ("auditory".into(), 1.0),
            ],
        );
        m.insert(
            "air".into(),
            vec![
                ("visual".into(), 1.5),
                ("olfactory".into(), 0.3),
                ("auditory".into(), 0.5),
            ],
        );
        m.insert(
            "underground".into(),
            vec![
                ("visual".into(), 0.0),
                ("olfactory".into(), 0.8),
                ("auditory".into(), 1.2),
            ],
        );
        m
    }
}
