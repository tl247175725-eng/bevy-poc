//! Player AI runtime state — brain output + task FSM.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SdtNeeds {
    pub autonomy: i32,
    pub competence: i32,
    pub relatedness: i32,
}

impl SdtNeeds {
    pub fn baseline() -> Self {
        Self {
            autonomy: 100,
            competence: 100,
            relatedness: 100,
        }
    }

    pub fn clamp(mut self) -> Self {
        self.autonomy = self.autonomy.clamp(0, 100);
        self.competence = self.competence.clamp(0, 100);
        self.relatedness = self.relatedness.clamp(0, 100);
        self
    }

    pub fn lowest_need(&self) -> &'static str {
        let mut best = "relatedness";
        let mut best_val = self.relatedness;
        if self.autonomy < best_val {
            best = "autonomy";
            best_val = self.autonomy;
        }
        if self.competence < best_val {
            best = "competence";
        }
        best
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AffordanceEntry {
    pub key: String,
    pub need: String,
    pub score: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskPhase {
    Plan,
    Move,
    Pickup,
    MoveTo,
    Drop,
    Act,
    Done,
    Fail,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerTask {
    pub task_type: String,
    pub phase: TaskPhase,
    pub stone1: Option<u64>,
    pub stone2: Option<u64>,
    pub target: Option<u64>,
    pub hit_count: u32,
    pub fail_reason: String,
}

impl Default for PlayerTask {
    fn default() -> Self {
        Self {
            task_type: String::new(),
            phase: TaskPhase::Plan,
            stone1: None,
            stone2: None,
            target: None,
            hit_count: 0,
            fail_reason: String::new(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlayerMind {
    pub hunger: f32,
    pub tools: Vec<String>,
    pub runtime_tags: HashMap<String, bool>,
    pub affordances: HashMap<String, AffordanceEntry>,
    pub affordance_hold: HashMap<String, u8>,
    pub affordance_last: HashMap<String, AffordanceEntry>,
    pub needs: SdtNeeds,
    pub top_desire: String,
    pub intent_key: String,
    pub task: Option<PlayerTask>,
    pub task_cooldowns: HashMap<String, f32>,
    pub goal_text: String,
    pub state_label: String,
    pub auto_plan: bool,
    pub recent_hunt_success: bool,
    pub threat_level: u8,
}

impl PlayerMind {
    pub fn new_spawn() -> Self {
        Self {
            hunger: 25.0,
            auto_plan: true,
            needs: SdtNeeds::baseline(),
            ..Default::default()
        }
    }

    pub fn mind_display_line(&self) -> String {
        if !self.goal_text.is_empty() {
            return format!("目标：{}", self.goal_text);
        }
        if !self.top_desire.is_empty() {
            return format!("意向：{}", desire_label(&self.top_desire));
        }
        "观望中".into()
    }
}

pub fn desire_label(key: &str) -> String {
    match key {
        "forage" => "觅食".into(),
        "craft_knife" => "制作小刀".into(),
        "craft_spear" => "制作长矛".into(),
        "build_hut" => "搭建草棚".into(),
        "collect_fuel" => "收集燃料".into(),
        "hunt_armed" | "hunt_bare" => "狩猎".into(),
        "flee_threat" => "躲避威胁".into(),
        other => other.to_string(),
    }
}

pub fn task_on_cooldown(mind: &PlayerMind, task_type: &str) -> bool {
    mind
        .task_cooldowns
        .get(task_type)
        .copied()
        .unwrap_or(0.0)
        > 0.0
}

pub fn set_task_cooldown(mind: &mut PlayerMind, task_type: &str, secs: f32) {
    mind.task_cooldowns.insert(task_type.to_string(), secs);
}

pub fn tick_cooldowns(mind: &mut PlayerMind, delta: f32) {
    for v in mind.task_cooldowns.values_mut() {
        *v = (*v - delta).max(0.0);
    }
}
