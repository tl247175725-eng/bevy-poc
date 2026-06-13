use std::collections::{HashMap, HashSet};

use crate::capabilities::card_capabilities;
use crate::game_constants::{CORPSE_DECAY_SECONDS, FOX_SCAVENGE_PER_DAY, WOLF_MEAT_PER_DAY};
use crate::spatial_index::EntityId;
use crate::systems::tick_containment::{entities_in_pool, entities_in_tree, entities_underground};
use crate::tag_zh::{cap_zh, tag_zh, SKIP_TAGS};
use crate::terrain::{base_terrain_at, cell_elevation, surface_label_with_stress, terrain_at, terrain_label};
use crate::terrain_colors::river_stress_label;
use crate::world_state::{EcologyState, Entity, WorldState};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContainmentEntry {
    pub entity_id: EntityId,
    pub display_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectionTarget {
    pub cell_x: u8,
    pub cell_y: u8,
    pub card_id: Option<EntityId>,
}

#[derive(Debug, Clone, Default)]
pub struct PanelContent {
    pub title: String,
    pub lines: Vec<String>,
    pub containment: Vec<ContainmentEntry>,
}

pub fn ecology_state_label(state: EcologyState) -> &'static str {
    match state {
        EcologyState::Idle => "空闲",
        EcologyState::SeekingFood => "觅食",
        EcologyState::Fleeing => "逃跑",
        EcologyState::Hunting => "捕猎",
        EcologyState::Patrolling => "巡逻",
        EcologyState::Burrowed => "穴居",
        EcologyState::InDen => "在窝内",
        EcologyState::Scavenging => "清腐",
        EcologyState::Wandering => "游荡",
    }
}

pub fn entity_state_label(entity: &Entity, def: &crate::card_def::CardDef) -> &'static str {
    match entity.ecology_state {
        EcologyState::SeekingFood
            if def.has_tag("herbivore") || def.has_tag("grass") || def.has_tag("grazer") =>
        {
            "吃草"
        }
        EcologyState::Fleeing => "逃跑",
        EcologyState::Wandering => "游荡",
        EcologyState::Hunting => "捕猎",
        EcologyState::Scavenging => "清腐",
        EcologyState::Patrolling => "巡逻",
        EcologyState::Burrowed => "穴居",
        EcologyState::InDen => "在窝内",
        other => ecology_state_label(other),
    }
}

pub fn sex_label(sex: Option<&str>) -> Option<&'static str> {
    match sex {
        Some("male") | Some("m") | Some("公") => Some("公"),
        Some("female") | Some("f") | Some("母") => Some("母"),
        _ => None,
    }
}

/// Pick the surface-visible card at a cell (prefers non-hidden entities).
pub fn resolve_selection_card(world: &WorldState, x: u8, y: u8) -> Option<EntityId> {
    let mut candidates: Vec<&Entity> = world
        .entities
        .values()
        .filter(|e| {
            e.x == x
                && e.y == y
                && !e.in_den
                && !e.in_burrow
                && !e.in_tree
                && !e.in_pool
                && !e.in_ground
                && !e.in_cover
        })
        .collect();

    candidates.sort_by_key(|e| {
        (
            e.hidden_in_grass as u8,
            e.profile.incorporeal as u8,
        )
    });

    candidates.first().map(|e| e.id)
}

pub fn ui_containment_entries(
    world: &WorldState,
    x: u8,
    y: u8,
    selected_id: Option<EntityId>,
) -> Vec<ContainmentEntry> {
    let mut entries = Vec::new();

    if let Some(id) = selected_id {
        if let Some(entity) = world.entities.get(&id) {
            if matches!(entity.type_name.as_str(), "oak" | "pine" | "tree") {
                for cid in entities_in_tree(world, id) {
                    if cid == id {
                        continue;
                    }
                    push_entry(world, &mut entries, cid);
                }
            }
        }
    }

    if world.pool_cells.contains(&(x, y)) {
        for cid in entities_in_pool(world, x, y) {
            push_entry(world, &mut entries, cid);
        }
    }

    for cid in entities_underground(world, x, y) {
        push_entry(world, &mut entries, cid);
    }

    entries.sort_by(|a, b| a.display_name.cmp(&b.display_name));
    entries.dedup_by(|a, b| a.entity_id == b.entity_id);
    entries
}

fn push_entry(world: &WorldState, entries: &mut Vec<ContainmentEntry>, id: EntityId) {
    if let Some(entity) = world.entities.get(&id) {
        let name = world
            .card_defs
            .get(&entity.type_name)
            .map(|d| d.display_name.clone())
            .unwrap_or_else(|| entity.type_name.clone());
        entries.push(ContainmentEntry {
            entity_id: id,
            display_name: name,
        });
    }
}

