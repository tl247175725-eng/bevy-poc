use smallvec::SmallVec;

use super::composition::CellSlot;
use super::profile::{EntityProfile, Medium, SocialStructure};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransformAction {
    Eat,
    Kill,
    Spawn,
    Harvest,
}

impl TransformAction {
    pub fn from_tag(s: &str) -> Self {
        match s {
            "kill" => Self::Kill,
            "spawn" => Self::Spawn,
            "harvest" => Self::Harvest,
            _ => Self::Eat,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Composition {
    Allowed { remaining: u8 },
    Denied { current: u8, max: u8 },
}

fn all_same_flock(cell: &CellSlot, incoming: &EntityProfile) -> bool {
    cell.is_flock
        && cell.flock_type == incoming.type_name
        && incoming.social_structure != SocialStructure::None
}

pub fn compose(cell: &CellSlot, incoming: &EntityProfile) -> Composition {
    if incoming.incorporeal {
        return Composition::Allowed { remaining: u8::MAX };
    }

    if incoming.type_name.ends_with("Corpse") {
        return Composition::Allowed { remaining: u8::MAX };
    }

    if cell.has_only_corpses() {
        return Composition::Allowed { remaining: 0 };
    }

    if cell.living_count > 0 {
        if all_same_flock(cell, incoming) {
            return Composition::Allowed { remaining: 0 };
        }
        return Composition::Denied {
            current: cell.living_count,
            max: 1,
        };
    }

    Composition::Allowed { remaining: 0 }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Traversal {
    Allowed,
    Denied {
        from: Medium,
        to: Medium,
        missing: String,
    },
}

pub fn traverse(profile: &EntityProfile, from: &Medium, to: &Medium) -> Traversal {
    if from == to {
        return Traversal::Allowed;
    }
    if profile.is_omnimedium {
        return Traversal::Allowed;
    }
    if &profile.native_medium == to {
        return Traversal::Allowed;
    }
    for (bf, bt) in &profile.bridges {
        if bf == from && bt == to {
            return Traversal::Allowed;
        }
    }
    Traversal::Denied {
        from: from.clone(),
        to: to.clone(),
        missing: format!("bridge:{from}->{to}"),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Perception {
    Detected {
        channels: SmallVec<[PerceptionChannel; 4]>,
    },
    Undetected,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PerceptionChannel {
    pub kind: String,
    pub effective_range: u8,
}

pub fn perceive(
    observer: &EntityProfile,
    target: &EntityProfile,
    distance: u8,
    observer_medium_conduction: &[(String, f32)],
    target_medium_conduction: &[(String, f32)],
) -> Perception {
    if observer.current_medium != target.current_medium
        && !observer.cross_perception.contains(&target.current_medium)
    {
        return Perception::Undetected;
    }

    let mut detected = SmallVec::new();
    for ch in &observer.channels {
        let obs_cond = observer_medium_conduction
            .iter()
            .find(|(k, _)| k == &ch.kind)
            .map(|(_, v)| *v)
            .unwrap_or(1.0);
        let tgt_cond = target_medium_conduction
            .iter()
            .find(|(k, _)| k == &ch.kind)
            .map(|(_, v)| *v)
            .unwrap_or(1.0);
        let conduction = obs_cond.min(tgt_cond);

        let effective = (ch.range as f32 * observer.keen_eyed_mod * target.visibility_mod * conduction)
            as u8;
        if effective >= distance {
            detected.push(PerceptionChannel {
                kind: ch.kind.clone(),
                effective_range: effective,
            });
        }
    }
    if detected.is_empty() {
        Perception::Undetected
    } else {
        Perception::Detected { channels: detected }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transformation {
    pub energy_drawn: u32,
    pub energy_received: u32,
    pub energy_lost: u32,
}

pub fn transform(
    source: &EntityProfile,
    target: &EntityProfile,
    action: TransformAction,
) -> Transformation {
    let drawn = source.energy;
    let efficiency = target
        .efficiencies
        .iter()
        .find(|(act, _)| *act == action)
        .map(|(_, m)| *m)
        .unwrap_or(0.5);
    let received = ((drawn as f32) * efficiency) as u32;
    let lost = drawn.saturating_sub(received);
    Transformation {
        energy_drawn: drawn,
        energy_received: received,
        energy_lost: lost,
    }
}
