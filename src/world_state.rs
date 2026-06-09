use crate::axioms::{
    AxiomEngine, CausalEvent, CausalStorage, CellComposition, Composition, Medium, Traversal,
};
use crate::card_def::{card_defs_map, load_card_defs, CardDef};
use crate::game_constants::TICK_SECONDS;
use crate::player::PlayerMind;
use crate::spatial_index::{EntityId, IndexedEntity, SpatialIndex};
use crate::terrain_ecology::MapEcology;
use crate::world_rules::card_has_capability;
use std::collections::{HashMap, HashSet};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveResult {
    Moved,
    Blocked,
    NoOp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EcologyState {
    Idle,
    SeekingFood,
    Fleeing,
    Hunting,
    Patrolling,
    Burrowed,
    InDen,
    Scavenging,
    Wandering,
}

#[derive(Debug, Clone)]
pub struct Entity {
    pub id: EntityId,
    pub type_name: String,
    pub x: u8,
    pub y: u8,
    pub hp: i32,
    pub fed: bool,
    pub is_corpse: bool,
    pub ecology_state: EcologyState,
    pub consumed: bool,
    pub sex: Option<String>,
    pub age: f32,
    pub fed_today: bool,
    pub starve_days: i32,
    pub in_den: bool,
    pub in_tree: bool,
    pub in_pool: bool,
    pub in_ground: bool,
    pub in_burrow: bool,
    pub carrying: Option<EntityId>,
    pub stunned: bool,
    pub hunt_cooldown: f32,
    pub eat_time: f32,
    pub perish_ticks: i32,
    pub produce_timer: f32,
    pub hidden_in_grass: bool,
    pub decay_timer: f32,
    pub den_id: Option<EntityId>,
    pub meat_fed_today: i32,
    pub scavenge_today: i32,
    pub harvest_cooldown: f32,
    pub host_tree_id: Option<EntityId>,
    pub host_pool_x: Option<u8>,
    pub host_pool_y: Option<u8>,
    /// Set by OnMove prey notify; consumed once at end of `main_tick`.
    pub needs_patrol: bool,
    /// Baseline herbivore/forager ecology tick — cleared by move or end-of-tick flush.
    pub needs_grazing_tick: bool,
    /// Precomputed axiom profile (tags parsed once at spawn / refresh).
    pub profile: crate::axioms::EntityProfile,
    /// Remaining scatter ticks — cohesion suppressed while > 0.
    pub scatter_timer: i8,
}

impl Entity {
    pub fn is_autonomous(&self, defs: &HashMap<String, CardDef>) -> bool {
        defs.get(&self.type_name)
            .map(|d| card_has_capability(d, "capability.move"))
            .unwrap_or(false)
            && !self.is_corpse
            && !self.in_den
            && !self.in_burrow
    }
}

pub struct WorldState {
    pub entities: HashMap<EntityId, Entity>,
    pub spatial_index: SpatialIndex,
    pub card_defs: HashMap<String, CardDef>,
    /// Godot `WorldRulesTerrain` — fixed map ecology (elevation, pools, wetlands).
    pub ecology: MapEcology,
    /// Test / legacy overrides; not part of Godot v3 surface river (always empty after `ensure_map_ecology`).
    pub river_cells: HashSet<(u8, u8)>,
    pub pool_cells: HashSet<(u8, u8)>,
    pub fire_cells: HashSet<(u8, u8)>,
    pub bush_microfauna: HashMap<(u8, u8), i32>,
    pub humus_layers: HashMap<(u8, u8), i32>,
    pub humus_age: HashMap<(u8, u8), f32>,
    pub grass_regen_timer: f32,
    pub repro_timer: f32,
    pub rabbit_repro_timer: f32,
    pub aquatic_timer: f32,
    pub yam_timer: f32,
    pub riparian_timer: f32,
    pub tick_count: u64,
    pub elapsed: f32,
    pub tick_delta: f32,
    pub pending_events: Vec<crate::sim_events::SimEvent>,
    pub pending_move_anims: Vec<crate::sim_events::MoveAnimEvent>,
    /// Prevents OnMove neighbor notify re-entry (flee → move → hunt loops).
    pub sim_observer_depth: u8,
    /// Spawn-time ecology deferred to end of tick (assert co-spawn before first tick).
    pub pending_spawn_ecology: Vec<EntityId>,
    /// Reused each tick to avoid allocations in patrol / aquatic scans.
    pub tick_scratch: Vec<EntityId>,
    /// Phase 5 player AI minds — one entry per player entity.
    pub player_minds: HashMap<EntityId, PlayerMind>,
    pub cell_composition: CellComposition,
    pub causal_storage: CausalStorage,
    pub medium_conductions: HashMap<Medium, Vec<(String, f32)>>,
    next_id: u64,
}

impl WorldState {
    pub fn new(card_defs: HashMap<String, CardDef>) -> Self {
        Self {
            entities: HashMap::new(),
            spatial_index: SpatialIndex::new(),
            card_defs,
            ecology: MapEcology::default(),
            river_cells: HashSet::new(),
            pool_cells: HashSet::new(),
            fire_cells: HashSet::new(),
            bush_microfauna: HashMap::new(),
            humus_layers: HashMap::new(),
            humus_age: HashMap::new(),
            grass_regen_timer: 0.0,
            repro_timer: 0.0,
            rabbit_repro_timer: 0.0,
            aquatic_timer: 0.0,
            yam_timer: 0.0,
            riparian_timer: 0.0,
            tick_count: 0,
            elapsed: 0.0,
            tick_delta: crate::game_constants::TICK_SECONDS,
            pending_events: Vec::new(),
            pending_move_anims: Vec::new(),
            sim_observer_depth: 0,
            pending_spawn_ecology: Vec::new(),
            tick_scratch: Vec::new(),
            player_minds: HashMap::new(),
            cell_composition: CellComposition::empty(),
            causal_storage: CausalStorage::default(),
            medium_conductions: AxiomEngine::default_medium_conductions(),
            next_id: 1,
        }
    }

    pub fn set_causal_mode(&mut self, is_smoke_test: bool) {
        self.causal_storage = CausalStorage::for_mode(is_smoke_test, cfg!(debug_assertions));
    }

    pub fn get_medium_conduction(&self, medium: &Medium) -> Vec<(String, f32)> {
        self.medium_conductions
            .get(medium)
            .cloned()
            .unwrap_or_else(|| {
                self.medium_conductions
                    .get("land")
                    .cloned()
                    .unwrap_or_default()
            })
    }

    pub fn query_near_filtered(
        &self,
        x: u8,
        y: u8,
        tag: &str,
        range: u8,
        observer_id: EntityId,
    ) -> Vec<EntityId> {
        let raw = self.spatial_index.query_near(x, y, tag, range);
        let Some(observer) = self.entities.get(&observer_id) else {
            return raw;
        };
        let obs_profile = &observer.profile;

        raw.into_iter()
            .filter(|id| {
                let Some(target) = self.entities.get(id) else {
                    return false;
                };
                let tgt_profile = &target.profile;
                let dist =
                    crate::world_rules::chebyshev_distance(x, y, target.x, target.y) as u8;
                matches!(
                    AxiomEngine::perceive(
                        obs_profile,
                        tgt_profile,
                        dist,
                        &self.get_medium_conduction(&obs_profile.current_medium),
                        &self.get_medium_conduction(&tgt_profile.current_medium),
                    ),
                    crate::axioms::Perception::Detected { .. }
                )
            })
            .collect()
    }

    pub fn from_card_defs_file(path: impl AsRef<Path>) -> Self {
        let defs = load_card_defs(path);
        Self::new(card_defs_map(&defs))
    }

    pub fn def_for(&self, type_name: &str) -> Option<&CardDef> {
        self.card_defs.get(type_name)
    }

    pub fn spawn(&mut self, type_name: &str, x: u8, y: u8) -> EntityId {
        self.spawn_with_sex(type_name, x, y, None)
    }

    pub fn spawn_with_sex(
        &mut self,
        type_name: &str,
        x: u8,
        y: u8,
        sex: Option<String>,
    ) -> EntityId {
        use crate::world_rules::{GRID_HEIGHT, GRID_WIDTH};
        let mut x = x.clamp(0, GRID_WIDTH - 1);
        let mut y = y.clamp(0, GRID_HEIGHT - 1);
        let def = self
            .card_defs
            .get(type_name)
            .cloned()
            .unwrap_or_else(|| panic!("unknown card type: {type_name}"));
        let id = EntityId(self.next_id);
        self.next_id += 1;

        let mut profile = AxiomEngine::build_profile(id, type_name, &def.tags, def.hp, self, x, y);
        profile.current_medium = self.cell_composition.slot(x, y).medium.clone();

        let entity = Entity {
            id,
            type_name: type_name.to_string(),
            x,
            y,
            hp: def.hp,
            fed: false,
            is_corpse: type_name.ends_with("Corpse"),
            ecology_state: EcologyState::Idle,
            consumed: false,
            sex,
            age: 0.0,
            fed_today: false,
            starve_days: 0,
            in_den: false,
            in_tree: def.is_rooted && type_name != "grass",
            in_pool: type_name == "algae"
                || type_name == "waterBug"
                || type_name == "fish"
                || type_name == "shellfish"
                || type_name == "waterCaltrop"
                || type_name == "lotus",
            in_ground: type_name == "wildYam",
            in_burrow: false,
            carrying: None,
            stunned: false,
            hunt_cooldown: 0.0,
            eat_time: 0.0,
            perish_ticks: -1,
            produce_timer: 0.0,
            hidden_in_grass: false,
            decay_timer: 0.0,
            den_id: None,
            meat_fed_today: 0,
            scavenge_today: 0,
            harvest_cooldown: 0.0,
            host_tree_id: None,
            host_pool_x: None,
            host_pool_y: None,
            needs_patrol: false,
            needs_grazing_tick: false,
            profile,
            scatter_timer: 0,
        };
        if type_name == "bush" {
            self.bush_microfauna.insert((x, y), crate::game_constants::BUSH_INITIAL_MICROFAUNA);
        }
        self.cell_composition.occupy_entity(x, y, &entity);
        self.index_entity(&entity, &def);
        self.entities.insert(id, entity);
        AxiomEngine::trace(
            &mut self.causal_storage,
            CausalEvent {
                tick: self.tick_count,
                cause_entity_id: id.0,
                cause_tag: "spawn".into(),
                effect_entity_id: id.0,
                effect_description: format!("spawned({type_name})"),
            },
        );
        crate::sim_observer::on_spawn(self, id, type_name, x, y);
        id
    }

    fn find_spawn_cell(&self, x: u8, y: u8, profile: &crate::axioms::EntityProfile) -> Option<(u8, u8)> {
        if self.cell_composition.can_occupy(x, y, profile) {
            let slot = self.cell_composition.slot(x, y);
            let from = &profile.current_medium;
            let to = &slot.medium;
            if matches!(AxiomEngine::traverse(profile, from, to), Traversal::Allowed) {
                return Some((x, y));
            }
        }
        None
    }

    fn nearest_vacant_cell(
        &self,
        x: u8,
        y: u8,
        profile: &crate::axioms::EntityProfile,
    ) -> Option<(u8, u8)> {
        use crate::world_rules::{GRID_HEIGHT, GRID_WIDTH};
        for r in 1..8u8 {
            for dx in -(r as i16)..=(r as i16) {
                for dy in -(r as i16)..=(r as i16) {
                    if dx.unsigned_abs().max(dy.unsigned_abs()) != r as u16 {
                        continue;
                    }
                    let nx = x as i16 + dx;
                    let ny = y as i16 + dy;
                    if nx < 0
                        || ny < 0
                        || nx >= GRID_WIDTH as i16
                        || ny >= GRID_HEIGHT as i16
                    {
                        continue;
                    }
                    let ux = nx as u8;
                    let uy = ny as u8;
                    if !self.cell_composition.can_occupy(ux, uy, profile) {
                        continue;
                    }
                    let to = &self.cell_composition.slot(ux, uy).medium;
                    if matches!(
                        AxiomEngine::traverse(profile, &profile.current_medium, to),
                        Traversal::Allowed
                    ) {
                        return Some((ux, uy));
                    }
                }
            }
        }
        None
    }

    fn index_entity(&mut self, entity: &Entity, def: &CardDef) {
        let mut tags = def.tags.clone();
        if !tags.iter().any(|t| t == &entity.type_name) {
            tags.push(entity.type_name.clone());
        }
        if entity.is_corpse {
            tags.push("corpse".to_string());
        }
        self.spatial_index.insert(&IndexedEntity {
            id: entity.id,
            x: entity.x,
            y: entity.y,
            tags,
        });
    }

    pub fn reindex_entity(&mut self, id: EntityId) {
        if let Some(entity) = self.entities.get(&id).cloned() {
            if let Some(def) = self.card_defs.get(&entity.type_name).cloned() {
                self.spatial_index.remove(id);
                self.index_entity(&entity, &def);
            }
        }
    }

    pub fn move_entity(&mut self, id: EntityId, new_x: u8, new_y: u8) -> MoveResult {
        use crate::world_rules::{GRID_HEIGHT, GRID_WIDTH};
        let old = self
            .entities
            .get(&id)
            .map(|e| (e.x, e.y))
            .unwrap_or((0, 0));
        let new_x = new_x.clamp(0, GRID_WIDTH - 1);
        let new_y = new_y.clamp(0, GRID_HEIGHT - 1);
        if old == (new_x, new_y) {
            return MoveResult::NoOp;
        }

        let old_entity = match self.entities.get(&id).cloned() {
            Some(e) => e,
            None => return MoveResult::Blocked,
        };
        let profile = old_entity.profile.clone();

        let dest_slot = self.cell_composition.slot(new_x, new_y);
        let to_medium = dest_slot.medium.clone();
        if !matches!(
            AxiomEngine::traverse(&profile, &profile.current_medium, &to_medium),
            Traversal::Allowed
        ) {
            return MoveResult::Blocked;
        }
        if !matches!(
            AxiomEngine::compose(dest_slot, &profile),
            Composition::Allowed { .. }
        ) {
            return MoveResult::Blocked;
        }

        let (old_x, old_y) = old;
        self.cell_composition
            .vacate_entity(old_x, old_y, &old_entity);

        if let Some(entity) = self.entities.get_mut(&id) {
            entity.x = new_x;
            entity.y = new_y;
            entity.profile.current_medium = to_medium.clone();
            self.spatial_index.move_entity(id, entity.x, entity.y);
        }

        if let Some(entity) = self.entities.get(&id).cloned() {
            self.cell_composition
                .occupy_entity(new_x, new_y, &entity);
        }

        AxiomEngine::trace(
            &mut self.causal_storage,
            CausalEvent {
                tick: self.tick_count,
                cause_entity_id: id.0,
                cause_tag: "move".into(),
                effect_entity_id: id.0,
                effect_description: format!("moved({old_x},{old_y}→{new_x},{new_y})"),
            },
        );

        let new_pos = (new_x, new_y);
        if old != new_pos {
            crate::sim_observer::on_move(self, id, old, new_pos);
            let duration_per_step = self
                .entities
                .get(&id)
                .map(|e| e.profile.move_speed)
                .unwrap_or(0.25);
            self.pending_move_anims.push(crate::sim_events::MoveAnimEvent {
                entity_id: id,
                from_x: old_x,
                from_y: old_y,
                to_x: new_x,
                to_y: new_y,
                duration_per_step,
            });
        }
        MoveResult::Moved
    }

    pub fn remove_entity(&mut self, id: EntityId) {
        let entity = self.entities.get(&id).cloned();
        if let Some(ref e) = entity {
            self.cell_composition.vacate_entity(e.x, e.y, e);
            crate::sim_observer::on_despawn(self, &e.type_name, e.x, e.y);
            AxiomEngine::trace(
                &mut self.causal_storage,
                CausalEvent {
                    tick: self.tick_count,
                    cause_entity_id: id.0,
                    cause_tag: "remove".into(),
                    effect_entity_id: id.0,
                    effect_description: "removed".into(),
                },
            );
        }
        self.spatial_index.remove(id);
        self.entities.remove(&id);
    }

    /// Build Godot `world_rules_terrain.gd` fixed map ecology and sync derived cell sets.
    pub fn ensure_map_ecology(&mut self) {
        use crate::world_rules::{GRID_HEIGHT, GRID_WIDTH};
        self.ecology.ensure();
        self.sync_terrain_cell_sets();
        let mut mediums = Vec::with_capacity(GRID_HEIGHT as usize * GRID_WIDTH as usize);
        for y in 0..GRID_HEIGHT {
            for x in 0..GRID_WIDTH {
                let terrain = crate::terrain::terrain_at(self, x, y);
                mediums.push(crate::axioms::profile::medium_for_cell(terrain));
            }
        }
        let mut idx = 0;
        for y in 0..GRID_HEIGHT {
            for x in 0..GRID_WIDTH {
                self.cell_composition.grid[y as usize][x as usize].medium =
                    mediums[idx].clone();
                idx += 1;
            }
        }
    }

    fn sync_terrain_cell_sets(&mut self) {
        self.pool_cells = self.ecology.pool_cells.clone();
    }

    pub fn mark_river(&mut self, x: u8, y: u8) {
        self.river_cells.insert((x, y));
    }

    /// Ad-hoc pool for unit tests; also updates ecology when built.
    pub fn mark_pool(&mut self, x: u8, y: u8) {
        self.pool_cells.insert((x, y));
        if self.ecology.ready {
            self.ecology.pool_cells.insert((x, y));
        }
    }

    pub fn mark_fire(&mut self, x: u8, y: u8) {
        self.fire_cells.insert((x, y));
    }

    pub fn count_type(&self, type_name: &str) -> usize {
        self.entities
            .values()
            .filter(|e| e.type_name == type_name && !e.is_corpse)
            .count()
    }

    pub fn sheep_count(&self) -> usize {
        self.count_type("sheep")
    }

    pub fn wolf_count(&self) -> usize {
        self.count_type("wolf")
    }

    pub fn grass_count(&self) -> usize {
        self.count_type("grass")
    }

    pub fn entities_at(&self, x: u8, y: u8) -> Vec<EntityId> {
        self.entities
            .values()
            .filter(|e| e.x == x && e.y == y)
            .map(|e| e.id)
            .collect()
    }

    pub fn has_tag_at(&self, x: u8, y: u8, tag: &str) -> bool {
        self.entities_at(x, y).iter().any(|id| {
            self.entities
                .get(id)
                .and_then(|e| self.card_defs.get(&e.type_name))
                .is_some_and(|d| crate::world_rules::card_has_tag(d, tag))
        })
    }

    pub fn active_entity_ids(&self) -> Vec<EntityId> {
        self.entities
            .values()
            .filter(|e| e.is_autonomous(&self.card_defs))
            .map(|e| e.id)
            .collect()
    }

    pub fn run_ticks(&mut self, n: u64) {
        for _ in 0..n {
            self.tick_once();
        }
    }

    pub fn tick_once(&mut self) -> Vec<crate::sim_events::MoveAnimEvent> {
        // DEBUG
        if self.tick_count <= 3 {
            let _ = std::fs::write("E:/debug_tick.log", format!("tick_once tick={}\n", self.tick_count));
        }
        crate::systems::main_tick::main_tick(self, TICK_SECONDS);
        // Headless sim loops (bench) do not drain UI events; drop silent backlog.
        if self.pending_events.len() > 512 {
            self.pending_events.clear();
        }
        std::mem::take(&mut self.pending_move_anims)
    }

    pub fn drain_pending_events(&mut self) -> Vec<crate::sim_events::SimEvent> {
        std::mem::take(&mut self.pending_events)
    }
}

pub fn drain_pending_events(world: &mut WorldState) -> Vec<crate::sim_events::SimEvent> {
    world.drain_pending_events()
}

pub fn demo_world() -> WorldState {
    let mut world = WorldState::from_card_defs_file(crate::assets_util::card_defs_path());
    for x in 0..crate::world_rules::GRID_WIDTH {
        world.mark_river(x, 0);
    }
    for x in 5..15 {
        for y in 5..12 {
            world.spawn("grass", x, y);
        }
    }
    for x in 20..26 {
        world.spawn("sheep", x, 10);
    }
    for x in 28..32 {
        world.spawn("wolf", x, 10);
    }
    world.spawn("fox", 15, 8);
    world.mark_pool(8, 15);
    world.mark_pool(9, 15);
    world.spawn("oak", 10, 14);
    world
}

pub fn empty_world() -> WorldState {
    WorldState::from_card_defs_file(crate::assets_util::card_defs_path())
}

pub fn regen_due(world: &WorldState) -> bool {
    world.tick_count > 0
        && world.tick_count % crate::world_rules::GRASS_REGEN_INTERVAL == 0
}

pub fn reproduction_allowed(world: &WorldState) -> bool {
    world.tick_count % crate::world_rules::REPRO_COOLDOWN_TICKS == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_world_entity_count_after_1000_ticks() {
        let mut w = demo_world();
        w.run_ticks(1000);
        eprintln!("demo entities after 1000 ticks: {}", w.entities.len());
        assert!(w.entities.len() < 800);
        assert!(w.entities.len() < 500);
    }
}