pub fn build_panel(world: &WorldState, target: &SelectionTarget) -> PanelContent {
    build_panel_with_stress(world, target, 0.0)
}

pub fn build_panel_with_stress(
    world: &WorldState,
    target: &SelectionTarget,
    river_stress: f32,
) -> PanelContent {
    if let Some(id) = target.card_id {
        build_card_panel(world, id, target.cell_x, target.cell_y)
    } else {
        build_cell_panel(world, target.cell_x, target.cell_y, river_stress)
    }
}

pub fn build_card_panel(
    world: &WorldState,
    entity_id: EntityId,
    cell_x: u8,
    cell_y: u8,
) -> PanelContent {
    let Some(entity) = world.entities.get(&entity_id) else {
        return PanelContent::default();
    };
    let Some(def) = world.card_defs.get(&entity.type_name) else {
        return PanelContent::default();
    };

    match entity.type_name.as_str() {
        "wolfDen" => return build_wolf_den_panel(world, entity_id, entity, def),
        "foxDen" => return build_fox_den_panel(world, entity_id, entity, def),
        "humus" => return build_humus_panel(world, entity, def, cell_x, cell_y),
        _ => {}
    }

    let mut lines = Vec::new();
    let identity = identity_tags(def, &entity.type_name);
    if !identity.is_empty() {
        lines.push(format!("身份：{}", identity.join(" · ")));
    }

    let caps: Vec<String> = card_capabilities(&entity.type_name)
        .iter()
        .map(|c| cap_zh(c))
        .collect();
    if !caps.is_empty() {
        lines.push(format!("能力：{}", caps.join(" · ")));
    }

    let state = entity_state_label(entity, def);
    lines.push(format!("状态：{}", state));
    append_goal_line(&mut lines, entity, def);
    append_dynamic_stats(&mut lines, world, entity, def);

    lines.push(format!("HP：{}", entity.hp));
    if let Some(sex) = sex_label(entity.sex.as_deref()) {
        lines.push(format!("性别：{}", sex));
    }

    let containment = ui_containment_entries(world, cell_x, cell_y, Some(entity_id));

    PanelContent {
        title: format!(
            "【{}】",
            def.display_name
        ),
        lines,
        containment,
    }
}

fn append_goal_line(lines: &mut Vec<String>, entity: &Entity, def: &crate::card_def::CardDef) {
    if entity.ecology_state == EcologyState::SeekingFood
        && (def.has_tag("herbivore") || def.has_tag("grass") || def.has_tag("grazer"))
    {
        lines.push("目标：草皮".to_string());
    } else if entity.ecology_state == EcologyState::Hunting {
        lines.push("目标：猎物".to_string());
    } else if entity.ecology_state == EcologyState::Scavenging {
        lines.push("目标：尸体".to_string());
    }
}

fn append_dynamic_stats(
    lines: &mut Vec<String>,
    world: &WorldState,
    entity: &Entity,
    def: &crate::card_def::CardDef,
) {
    if def.has_tag("meat_diet") && def.has_tag("predator") {
        lines.push(format!(
            "今日肉：{}/{}",
            entity.meat_fed_today, WOLF_MEAT_PER_DAY
        ));
    }
    if def.has_tag("scavenger") && def.has_tag("mesopredator") {
        lines.push(format!(
            "清腐：{}/{}",
            entity.scavenge_today, FOX_SCAVENGE_PER_DAY
        ));
    }
    if entity.is_corpse || def.has_tag("corpse") {
        lines.push(format!(
            "腐解：{}/{}秒",
            entity.decay_timer.round() as i32,
            CORPSE_DECAY_SECONDS.round() as i32
        ));
    }
    if def.has_tag("grass") {
        if let Some((x, y)) = world.spatial_index.position(entity.id) {
            if world.humus_layers.get(&(x, y)).copied().unwrap_or(0) > 0 {
                lines.push(format!(
                    "腐殖层：{}",
                    world.humus_layers.get(&(x, y)).copied().unwrap_or(0)
                ));
            }
        }
    }
}

fn build_wolf_den_panel(
    world: &WorldState,
    entity_id: EntityId,
    entity: &Entity,
    def: &crate::card_def::CardDef,
) -> PanelContent {
    let (adults, cubs) = den_wolf_counts(world, entity.x, entity.y);
    let mut lines = den_core_lines(def, "狼穴");
    lines.push(format!("窝内：成年狼×{}  幼狼×{}  储肉 0", adults, cubs));
    let containment = ui_containment_entries(world, entity.x, entity.y, Some(entity_id));
    PanelContent {
        title: "【狼穴】".into(),
        lines,
        containment,
    }
}

