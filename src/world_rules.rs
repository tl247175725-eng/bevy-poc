use crate::capabilities::card_capabilities;
use crate::card_def::CardDef;

pub const GRID_WIDTH: u8 = 36;
pub const GRID_HEIGHT: u8 = 24;
pub const HUNT_RANGE: u8 = 5;
pub const FEAR_RANGE: u8 = 4;
pub const PACK_MIN_STRENGTH: usize = 2;
pub const FLOCKING_REPRO_MIN: usize = 3;
pub const GRASS_REGEN_INTERVAL: u64 = 10;
pub const REPRO_COOLDOWN_TICKS: u64 = 50;
pub const POPULATION_REPRO_CYCLE: f32 = 12.0;
pub const PROLIFIC_REPRO_CYCLE: f32 = 3.0;

pub const HUNT_PROFILE_PACK: &str = "pack_predator";
pub const HUNT_PROFILE_MESO: &str = "mesopredator";
pub const HUNT_PROFILE_TOOL: &str = "tool_hunter";
pub const HUNT_DIET_FOX: &str = "fox";

pub const HUNT_SCORE_INF: f32 = f32::INFINITY;

// --- tag primitives ---

pub fn card_has_tag(def: &CardDef, tag: &str) -> bool {
    def.tags.iter().any(|t| t == tag || t.starts_with(&format!("{tag}.")))
}

pub fn parse_tag_u32(tags: &[String], prefix: &str, default: u32) -> u32 {
    for t in tags {
        if let Some(v) = t.strip_prefix(prefix) {
            return v.parse().unwrap_or(default);
        }
    }
    default
}

pub fn parse_meat_yield(def: &CardDef) -> u32 {
    parse_tag_u32(&def.tags, "meat_yield:", 1)
}

pub fn parse_meat_product(def: &CardDef) -> Option<String> {
    def.tags
        .iter()
        .find_map(|t| t.strip_prefix("meat_product:").map(str::to_string))
}

pub fn card_has_capability(def: &CardDef, capability: &str) -> bool {
    card_capabilities(&def.type_name).contains(&capability)
}

pub fn is_being(def: &CardDef) -> bool {
    card_has_tag(def, "being")
}

pub fn is_camp_fire_anchor(def: &CardDef) -> bool {
    def.type_name == "fire" || (card_has_tag(def, "camp.anchor") && card_has_tag(def, "heat"))
}

pub fn is_animal(def: &CardDef) -> bool {
    card_has_tag(def, "animal")
}

pub fn is_predator(def: &CardDef) -> bool {
    card_has_tag(def, "predator")
}

pub fn is_mesopredator(def: &CardDef) -> bool {
    card_has_tag(def, "mesopredator")
}

pub fn is_herbivore(def: &CardDef) -> bool {
    card_has_tag(def, "herbivore")
}

pub fn is_living_grass(def: &CardDef) -> bool {
    card_has_tag(def, "grass")
}

pub fn count_living_grasses(world: &crate::world_state::WorldState) -> usize {
    world
        .entities
        .values()
        .filter(|e| {
            !e.is_corpse
                && world
                    .card_defs
                    .get(&e.type_name)
                    .is_some_and(is_living_grass)
        })
        .count()
}

pub fn count_living_grasses_near_xy(
    world: &crate::world_state::WorldState,
    x: u8,
    y: u8,
    radius: u8,
) -> usize {
    world
        .query_near_filtered(x, y, "grass", radius, crate::spatial_index::EntityId(0))
        .iter()
        .filter(|id| world.entities.get(id).is_some_and(|e| !e.is_corpse))
        .count()
}

pub fn is_juvenile(def: &CardDef) -> bool {
    card_has_tag(def, "juvenile")
}

pub fn is_small_prey(def: &CardDef) -> bool {
    card_has_tag(def, "smallPrey")
}

pub fn is_large_prey(def: &CardDef) -> bool {
    card_has_tag(def, "largePrey")
}

pub fn is_grass(def: &CardDef) -> bool {
    card_has_tag(def, "grass")
}

pub fn is_aquatic_card(def: &CardDef) -> bool {
    card_has_tag(def, "aquatic")
}

