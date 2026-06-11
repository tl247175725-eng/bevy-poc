use crate::spatial_index::EntityId;
use smallvec::SmallVec;

use super::laws::TransformAction;

pub type Medium = String;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Height {
    Flat,
    Low,
    Medium,
    High,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SocialStructure {
    Flock,
    Pack,
    Herd,
    None,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NeedCurve {
    Steep,
    Flat,
    Sharp,
}

#[derive(Clone, Debug)]
pub struct NeedState {
    pub kind: String,
    pub current: f32,
    pub decay_rate: f32,
    pub curve: NeedCurve,
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
    /// Need channel this drive satisfies — used by utility scoring.
    pub need_kind: String,
}

/// Precomputed from card tags once at spawn; systems read profiles, not raw tag vecs.
#[derive(Clone, Debug)]
pub struct EntityProfile {
    pub entity_id: EntityId,
    pub type_name: String,

    pub size: u8,
    pub height: Height,
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
    /// Runtime need urgency — decayed each reactive tick.
    pub needs: SmallVec<[NeedState; 4]>,

    /// Seconds per grid step for normal movement (always 0.25).
    pub move_speed: f32,
    /// Seconds per grid step when seeking prey or fleeing (`sprint:*` tag).
    pub sprint_speed: f32,

    pub current_medium: Medium,

    pub social_structure: SocialStructure,
    pub flock_range: u8,
    pub flock_max: u8,
    pub flock_alert_range: u8,

    /// Parsed from `bulletin:*` tags — controls global zone channel access.
    pub bulletin_channels: SmallVec<[String; 6]>,
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
            height: Height::Medium,
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
            needs: SmallVec::new(),
            move_speed: 0.25,
            sprint_speed: 0.12,
            current_medium: "land".into(),
            social_structure: SocialStructure::None,
            flock_range: 0,
            flock_max: 1,
            flock_alert_range: 0,
            bulletin_channels: SmallVec::new(),
        }
    }
}

impl EntityProfile {
    pub fn has_bulletin_access(&self, channel: &str) -> bool {
        if self
            .bulletin_channels
            .iter()
            .any(|c| c == "full")
        {
            return true;
        }
        let key = match channel {
            "food_zones" => "food",
            "predator_zones" => "predator",
            "prey_zones" => "prey",
            "water_zones" => "water",
            "corpse_zones" => "corpse",
            "shelter_zones" => "shelter",
            _ => return false,
        };
        self.bulletin_channels.iter().any(|c| c == key)
    }
}

pub fn parse_bulletin_channels(tags: &[String]) -> SmallVec<[String; 6]> {
    let mut out = SmallVec::new();
    for t in tags {
        if let Some(rest) = t.strip_prefix("bulletin:") {
            out.push(rest.to_string());
        }
    }
    out
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
                "endurance" => 0.14,
                _ => 0.12,
            };
        }
    }
    0.12
}

pub fn parse_need_curve(name: &str) -> NeedCurve {
    match name {
        "flat" => NeedCurve::Flat,
        "sharp" => NeedCurve::Sharp,
        _ => NeedCurve::Steep,
    }
}

pub fn parse_needs(tags: &[String]) -> SmallVec<[NeedState; 4]> {
    let mut out = SmallVec::new();
    for t in tags {
        let Some(rest) = t.strip_prefix("need:") else {
            continue;
        };
        let (kind, params) = match rest.split_once('(') {
            Some((k, p)) => (k, p.trim_end_matches(')')),
            None => (rest, ""),
        };
        let decay_rate = parse_tag_f32_param(params, "rate").unwrap_or(0.5);
        let curve = parse_tag_str_param(params, "curve")
            .map(|c| parse_need_curve(&c))
            .unwrap_or(NeedCurve::Steep);
        out.push(NeedState {
            kind: kind.to_string(),
            current: 0.0,
            decay_rate,
            curve,
        });
    }
    out
}

fn needs_contain(needs: &[NeedState], kind: &str) -> bool {
    needs.iter().any(|n| n.kind == kind)
}