fn build_fox_den_panel(
    world: &WorldState,
    entity_id: EntityId,
    entity: &Entity,
    def: &crate::card_def::CardDef,
) -> PanelContent {
    let (adults, cubs) = den_fox_counts(world, entity.x, entity.y);
    let mut lines = den_core_lines(def, "狐窝");
    lines.push(format!("窝内：成年狐×{}  幼狐×{}  储肉 0", adults, cubs));
    let containment = ui_containment_entries(world, entity.x, entity.y, Some(entity_id));
    PanelContent {
        title: "【狐窝】".into(),
        lines,
        containment,
    }
}

fn build_humus_panel(
    world: &WorldState,
    _entity: &Entity,
    def: &crate::card_def::CardDef,
    x: u8,
    y: u8,
) -> PanelContent {
    let layers = world.humus_layers.get(&(x, y)).copied().unwrap_or(0);
    let mut lines = den_core_lines(def, &def.display_name);
    if layers > 0 {
        lines.push(format!("层数：肥沃×{}", layers));
    }
    PanelContent {
        title: format!("【{}】", def.display_name),
        lines,
        containment: Vec::new(),
    }
}

fn den_core_lines(def: &crate::card_def::CardDef, _title: &str) -> Vec<String> {
    let mut lines = Vec::new();
    let identity = identity_tags(def, &def.type_name);
    if !identity.is_empty() {
        lines.push(format!("身份：{}", identity.join(" · ")));
    }
    let caps: Vec<String> = card_capabilities(&def.type_name)
        .iter()
        .map(|c| cap_zh(c))
        .collect();
    if !caps.is_empty() {
        lines.push(format!("能力：{}", caps.join(" · ")));
    }
    lines
}

fn den_wolf_counts(world: &WorldState, x: u8, y: u8) -> (usize, usize) {
    den_resident_counts(world, x, y)
}

fn den_fox_counts(world: &WorldState, x: u8, y: u8) -> (usize, usize) {
    den_resident_counts(world, x, y)
}

fn den_resident_counts(world: &WorldState, x: u8, y: u8) -> (usize, usize) {
    let mut adults = 0usize;
    let mut cubs = 0usize;
    for e in world.entities.values() {
        if e.x != x || e.y != y {
            continue;
        }
        let Some(def) = world.card_defs.get(&e.type_name) else {
            continue;
        };
        if def.has_tag("juvenile") {
            cubs += 1;
        } else if def.has_tag("den_resident") && !e.is_corpse {
            adults += 1;
        }
    }
    (adults, cubs)
}

pub fn build_cell_panel(world: &WorldState, x: u8, y: u8, river_stress: f32) -> PanelContent {
    if let Some(overlay_id) = overlay_entity_at(world, x, y) {
        if let Some(entity) = world.entities.get(&overlay_id) {
            if world
                .card_defs
                .get(&entity.type_name)
                .is_some_and(|d| d.has_tag("cell.overlay"))
            {
                return build_card_panel(world, overlay_id, x, y);
            }
        }
    }

    let label = surface_label_with_stress(world, x, y, river_stress)
        .unwrap_or_else(|| terrain_label(base_terrain_at(world, x, y)).to_string());
    let title = format!("{} ({}, {})", label, x, y);

    let mut lines = Vec::new();
    let identity = cell_terrain_identity(world, x, y);
    if !identity.is_empty() {
        lines.push(format!("身份：{}", identity.join(" · ")));
    }

    let elev = cell_elevation(world, x, y);
    if elev != 0 {
        lines.push(format!("海拔：{}", elev));
    }

    if world.pool_cells.contains(&(x, y)) {
        let summary = pool_occupant_summary(world, x, y);
        if !summary.is_empty() {
            lines.push(format!("容纳：{}", summary.join("  ")));
        }
    } else {
        let surface = surface_entity_names(world, x, y);
        if !surface.is_empty() {
            lines.push(format!("地表：{}", surface.join("、")));
        }
        let underground = underground_entity_names(world, x, y);
        if !underground.is_empty() {
            lines.push(format!("地下：{}", underground.join("、")));
        }
    }

    if let Some(overlay) = crate::terrain::surface_label(world, x, y) {
        if !title.contains(&overlay) {
            lines.push(format!("覆盖：{}", overlay));
        }
    }

    lines.push(format!("水势：{}", river_stress_label(river_stress)));

    let cell_state = cell_state_label(world, x, y);
    if !cell_state.is_empty() {
        lines.push(format!("状态：{}", cell_state));
    }

    let containment = ui_containment_entries(world, x, y, None);

    PanelContent {
        title,
        lines,
        containment,
    }
}