pub fn is_burrower(def: &CardDef) -> bool {
    card_has_tag(def, "burrower")
}

// --- ecology ---

pub fn flocking_blocks_reproduction(adults: &[&CardDef]) -> bool {
    if adults.is_empty() {
        return false;
    }
    if !card_has_tag(adults[0], "flocking") {
        return false;
    }
    adults.len() < FLOCKING_REPRO_MIN
}

pub fn pack_hunter_under_strength(hunters: &[&CardDef]) -> bool {
    if hunters.is_empty() {
        return true;
    }
    if !card_has_tag(hunters[0], "pack_hunter") {
        return false;
    }
    hunters.len() < PACK_MIN_STRENGTH
}

pub fn is_grazer_flee_wolf_threat(actor_def: &CardDef) -> bool {
    card_has_tag(actor_def, "predator")
        && card_has_tag(actor_def, "pack_hunter")
        && !card_has_tag(actor_def, "juvenile")
}

pub fn can_hunt_def(def: &CardDef) -> bool {
    card_has_capability(def, "capability.hunt") || is_predator(def)
}

pub fn can_be_hunted_def(def: &CardDef) -> bool {
    card_has_capability(def, "capability.be_hunted")
}

pub fn is_tough_adult_prey(def: &CardDef) -> bool {
    card_has_tag(def, "tough") && !card_has_tag(def, "juvenile")
}

pub fn hunt_profile_for(hunter_def: &CardDef) -> &'static str {
    if !can_hunt_def(hunter_def) {
        return "";
    }
    if card_has_tag(hunter_def, "mesopredator") {
        return HUNT_PROFILE_MESO;
    }
    if card_has_tag(hunter_def, "predator") {
        return HUNT_PROFILE_PACK;
    }
    if card_has_tag(hunter_def, "actor") {
        return HUNT_PROFILE_TOOL;
    }
    ""
}

fn mesopredator_diet_key(hunter_def: &CardDef) -> &'static str {
    if hunter_def.type_name == "fox" {
        HUNT_DIET_FOX
    } else {
        ""
    }
}

fn pack_prey_allowed(hunter: &CardDef, target: &CardDef, pack_adult_count: usize) -> bool {
    if !can_be_hunted_def(target) {
        return false;
    }
    let under = card_has_tag(hunter, "pack_hunter") && pack_adult_count < PACK_MIN_STRENGTH;
    if under {
        return card_has_tag(target, "smallPrey") || card_has_tag(target, "smallHerbivore");
    }
    if card_has_tag(target, "smallPrey") || card_has_tag(target, "smallHerbivore") {
        return false;
    }
    if card_has_tag(target, "mesopredator") || card_has_tag(target, "predator") {
        return false;
    }
    if card_has_tag(target, "actor") {
        return false;
    }
    if is_tough_adult_prey(target) {
        return false;
    }
    card_has_tag(target, "largePrey") || card_has_tag(target, "herbivore")
}

fn meso_prey_allowed(hunter: &CardDef, target: &CardDef) -> bool {
    if !can_be_hunted_def(target) {
        return false;
    }
    if card_has_tag(target, "largePrey") && !card_has_tag(target, "juvenile") {
        return false;
    }
    if card_has_tag(target, "predator") || card_has_tag(target, "actor") {
        return false;
    }
    if is_tough_adult_prey(target) {
        return false;
    }
    if mesopredator_diet_key(hunter) == HUNT_DIET_FOX {
        return card_has_tag(target, "smallPrey") || card_has_tag(target, "smallHerbivore");
    }
    card_has_tag(target, "smallPrey") || card_has_tag(target, "smallHerbivore")
}

fn tool_prey_allowed(_hunter: &CardDef, target: &CardDef) -> bool {
    if !can_be_hunted_def(target) {
        return false;
    }
    !card_has_capability(target, "capability.be_cared_for")
}

pub fn is_hunt_target_for(hunter_def: &CardDef, target_def: &CardDef) -> bool {
    is_hunt_target_for_pack(hunter_def, target_def, PACK_MIN_STRENGTH)
}

