use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct CardDef {
    pub type_name: String,
    pub display_name: String,
    pub icon: String,
    pub tags: Vec<String>,
    pub color: (u8, u8, u8, u8),
    pub hp: i32,
    pub is_rooted: bool,
}

impl CardDef {
    pub fn has_tag(&self, tag: &str) -> bool {
        crate::world_rules::card_has_tag(self, tag)
    }

    pub fn color_f32(&self) -> (f32, f32, f32, f32) {
        let (r, g, b, a) = self.color;
        (r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, a as f32 / 255.0)
    }
}

pub fn load_card_defs(path: impl AsRef<Path>) -> Vec<CardDef> {
    let content = fs::read_to_string(path.as_ref()).expect("failed to read card_defs.ron");
    ron::from_str(&content).expect("failed to parse card_defs.ron")
}

pub fn card_defs_map(defs: &[CardDef]) -> HashMap<String, CardDef> {
    defs.iter()
        .map(|d| (d.type_name.clone(), d.clone()))
        .collect()
}

pub fn load_card_defs_map(path: impl AsRef<Path>) -> HashMap<String, CardDef> {
    card_defs_map(&load_card_defs(path))
}
