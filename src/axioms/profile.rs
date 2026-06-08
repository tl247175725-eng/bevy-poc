use crate::spatial_index::EntityId;
use smallvec::SmallVec;

use super::laws::TransformAction;

pub type Medium = String;

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

    pub current_medium: Medium,
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
            current_medium: "land".into(),
        }
    }
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
        return 2;
    }
    if tags.iter().any(|t| t == "body.small" || t == "body.tiny") {
        return 1;
    }
    if matches!(type_name, "waterBuffalo") {
        return 3;
    }
    1
}

pub fn parse_native_medium(tags: &[String], type_name: &str) -> Medium {
    for t in tags {
        if let Some(name) = t.strip_prefix("medium:") {
            return name.to_string();
        }
    }
    if tags.iter().any(|t| t == "aquatic") || type_name == "algae" {
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
    if matches!(type_name, "waterBuffalo" | "waterBuffaloCalf") {
        if !bridges.iter().any(|(f, t)| f == "land" && t == "water") {
            bridges.push(("land".into(), "water".into()));
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
    if channels.is_empty() && matches!(type_name, "fish" | "waterBug") {
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