pub fn is_hunt_target_for_pack(
    hunter_def: &CardDef,
    target_def: &CardDef,
    pack_adult_count: usize,
) -> bool {
    let profile = hunt_profile_for(hunter_def);
    match profile {
        HUNT_PROFILE_PACK => pack_prey_allowed(hunter_def, target_def, pack_adult_count),
        HUNT_PROFILE_MESO => meso_prey_allowed(hunter_def, target_def),
        HUNT_PROFILE_TOOL => tool_prey_allowed(hunter_def, target_def),
        _ => false,
    }
}

/// 已掏空——始终返回不可狩猎
pub fn hunt_target_score(
    _hunter_def: &CardDef,
    _target_def: &CardDef,
    _distance: f32,
    _pack_adult_count: usize,
) -> f32 {
    HUNT_SCORE_INF
}

#[derive(Debug, Clone)]
pub struct HuntCandidate<'a> {
    pub def: &'a CardDef,
    pub distance: f32,
}

pub fn best_hunt_target<'a>(
    hunter_def: &CardDef,
    candidates: &[HuntCandidate<'a>],
    pack_adult_count: usize,
) -> Option<&'a CardDef> {
    candidates
        .iter()
        .filter(|c| {
            is_hunt_target_for_pack(hunter_def, c.def, pack_adult_count)
                && hunt_target_score(hunter_def, c.def, c.distance, pack_adult_count)
                    < HUNT_SCORE_INF
        })
        .min_by(|a, b| {
            let sa = hunt_target_score(hunter_def, a.def, a.distance, pack_adult_count);
            let sb = hunt_target_score(hunter_def, b.def, b.distance, pack_adult_count);
            sa.partial_cmp(&sb).unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|c| c.def)
}

pub fn is_feed_source(actor_def: &CardDef, source_def: &CardDef) -> bool {
    if !can_hunt_def(actor_def) {
        return false;
    }
    card_has_tag(source_def, "perishable") || card_has_tag(source_def, "corpse")
}

#[derive(Debug, Clone)]
pub struct FeedCandidate<'a> {
    pub def: &'a CardDef,
    pub distance: f32,
}

pub fn best_feed_source_for<'a>(
    actor_def: &CardDef,
    candidates: &[FeedCandidate<'a>],
) -> Option<&'a CardDef> {
    candidates
        .iter()
        .filter(|c| is_feed_source(actor_def, c.def))
        .min_by(|a, b| {
            a.distance
                .partial_cmp(&b.distance)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|c| c.def)
}

#[derive(Debug, Clone, Default)]
pub struct PerishableState {
    pub perish_ticks: i32,
}

pub fn mark_perishable(state: &mut PerishableState) {
    state.perish_ticks = crate::game_constants::PERISHABLE_TICKS;
}

// --- reproduction ---

pub fn can_reproduce(male_def: &CardDef, female_def: &CardDef) -> bool {
    male_def.type_name == female_def.type_name
        && card_has_capability(male_def, "capability.reproduce")
        && card_has_capability(female_def, "capability.reproduce")
}

pub fn prolific_litter_size(def: &CardDef) -> i32 {
    if card_has_tag(def, "prolific") {
        3
    } else {
        1
    }
}

pub fn prolific_repro_cycle(def: &CardDef) -> f32 {
    if card_has_tag(def, "prolific") {
        PROLIFIC_REPRO_CYCLE
    } else {
        POPULATION_REPRO_CYCLE
    }
}

// --- POC compat helpers (behavior layer uses these until M2) ---

pub fn can_hunt_target(pack_size: usize, wolf_def: &CardDef, prey_def: &CardDef) -> bool {
    is_hunt_target_for_pack(wolf_def, prey_def, pack_size)
}

pub fn chebyshev_distance(x1: u8, y1: u8, x2: u8, y2: u8) -> u8 {
    x1.abs_diff(x2).max(y1.abs_diff(y2))
}

pub fn in_range(x1: u8, y1: u8, x2: u8, y2: u8, range: u8) -> bool {
    chebyshev_distance(x1, y1, x2, y2) <= range
}

// --- M2 herbivore profiles ---

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrazerProfile {
    Juvenile,
    Rabbit,
    Deer,
    Sheep,
    Slow,
    Pheasant,
}

pub fn can_forage(def: &CardDef) -> bool {
    card_has_capability(def, "capability.forage")
}

