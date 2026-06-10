//! Lightweight Rete-style rule index — tag → shared nodes → rules.
//! Godot `player_affordance` / WorldRules 的 Rust 侧索引层；不引入外部 crate。

use std::collections::HashMap;

use crate::card_def::CardDef;
use crate::spatial_index::EntityId;
use crate::world_rules::{
    card_has_capability, card_has_tag, can_forage, can_hunt_def, chebyshev_distance,
    ecology_was_fed_today, in_range, is_camp_fire_anchor, is_herbivore, is_hunt_target_for_pack,
    is_mesopredator, is_predator, pack_hunter_under_strength, BEHAVIOR_COVER_FORAGER,
    BEHAVIOR_HERBIVORE_GRAZER, BEHAVIOR_MESOPREDATOR_HUNT, BEHAVIOR_PREDATOR_DEN, HUNT_RANGE,
    PACK_MIN_STRENGTH,
};
use crate::world_state::WorldState;

pub type RuleId = u32;
pub type NodeId = u32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EcologyAction {
    Hunt,
    Stalk,
    FleeIfAlone,
    Graze,
    Eat,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SharedCondition {
    InRange { radius: u8 },
    HasTagOnSelf { tag: String },
    CountNearby { tag: String, max_count: usize },
    TargetHasTag { tag: String },
    NotFedToday,
    NearCampAnchor,
}

#[derive(Debug, Clone)]
pub struct SharedNode {
    pub id: NodeId,
    pub condition: SharedCondition,
    pub children: Vec<NodeId>,
    pub rules: Vec<RuleId>,
}

#[derive(Debug, Clone)]
pub struct RuleDef {
    pub id: RuleId,
    pub name: &'static str,
    pub required_tags: Vec<&'static str>,
    pub shared_node_path: Vec<NodeId>,
    pub action: EcologyAction,
    pub behavior_key: Option<&'static str>,
}

#[derive(Debug, Clone)]
pub struct RuleIndex {
    pub tag_index: HashMap<String, Vec<RuleId>>,
    pub shared_nodes: Vec<SharedNode>,
    pub rules: Vec<RuleDef>,
    node_by_condition: HashMap<SharedCondition, NodeId>,
}

impl RuleIndex {
    pub fn build() -> Self {
        let mut index = Self {
            tag_index: HashMap::new(),
            shared_nodes: Vec::new(),
            rules: Vec::new(),
            node_by_condition: HashMap::new(),
        };
        index.register_ecology_rules();
        index.register_behavior_rules();
        index
    }

    fn intern_node(&mut self, condition: SharedCondition) -> NodeId {
        if let Some(&id) = self.node_by_condition.get(&condition) {
            return id;
        }
        let id = self.shared_nodes.len() as NodeId;
        self.shared_nodes.push(SharedNode {
            id,
            condition: condition.clone(),
            children: Vec::new(),
            rules: Vec::new(),
        });
        self.node_by_condition.insert(condition, id);
        id
    }

    fn link_child(&mut self, parent: NodeId, child: NodeId) {
        if let Some(node) = self.shared_nodes.get_mut(parent as usize) {
            if !node.children.contains(&child) {
                node.children.push(child);
            }
        }
    }

    fn attach_rule_to_nodes(&mut self, rule_id: RuleId, path: &[NodeId]) {
        for &nid in path {
            if let Some(node) = self.shared_nodes.get_mut(nid as usize) {
                if !node.rules.contains(&rule_id) {
                    node.rules.push(rule_id);
                }
            }
        }
    }

    fn register_rule(
        &mut self,
        name: &'static str,
        required_tags: &[&'static str],
        path: Vec<NodeId>,
        action: EcologyAction,
        behavior_key: Option<&'static str>,
    ) -> RuleId {
        let id = self.rules.len() as RuleId;
        for tag in required_tags {
            self.tag_index
                .entry((*tag).to_string())
                .or_default()
                .push(id);
        }
        self.rules.push(RuleDef {
            id,
            name,
            required_tags: required_tags.to_vec(),
            shared_node_path: path.clone(),
            action,
            behavior_key,
        });
        self.attach_rule_to_nodes(id, &path);
        id
    }

    fn register_ecology_rules(&mut self) {
        let in_hunt_range = self.intern_node(SharedCondition::InRange {
            radius: HUNT_RANGE,
        });
        let prey_tag = self.intern_node(SharedCondition::TargetHasTag {
            tag: "herbivore".into(),
        });
        self.link_child(in_hunt_range, prey_tag);

        let has_weapon = self.intern_node(SharedCondition::HasTagOnSelf {
            tag: "sharp".into(),
        });
        self.link_child(prey_tag, has_weapon);

        let pack_alone = self.intern_node(SharedCondition::CountNearby {
            tag: "predator".to_string(),
            max_count: PACK_MIN_STRENGTH,
        });
        let near_camp = self.intern_node(SharedCondition::NearCampAnchor);
        self.link_child(pack_alone, near_camp);

        let grass_near = self.intern_node(SharedCondition::InRange { radius: 6 });
        let grass_tag = self.intern_node(SharedCondition::TargetHasTag {
            tag: "grass".into(),
        });
        self.link_child(grass_near, grass_tag);

        let not_fed = self.intern_node(SharedCondition::NotFedToday);
        self.link_child(grass_tag, not_fed);

        self.register_rule(
            "predator_near_prey_hunt",
            &["predator"],
            vec![in_hunt_range, prey_tag],
            EcologyAction::Hunt,
            None,
        );
        self.register_rule(
            "predator_near_prey_weapon_stalk",
            &["predator", "mesopredator"],
            vec![in_hunt_range, prey_tag, has_weapon],
            EcologyAction::Stalk,
            None,
        );
        self.register_rule(
            "predator_near_camp_alone_flee",
            &["predator", "pack_hunter"],
            vec![pack_alone, near_camp],
            EcologyAction::FleeIfAlone,
            None,
        );
        self.register_rule(
            "prey_near_grass_graze",
            &["herbivore", "grazer"],
            vec![grass_near, grass_tag],
            EcologyAction::Graze,
            None,
        );
        self.register_rule(
            "herbivore_near_grass_hungry_eat",
            &["herbivore"],
            vec![grass_near, grass_tag, not_fed],
            EcologyAction::Eat,
            None,
        );
    }

    fn register_behavior_rules(&mut self) {
        self.register_rule(
            "behavior_predator_den",
            &["predator"],
            vec![],
            EcologyAction::Hunt,
            Some(BEHAVIOR_PREDATOR_DEN),
        );
        self.register_rule(
            "behavior_mesopredator_hunt",
            &["mesopredator"],
            vec![],
            EcologyAction::Stalk,
            Some(BEHAVIOR_MESOPREDATOR_HUNT),
        );
        self.register_rule(
            "behavior_cover_forager",
            &["cover_user", "burrower"],
            vec![],
            EcologyAction::Graze,
            Some(BEHAVIOR_COVER_FORAGER),
        );
        self.register_rule(
            "behavior_herbivore_grazer",
            &["herbivore", "grazer"],
            vec![],
            EcologyAction::Graze,
            Some(BEHAVIOR_HERBIVORE_GRAZER),
        );
    }

    pub fn rules_for_tag(&self, tag: &str) -> &[RuleId] {
        self.tag_index.get(tag).map(|v| v.as_slice()).unwrap_or(&[])
    }

    pub fn shared_node(&self, id: NodeId) -> Option<&SharedNode> {
        self.shared_nodes.get(id as usize)
    }

    pub fn rule(&self, id: RuleId) -> Option<&RuleDef> {
        self.rules.get(id as usize)
    }

    pub fn rules_for_def(&self, def: &CardDef) -> Vec<RuleId> {
        let mut seen = Vec::new();
        for tag in def_tags(def) {
            for &rid in self.rules_for_tag(&tag) {
                if !seen.contains(&rid) {
                    seen.push(rid);
                }
            }
        }
        seen
    }

    pub fn evaluate_action(
        &self,
        world: &WorldState,
        actor_id: EntityId,
        action: EcologyAction,
    ) -> bool {
        let Some(actor) = world.entities.get(&actor_id) else {
            return false;
        };
        let Some(actor_def) = world.card_defs.get(&actor.type_name) else {
            return false;
        };
        let pack_size = pack_size_for(world, actor_def);
        for rid in self.rules_for_def(actor_def) {
            let Some(rule) = self.rule(rid) else {
                continue;
            };
            if rule.action != action {
                continue;
            }
            if self.eval_rule_path(world, actor_id, actor_def, pack_size, &rule.shared_node_path) {
                return true;
            }
        }
        false
    }

    pub fn behavior_key_for(&self, def: &CardDef, type_name: &str) -> Option<&'static str> {
        const PRIORITY: [&str; 4] = [
            BEHAVIOR_PREDATOR_DEN,
            BEHAVIOR_MESOPREDATOR_HUNT,
            BEHAVIOR_COVER_FORAGER,
            BEHAVIOR_HERBIVORE_GRAZER,
        ];
        for key in PRIORITY {
            for rid in self.rules_for_def(def) {
                let Some(rule) = self.rule(rid) else {
                    continue;
                };
                if rule.behavior_key == Some(key) && behavior_rule_matches(def, type_name, key) {
                    return Some(key);
                }
            }
        }
        None
    }

    fn eval_rule_path(
        &self,
        world: &WorldState,
        actor_id: EntityId,
        actor_def: &CardDef,
        pack_size: usize,
        path: &[NodeId],
    ) -> bool {
        path.iter()
            .all(|&nid| self.eval_node(world, actor_id, actor_def, pack_size, nid))
    }

    fn eval_node(
        &self,
        world: &WorldState,
        actor_id: EntityId,
        actor_def: &CardDef,
        pack_size: usize,
        node_id: NodeId,
    ) -> bool {
        let Some(node) = self.shared_node(node_id) else {
            return false;
        };
        let actor = &world.entities[&actor_id];
        match &node.condition {
            SharedCondition::InRange { radius } => {
                prey_in_range(world, actor_id, actor.x, actor.y, *radius, actor_def, pack_size)
                    .is_some()
                    || grass_in_range(world, actor_id, actor.x, actor.y, *radius)
            }
            SharedCondition::HasTagOnSelf { tag } => card_has_tag(actor_def, tag),
            SharedCondition::CountNearby { tag, max_count } => {
                let n = world
                    .query_near_filtered(actor.x, actor.y, tag, HUNT_RANGE, actor_id)
                    .len();
                n < *max_count
            }
            SharedCondition::TargetHasTag { tag } if tag == "herbivore" => {
                prey_in_range(world, actor_id, actor.x, actor.y, HUNT_RANGE, actor_def, pack_size)
                    .is_some()
            }
            SharedCondition::TargetHasTag { tag } if tag == "grass" => {
                grass_in_range(world, actor_id, actor.x, actor.y, 6)
            }
            SharedCondition::TargetHasTag { .. } => false,
            SharedCondition::NotFedToday => {
                !ecology_was_fed_today(actor, actor_def)
            }
            SharedCondition::NearCampAnchor => near_camp_anchor(world, actor.x, actor.y),
        }
    }
}

fn def_tags(def: &CardDef) -> Vec<String> {
    let mut tags: Vec<String> = def.tags.clone();
    tags.push(def.type_name.clone());
    tags
}

fn pack_size_for(world: &WorldState, actor_def: &CardDef) -> usize {
    if card_has_tag(actor_def, "pack_hunter") {
        world.count_by_tag("pack_hunter").max(1)
    } else {
        1
    }
}

fn prey_in_range(
    world: &WorldState,
    hunter_id: EntityId,
    x: u8,
    y: u8,
    radius: u8,
    hunter_def: &CardDef,
    pack_size: usize,
) -> Option<EntityId> {
    let candidates: Vec<EntityId> = world
        .query_near_filtered(x, y, "herbivore", radius, hunter_id)
        .into_iter()
        .chain(world.query_near_filtered(x, y, "smallPrey", radius, hunter_id))
        .collect();
    for prey_id in candidates {
        let prey = world.entities.get(&prey_id)?;
        if prey.is_corpse {
            continue;
        }
        let prey_def = world.card_defs.get(&prey.type_name)?;
        if !is_hunt_target_for_pack(hunter_def, prey_def, pack_size) {
            continue;
        }
        if prey.hidden_in_grass && chebyshev_distance(x, y, prey.x, prey.y) > 1 {
            continue;
        }
        return Some(prey_id);
    }
    None
}

fn grass_in_range(world: &WorldState, observer_id: EntityId, x: u8, y: u8, radius: u8) -> bool {
    !world
        .query_near_filtered(x, y, "foodSource", radius, observer_id)
        .is_empty()
        || !world
            .query_near_filtered(x, y, "grass", radius, observer_id)
            .is_empty()
}

fn near_camp_anchor(world: &WorldState, x: u8, y: u8) -> bool {
    for e in world.entities.values() {
        if let Some(def) = world.card_defs.get(&e.type_name) {
            if is_camp_fire_anchor(def) && in_range(x, y, e.x, e.y, 4) {
                return true;
            }
        }
    }
    false
}

fn behavior_rule_matches(def: &CardDef, type_name: &str, key: &str) -> bool {
    match key {
        BEHAVIOR_PREDATOR_DEN => {
            matches!(type_name, "wolf" | "wolfCub" | "fox" | "foxCub")
                && (is_predator(def) || is_mesopredator(def))
                || is_predator(def) && card_has_capability(def, "capability.hunt")
        }
        BEHAVIOR_MESOPREDATOR_HUNT => {
            is_mesopredator(def) && !card_has_capability(def, "capability.return_home")
        }
        BEHAVIOR_COVER_FORAGER => {
            card_has_tag(def, "cover_user")
                || card_has_tag(def, "burrower")
                || matches!(type_name, "fieldMouse" | "fieldMousePup" | "bambooRat")
        }
        BEHAVIOR_HERBIVORE_GRAZER => {
            is_herbivore(def)
                || can_forage(def)
                || card_has_tag(def, "grazer")
        }
        _ => false,
    }
}

// --- legacy mirrors for dual-track ---

pub fn legacy_should_hunt(world: &WorldState, actor_id: EntityId) -> bool {
    let Some(actor) = world.entities.get(&actor_id) else {
        return false;
    };
    let Some(def) = world.card_defs.get(&actor.type_name) else {
        return false;
    };
    if !can_hunt_def(def) {
        return false;
    }
    let pack = pack_size_for(world, def);
    prey_in_range(world, actor_id, actor.x, actor.y, HUNT_RANGE, def, pack).is_some()
}

pub fn legacy_should_stalk(world: &WorldState, actor_id: EntityId) -> bool {
    let Some(actor) = world.entities.get(&actor_id) else {
        return false;
    };
    let Some(def) = world.card_defs.get(&actor.type_name) else {
        return false;
    };
    legacy_should_hunt(world, actor_id)
        && (is_mesopredator(def) || card_has_tag(def, "sharp"))
}

pub fn legacy_should_flee_if_alone(world: &WorldState, actor_id: EntityId) -> bool {
    let Some(actor) = world.entities.get(&actor_id) else {
        return false;
    };
    let Some(def) = world.card_defs.get(&actor.type_name) else {
        return false;
    };
    if !card_has_tag(def, "pack_hunter") {
        return false;
    }
    let pack_defs: Vec<&CardDef> = world
        .entities
        .values()
        .filter(|e| e.type_name == def.type_name && !e.is_corpse)
        .filter_map(|e| world.card_defs.get(&e.type_name))
        .collect();
    pack_hunter_under_strength(&pack_defs) && near_camp_anchor(world, actor.x, actor.y)
}

pub fn legacy_should_graze(world: &WorldState, actor_id: EntityId) -> bool {
    let Some(actor) = world.entities.get(&actor_id) else {
        return false;
    };
    grass_in_range(world, actor_id, actor.x, actor.y, 6)
}

pub fn legacy_should_eat(world: &WorldState, actor_id: EntityId) -> bool {
    let Some(actor) = world.entities.get(&actor_id) else {
        return false;
    };
    let Some(def) = world.card_defs.get(&actor.type_name) else {
        return false;
    };
    legacy_should_graze(world, actor_id) && !ecology_was_fed_today(actor, def)
}

pub fn dual_track_action(
    index: &RuleIndex,
    world: &WorldState,
    actor_id: EntityId,
    action: EcologyAction,
    legacy: bool,
) -> bool {
    let rete = index.evaluate_action(world, actor_id, action);
    if rete == legacy {
        rete
    } else {
        legacy
    }
}

pub fn dual_track_hunt(index: &RuleIndex, world: &WorldState, actor_id: EntityId) -> bool {
    dual_track_action(
        index,
        world,
        actor_id,
        EcologyAction::Hunt,
        legacy_should_hunt(world, actor_id),
    )
}

pub fn dual_track_stalk(index: &RuleIndex, world: &WorldState, actor_id: EntityId) -> bool {
    dual_track_action(
        index,
        world,
        actor_id,
        EcologyAction::Stalk,
        legacy_should_stalk(world, actor_id),
    )
}

pub fn dual_track_flee_if_alone(index: &RuleIndex, world: &WorldState, actor_id: EntityId) -> bool {
    dual_track_action(
        index,
        world,
        actor_id,
        EcologyAction::FleeIfAlone,
        legacy_should_flee_if_alone(world, actor_id),
    )
}

pub fn dual_track_graze(index: &RuleIndex, world: &WorldState, actor_id: EntityId) -> bool {
    dual_track_action(
        index,
        world,
        actor_id,
        EcologyAction::Graze,
        legacy_should_graze(world, actor_id),
    )
}

pub fn dual_track_eat(index: &RuleIndex, world: &WorldState, actor_id: EntityId) -> bool {
    dual_track_action(
        index,
        world,
        actor_id,
        EcologyAction::Eat,
        legacy_should_eat(world, actor_id),
    )
}

static RULE_INDEX: std::sync::OnceLock<RuleIndex> = std::sync::OnceLock::new();

pub fn rule_index() -> &'static RuleIndex {
    RULE_INDEX.get_or_init(RuleIndex::build)
}

/// Step 3/4：RuleIndex 与 legacy 一致时用索引结果，否则回退 legacy。
pub fn merge_behavior_key(
    index: &RuleIndex,
    def: &CardDef,
    type_name: &str,
) -> &'static str {
    let legacy = crate::world_rules::ecosystem_behavior_key_legacy(def, type_name);
    if let Some(key) = index.behavior_key_for(def, type_name) {
        if key == legacy || legacy.is_empty() {
            return if legacy.is_empty() { key } else { legacy };
        }
    }
    legacy
}
