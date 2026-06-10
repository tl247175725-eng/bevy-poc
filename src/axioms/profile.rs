use crate::spatial_index::EntityId;
use smallvec::SmallVec;

use super::laws::TransformAction;

pub type Medium = String;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SocialStructure {
    Flock,
    Pack,
    Herd,
    None,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DriveBehavior {
    Seek,
    Flee,
    Flock,
    Hide,
    ReturnDen,
    Scavenge,
    Wander,
    Idle,
}

#[derive(Clone, Debug)]
pub struct DriveDef {
    pub behavior: DriveBehavior,
    pub target_tag: String,
    pub range: u8,
    pub priority: u8,
    pub condition_fed: bool,
}

/// Precomputed from card tags once at spawn; systems read profiles, not raw tag vecs.
#[derive(Clone, Debug)]
pub struct EntityProfile {
    pub entity_id: EntityId,
    pub type_name: String,

    pub size: u8,
    pub incorporeal: bool,

    pub native_medium: Medium,
    pub bridges: SmallVec<[(Medium, Medium); 4]>,
    pub is_omnimedium: bool,

    pub channels: SmallVec<[ChannelDef; 4]>,
    pub cross_perception: SmallVec<[Medium; 2]>,
    pub visibility_mod: f32,
    pub keen_eyed_mod: f32,

    pub energy: u32,
    pub efficiencies: SmallVec<[(TransformAction, f32); 6]>,
    pub drives: SmallVec<[DriveDef; 6]>,

    /// Seconds per grid step for normal movement (always 0.25).
    pub move_speed: f32,
    /// Seconds per grid step when seeking prey or fleeing (`sprint:*` tag).
    pub sprint_speed: f32,

    pub current_medium: Medium,

    pub social_structure: SocialStructure,
    pub flock_range: u8,
    pub flock_max: u8,
    pub flock_alert_range: u8,
}

#[derive(Clone, Debug)]
pub struct ChannelDef {
    pub kind: String,
    pub range: u8,
}

impl Default for EntityProfile {
    fn default() -> Self {
        Self {
            entity_id: EntityId(0),
            type_name: String::new(),
            size: 1,
            incorporeal: false,
            native_medium: "land".into(),
            bridges: SmallVec::new(),
            is_omnimedium: false,
            channels: SmallVec::new(),
            cross_perception: SmallVec::new(),
            visibility_mod: 1.0,
            keen_eyed_mod: 1.0,
            energy: 0,
            efficiencies: SmallVec::new(),
            drives: SmallVec::new(),
            move_speed: 0.25,
            sprint_speed: 0.12,
            current_medium: "land".into(),
            social_structure: SocialStructure::None,
            flock_range: 0,
            flock_max: 1,
            flock_alert_range: 0,
        }
    }
}

pub struct FlockParams {
    pub social_structure: SocialStructure,
    pub range: u8,
    pub max: u8,
    pub alert_range: u8,
}

fn structure_defaults(structure: SocialStructure) -> FlockParams {
    match structure {
        SocialStructure::Flock => FlockParams {
            social_structure: structure,
            range: 4,
            max: 8,
            alert_range: 3,
        },
        SocialStructure::Pack => FlockParams {
            social_structure: structure,
            range: 10,
            max: 6,
            alert_range: 0,
        },
        SocialStructure::Herd => FlockParams {
            social_structure: structure,
            range: 6,
            max: 12,
            alert_range: 4,
        },
        SocialStructure::None => FlockParams {
            social_structure: structure,
            range: 0,
            max: 1,
            alert_range: 0,
        },
    }
}

pub fn parse_flock_params(tags: &[String]) -> FlockParams {
    let mut structure = SocialStructure::None;
    for t in tags {
        if let Some(name) = t.strip_prefix("social_structure:") {
            structure = match name {
                "flock" => SocialStructure::Flock,
                "pack" => SocialStructure::Pack,
                "herd" => SocialStructure::Herd,
                _ => SocialStructure::None,
            };
            break;
        }
    }

    if structure == SocialStructure::None {
        return structure_defaults(SocialStructure::None);
    }

    let defaults = structure_defaults(structure);
    let mut range = defaults.range;
    let mut max = defaults.max;
    let mut alert_range = defaults.alert_range;

    for t in tags {
        if let Some(v) = t.strip_prefix("flock_range:") {
            if let Ok(n) = v.parse::<u8>() {
                range = n;
            }
        } else if let Some(v) = t.strip_prefix("flock_max:") {
            if let Ok(n) = v.parse::<u8>() {
                max = n.max(1);
            }
        } else if let Some(v) = t.strip_prefix("flock_alert_range:") {
            if let Ok(n) = v.parse::<u8>() {
                alert_range = n;
            }
        }
    }

    FlockParams {
        social_structure: structure,
        range,
        max,
        alert_range,
    }
}

/// All cards walk at 0.25s per grid step.
pub fn parse_move_speed(_tags: &[String]) -> f32 {
    0.25
}

/// Sprint tier for seek/flee — `sprint:slow|normal|fast|burst`.
pub fn parse_sprint_speed(tags: &[String]) -> f32 {
    for t in tags {
        if let Some(name) = t.strip_prefix("sprint:") {
            return match name {
                "slow" => 0.18,
                "normal" => 0.12,
                "fast" => 0.08,
                "burst" => 0.05,
                _ => 0.12,
            };
        }
    }
    0.12
}

pub fn parse_drives(tags: &[String]) -> SmallVec<[DriveDef; 6]> {
    let mut out = SmallVec::new();
    for t in tags {
        let Some(rest) = t.strip_prefix("drive:") else {
            continue;
        };
        let (behavior_str, params) = match rest.split_once('(') {
            Some((b, p)) => (b, p.trim_end_matches(')')),
            None => (rest, ""),
        };
        let behavior = match behavior_str {
            "seek" => DriveBehavior::Seek,
            "flee" => DriveBehavior::Flee,
            "flock" => DriveBehavior::Flock,
            "hide" => DriveBehavior::Hide,
            "return_den" => DriveBehavior::ReturnDen,
            "scavenge" => DriveBehavior::Scavenge,
            _ => continue,
        };
        let target_tag = parse_tag_str_param(params, "target").unwrap_or_default();
        let range = parse_tag_u8_param(params, "range").unwrap_or(if behavior == DriveBehavior::Hide {
            4
        } else {
            0
        });
        let priority = parse_tag_u8_param(params, "priority").unwrap_or(1);
        let condition_fed = behavior == DriveBehavior::Seek;
        out.push(DriveDef {
            behavior,
            target_tag,
            range,
            priority,
            condition_fed,
        });
    }
    out
}

fn parse_tag_str_param(s: &str, key: &str) -> Option<String> {
    let needle = format!("{key}=");
    for part in s.split(',') {
        let part = part.trim();
        if let Some(v) = part.strip_prefix(&needle) {
            return Some(v.to_string());
        }
    }
    None
}

fn parse_tag_u8_param(s: &str, key: &str) -> Option<u8> {
    let needle = format!("{key}=");
    for part in s.split(',') {
        let part = part.trim();
        if let Some(v) = part.strip_prefix(&needle) {
            return v.parse().ok();
        }
    }
    None
}

pub fn parse_size(tags: &[String], type_name: &str) -> u8 {
    for t in tags {
        if let Some(n) = t.strip_prefix("size:") {
            if let Ok(v) = n.parse::<u8>() {
                return v.max(1);
            }
        }
    }
    if tags.iter().any(|t| t == "body.large") {
        return 3;
    }
    if tags.iter().any(|t| t == "body.medium") {
        return 1;
    }
    if tags.iter().any(|t| t == "body.small" || t == "body.tiny") {
        return 1;
    }
    1
}

pub fn parse_native_medium(tags: &[String], type_name: &str) -> Medium {
    for t in tags {
        if let Some(name) = t.strip_prefix("medium:") {
            return name.to_string();
        }
    }
    if tags.iter().any(|t| t == "aquatic") {
        return "water".into();
    }
    "land".into()
}

pub fn parse_bridges(tags: &[String], type_name: &str) -> SmallVec<[(Medium, Medium); 4]> {
    let mut bridges = SmallVec::new();
    for t in tags {
        if let Some(rest) = t.strip_prefix("bridge:") {
            if rest == "omnimedium" {
                continue;
            }
            if let Some((from, to)) = rest.split_once("->") {
                bridges.push((from.to_string(), to.to_string()));
            }
        }
    }
    bridges
}

pub fn parse_cross_perception(tags: &[String]) -> SmallVec<[Medium; 2]> {
    let mut out = SmallVec::new();
    for t in tags {
        if let Some(medium) = t.strip_prefix("bridge:perceive->") {
            out.push(medium.to_string());
        }
    }
    out
}

pub fn parse_channels(tags: &[String], type_name: &str) -> SmallVec<[ChannelDef; 4]> {
    let mut channels = SmallVec::new();
    for t in tags {
        if let Some(rest) = t.strip_prefix("perception:") {
            if rest.starts_with("keen_eyed") {
                continue;
            }
            if let Some(kind) = rest.split('(').next() {
                let range = parse_tag_f_param(rest, 'r').unwrap_or(1.0) as u8;
                channels.push(ChannelDef {
                    kind: kind.to_string(),
                    range: range.max(1),
                });
            }
        }
    }
    if channels.is_empty() && is_being_tagged(tags) {
        channels.push(ChannelDef {
            kind: "visual".into(),
            range: 6,
        });
    }
    if channels.is_empty() && tags.iter().any(|t| t == "aquatic") {
        channels.push(ChannelDef {
            kind: "visual".into(),
            range: 4,
        });
    }
    channels
}

fn is_being_tagged(tags: &[String]) -> bool {
    tags.iter().any(|t| t == "being" || t.starts_with("being."))
}

pub fn parse_visibility_mod(tags: &[String]) -> f32 {
    for t in tags {
        if t == "visibility:tiny" {
            return 0.1;
        }
        if t == "visibility:transparent" {
            return 0.0;
        }
        if let Some(rest) = t.strip_prefix("visibility:") {
            if let Some(m) = parse_tag_f_param(rest, 'm') {
                return m;
            }
        }
    }
    1.0
}

pub fn parse_keen_eyed_mod(tags: &[String]) -> f32 {
    for t in tags {
        if let Some(rest) = t.strip_prefix("perception:keen_eyed") {
            if let Some(m) = parse_tag_f_param(rest, 'm') {
                return m;
            }
        }
    }
    1.0
}

pub fn parse_energy(tags: &[String], hp: i32) -> u32 {
    for t in tags {
        if let Some(n) = t.strip_prefix("energy:") {
            if let Ok(v) = n.parse::<u32>() {
                return v;
            }
        }
    }
    if hp > 0 {
        hp as u32
    } else {
        1
    }
}

pub fn parse_efficiencies(tags: &[String]) -> SmallVec<[(TransformAction, f32); 6]> {
    let mut out = SmallVec::new();
    for t in tags {
        if let Some(rest) = t.strip_prefix("efficiency:") {
            if let Some((action_str, params)) = rest.split_once('(') {
                let action = TransformAction::from_tag(action_str);
                let m = parse_tag_f_param(params, 'm').unwrap_or(0.5);
                out.push((action, m));
            }
        }
    }
    out
}

fn parse_tag_f_param(s: &str, key: char) -> Option<f32> {
    let needle = format!("{key}=");
    for part in s.split(|c| c == '(' || c == ')') {
        if let Some(v) = part.strip_prefix(&needle) {
            return v.parse().ok();
        }
    }
    None
}

pub fn medium_for_cell(terrain: &str) -> Medium {
    if matches!(terrain, "river" | "ford" | "pool" | "dark_river_pool") {
        "water".into()
    } else {
        "land".into()
    }
}
