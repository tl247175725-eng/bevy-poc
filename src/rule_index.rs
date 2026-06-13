//! 规则索引——已掏空，等待需求匹配引擎。
//!
//! 所有行为规则已移除。RuleIndex 将在元动作 + 需求匹配引擎完成后，
//! 作为"标签→需求→元动作序列"的映射表重新实现。

use std::collections::HashMap;
use crate::card_def::CardDef;
use crate::spatial_index::EntityId;
use crate::world_state::WorldState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EcologyAction { Hunt, Stalk, FleeIfAlone, Graze, Eat }

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
pub struct SharedNode { pub id: u32, pub condition: SharedCondition, pub children: Vec<u32>, pub rules: Vec<u32> }

#[derive(Debug, Clone)]
pub struct RuleDef {
    pub id: u32, pub name: &'static str,
    pub required_tags: Vec<&'static str>,
    pub shared_node_path: Vec<u32>,
    pub action: EcologyAction,
    pub behavior_key: Option<&'static str>,
}

pub struct RuleIndex {
    pub tag_index: HashMap<String, Vec<u32>>,
    pub shared_nodes: Vec<SharedNode>,
    pub rules: Vec<RuleDef>,
    node_by_condition: HashMap<SharedCondition, u32>,
}

impl RuleIndex {
    pub fn build() -> Self {
        Self { tag_index: HashMap::new(), shared_nodes: Vec::new(), rules: Vec::new(), node_by_condition: HashMap::new() }
    }

    pub fn rules_for_tag(&self, _tag: &str) -> &[u32] { &[] }
    pub fn shared_node(&self, _id: u32) -> Option<&SharedNode> { None }
    pub fn rule(&self, _id: u32) -> Option<&RuleDef> { None }
    pub fn rules_for_def(&self, _def: &CardDef) -> Vec<u32> { Vec::new() }
    pub fn evaluate_action(&self, _world: &WorldState, _actor_id: EntityId, _action: EcologyAction) -> bool { false }
    pub fn behavior_key_for(&self, _def: &CardDef, _type_name: &str) -> Option<&'static str> { None }
}

static RULE_INDEX: std::sync::OnceLock<RuleIndex> = std::sync::OnceLock::new();

pub fn rule_index() -> &'static RuleIndex {
    RULE_INDEX.get_or_init(RuleIndex::build)
}