pub fn default_needs_for_drives(drives: &[DriveDef]) -> SmallVec<[NeedState; 4]> {
    let mut out = SmallVec::new();
    for drive in drives {
        match drive.need_kind.as_str() {
            "eat" if !needs_contain(&out, "eat") => out.push(NeedState {
                kind: "eat".into(),
                current: 0.0,
                decay_rate: 0.5,
                curve: NeedCurve::Steep,
            }),
            "safety" if !needs_contain(&out, "safety") => out.push(NeedState {
                kind: "safety".into(),
                current: 0.0,
                decay_rate: 1.0,
                curve: NeedCurve::Sharp,
            }),
            "social" if !needs_contain(&out, "social") => out.push(NeedState {
                kind: "social".into(),
                current: 0.0,
                decay_rate: 0.2,
                curve: NeedCurve::Flat,
            }),
            "rest" if !needs_contain(&out, "rest") => out.push(NeedState {
                kind: "rest".into(),
                current: 0.0,
                decay_rate: 0.2,
                curve: NeedCurve::Flat,
            }),
            _ => {}
        }
    }
    out
}

pub fn drive_need_kind(behavior: DriveBehavior, target_tag: &str) -> String {
    match behavior {
        DriveBehavior::Flee | DriveBehavior::Hide => "safety".into(),
        DriveBehavior::Flock => "social".into(),
        DriveBehavior::ReturnDen => "rest".into(),
        DriveBehavior::Scavenge => "eat".into(),
        DriveBehavior::Seek => match target_tag {
            "herbivore" | "smallPrey" | "largePrey" | "smallHerbivore" | "algae" | "fish" => {
                "eat".into()
            }
            _ => "eat".into(),
        },
        _ => "explore".into(),
    }
}

pub fn need_curve_value(need: &NeedState) -> f32 {
    let t = (need.current / 100.0).clamp(0.0, 1.0);
    match need.curve {
        NeedCurve::Steep => t.powi(3),
        NeedCurve::Flat => t,
        NeedCurve::Sharp => {
            if need.current > 60.0 {
                t
            } else {
                0.0
            }
        }
    }
}

pub fn score_need_for_drive(need: &NeedState, range: u8, distance: u8) -> f32 {
    let base = need_curve_value(need);
    let attenuation = if range > 0 {
        1.0 - (distance as f32 / range as f32).min(1.0) * 0.5
    } else {
        1.0
    };
    base * attenuation
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
        let need_kind = drive_need_kind(behavior, &target_tag);
        out.push(DriveDef {
            behavior,
            target_tag,
            range,
            priority,
            condition_fed,
            need_kind,
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

fn parse_tag_f32_param(s: &str, key: &str) -> Option<f32> {
    let needle = format!("{key}=");
    for part in s.split(',') {
        let part = part.trim();
        if let Some(v) = part.strip_prefix(&needle) {
            return v.parse().ok();
        }
    }
    None
}

pub fn parse_height(tags: &[String]) -> Height {
    for t in tags {
        if let Some(name) = t.strip_prefix("height:") {
            return match name {
                "flat" => Height::Flat,
                "low" => Height::Low,
                "medium" => Height::Medium,
                "high" => Height::High,
                _ => Height::Medium,
            };
        }
    }
    if tags.iter().any(|t| t == "body.large") {
        return Height::High;
    }
    if tags.iter().any(|t| t == "body.tiny") {
        return Height::Low;
    }
    if is_being_tagged(tags)
        || tags.iter().any(|t| t == "body.medium" || t == "body.small")
    {
        return Height::Medium;
    }
    if tags.iter().any(|t| t == "bush" || t == "foodSource" || t == "fungi" || t == "fern") {
        return Height::Low;
    }
    Height::Medium
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

#[cfg(test)]
mod need_tests {
    use super::*;

    #[test]
    fn steep_curve_rises_with_urgency() {
        let low = NeedState {
            kind: "eat".into(),
            current: 30.0,
            decay_rate: 0.5,
            curve: NeedCurve::Steep,
        };
        let high = NeedState {
            kind: "eat".into(),
            current: 90.0,
            decay_rate: 0.5,
            curve: NeedCurve::Steep,
        };
        assert!(need_curve_value(&high) > need_curve_value(&low) * 5.0);
    }

    #[test]
    fn parse_need_tags() {
        let tags = vec!["need:eat(rate=0.5,curve=steep)".to_string()];
        let needs = parse_needs(&tags);
        assert_eq!(needs[0].kind, "eat");
        assert!((needs[0].decay_rate - 0.5).abs() < f32::EPSILON);
        assert_eq!(needs[0].curve, NeedCurve::Steep);
    }

    #[test]
    fn drive_need_kind_maps_flee_to_safety() {
        assert_eq!(drive_need_kind(DriveBehavior::Flee, "predator"), "safety");
        assert_eq!(drive_need_kind(DriveBehavior::Seek, "foodSource"), "eat");
        assert_eq!(drive_need_kind(DriveBehavior::Flock, "sheep"), "social");
    }
}