pub fn is_herbivore_grazer_juvenile(def: &CardDef) -> bool {
    card_has_tag(def, "juvenile") && card_has_capability(def, "capability.be_cared_for")
}

pub fn herbivore_grazer_profile(def: &CardDef) -> GrazerProfile {
    if is_herbivore_grazer_juvenile(def) {
        return GrazerProfile::Juvenile;
    }
    if (def.type_name == "pheasant" || def.type_name == "pheasantChick")
        && card_has_tag(def, "flocking")
        && card_has_tag(def, "omnivore.small")
    {
        return GrazerProfile::Pheasant;
    }
    if card_has_capability(def, "capability.escape_small") {
        return GrazerProfile::Rabbit;
    }
    if card_has_capability(def, "capability.escape_fast") {
        return GrazerProfile::Deer;
    }
    if card_has_capability(def, "capability.move_slow") && can_forage(def) {
        return GrazerProfile::Slow;
    }
    if card_has_capability(def, "capability.reproduce") && can_forage(def) {
        return GrazerProfile::Sheep;
    }
    GrazerProfile::Juvenile
}

pub fn ecology_uses_meat_quota(def: &CardDef) -> bool {
    can_hunt_def(def) && is_predator(def)
}

pub fn parse_max_starve(def: &CardDef) -> i32 {
    def.tags
        .iter()
        .find_map(|t| t.strip_prefix("max_starve:"))
        .and_then(|v| v.parse().ok())
        .unwrap_or(3)
}

pub fn parse_max_age(def: &CardDef) -> f32 {
    def.tags
        .iter()
        .find_map(|t| t.strip_prefix("max_age:"))
        .and_then(|v| v.parse().ok())
        .unwrap_or(0.0)
}

pub fn mark_ecology_fed(entity: &mut crate::world_state::Entity, def: &CardDef) {
    if ecology_uses_meat_quota(def) {
        entity.meat_fed_today += 1;
    } else {
        entity.fed_today = true;
        entity.fed = true;
    }
    entity.starve_days = 0;
    if let Some(need) = entity.profile.needs.iter_mut().find(|n| n.kind == "eat") {
        need.current = 0.0;
    }
}

pub fn ecology_was_fed_today(entity: &crate::world_state::Entity, def: &CardDef) -> bool {
    if ecology_uses_meat_quota(def) {
        entity.meat_fed_today > 0
    } else {
        entity.fed_today || entity.fed
    }
}

pub fn corpse_type_for(world: &crate::world_state::WorldState, living_type: &str) -> String {
    world
        .card_defs
        .get(living_type)
        .and_then(|def| {
            def.tags
                .iter()
                .find_map(|t| t.strip_prefix("corpse_type:"))
        })
        .map(str::to_string)
        .unwrap_or_else(|| format!("{living_type}Corpse"))
}

pub fn is_sessile(def: &CardDef) -> bool {
    card_has_tag(def, "sessile_aquatic") || card_has_tag(def, "sessile")
}

pub fn is_floating(def: &CardDef) -> bool {
    card_has_tag(def, "floating")
}

pub fn predators_near(
    world: &crate::world_state::WorldState,
    x: u8,
    y: u8,
    range: u8,
) -> Vec<crate::spatial_index::EntityId> {
    world.query_near_filtered(x, y, "predator", range, crate::spatial_index::EntityId(0))
}

pub fn wolves_near(
    world: &crate::world_state::WorldState,
    x: u8,
    y: u8,
    range: u8,
) -> Vec<crate::spatial_index::EntityId> {
    predators_near(world, x, y, range)
}

// --- ecosystem behavior keys (Godot `ecosystem_behavior_key`) ---

pub const BEHAVIOR_PREDATOR_DEN: &str = "predator_den";
pub const BEHAVIOR_MESOPREDATOR_HUNT: &str = "mesopredator_hunt";
pub const BEHAVIOR_HERBIVORE_GRAZER: &str = "herbivore_grazer";
pub const BEHAVIOR_COVER_FORAGER: &str = "cover_forager";

/// 行为键查询——已掏空，所有实体均为空行为键
pub fn ecosystem_behavior_key(_def: &CardDef, _type_name: &str) -> &'static str {
    ""
}