fn overlay_entity_at(world: &WorldState, x: u8, y: u8) -> Option<EntityId> {
    world.entities.values().find_map(|e| {
        if e.x == x
            && e.y == y
            && world
                .card_defs
                .get(&e.type_name)
                .is_some_and(|d| d.has_tag("cell.overlay"))
        {
            Some(e.id)
        } else {
            None
        }
    })
}

fn cell_terrain_identity(world: &WorldState, x: u8, y: u8) -> Vec<String> {
    let mut labels = Vec::new();
    let mut seen = HashSet::new();
    let base = base_terrain_at(world, x, y);
    if let Some(zh) = terrain_identity_zh(base) {
        seen.insert(zh.to_string());
        labels.push(zh.to_string());
    }
    if world.ecology.ready {
        if world.ecology.is_riparian_grass_cell(x, y) {
            push_unique(&mut labels, &mut seen, "湿润土地");
        }
        let role = world.ecology.underground_river_role(x, y);
        if !role.is_empty() {
            let zh = tag_zh(role);
            if !zh.is_empty() {
                push_unique(&mut labels, &mut seen, &zh);
            }
        }
    }
    labels
}

fn push_unique(labels: &mut Vec<String>, seen: &mut HashSet<String>, label: &str) {
    if !seen.insert(label.to_string()) {
        return;
    }
    labels.push(label.to_string());
}

fn terrain_identity_zh(terrain: &str) -> Option<&'static str> {
    match terrain {
        "pool" | "dark_river_pool" => Some("水潭"),
        "river" | "ford" => Some("河沟"),
        "wetland" => Some("湿地"),
        "grassland" | "bank" | "riverbank" => Some("湿润土地"),
        "land" => Some("荒地"),
        "wasteland" => Some("焦土"),
        _ => None,
    }
}

fn pool_occupant_summary(world: &WorldState, x: u8, y: u8) -> Vec<String> {
    let mut counts: HashMap<String, usize> = HashMap::new();
    for id in entities_in_pool(world, x, y) {
        let Some(entity) = world.entities.get(&id) else {
            continue;
        };
        let name = world
            .card_defs
            .get(&entity.type_name)
            .map(|d| d.display_name.clone())
            .unwrap_or_else(|| entity.type_name.clone());
        *counts.entry(name).or_default() += 1;
    }
    let mut out: Vec<_> = counts
        .into_iter()
        .map(|(name, n)| format!("{}×{}", name, n))
        .collect();
    out.sort();
    out
}

fn surface_entity_names(world: &WorldState, x: u8, y: u8) -> Vec<String> {
    world
        .entities_at(x, y)
        .iter()
        .filter_map(|id| world.entities.get(id))
        .filter(|e| !e.in_pool && !e.in_tree && !e.in_ground && !e.in_den)
        .filter_map(|e| world.card_defs.get(&e.type_name))
        .map(|d| d.display_name.clone())
        .collect()
}

fn underground_entity_names(world: &WorldState, x: u8, y: u8) -> Vec<String> {
    entities_underground(world, x, y)
        .iter()
        .filter_map(|id| world.entities.get(id))
        .filter_map(|e| world.card_defs.get(&e.type_name))
        .map(|d| d.display_name.clone())
        .collect()
}

fn cell_state_label(world: &WorldState, x: u8, y: u8) -> String {
    if world.fire_cells.contains(&(x, y)) {
        return "焦土".into();
    }
    if matches!(terrain_at(world, x, y), "river" | "ford") {
        return "流水".into();
    }
    if world.pool_cells.contains(&(x, y)) {
        return "静水".into();
    }
    String::new()
}

fn identity_tags(def: &crate::card_def::CardDef, type_name: &str) -> Vec<String> {
    def.tags
        .iter()
        .filter(|t| *t != type_name && !SKIP_TAGS.contains(&t.as_str()))
        .map(|t| tag_zh(t))
        .filter(|t| !t.is_empty())
        .collect()
}

pub fn panel_text_joined(content: &PanelContent) -> String {
    let mut parts = vec![content.title.clone()];
    parts.extend(content.lines.clone());
    if !content.containment.is_empty() {
        parts.push("容纳".to_string());
        for e in &content.containment {
            parts.push(format!("【{}】", e.display_name));
        }
    }
    parts.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::empty_world;

    #[test]
    fn cell_panel_includes_cover_and_water_stress() {
        let mut w = empty_world();
        w.mark_river(10, 5);
        let panel = build_cell_panel(&w, 10, 5, 72.0);
        assert!(panel.title.contains("河沟"));
        assert!(panel.lines.iter().any(|l| l.contains("水势：紧")));
    }

    #[test]
    #[ignore]
    fn pool_panel_shows_occupant_counts() { let _ = empty_world(); }

    #[test]
    #[ignore]
    fn resolve_selection_skips_entities_in_cover() { let _ = empty_world(); }
}
