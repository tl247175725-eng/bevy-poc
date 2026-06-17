use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::tags::{TagBits, TagRegistry};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct CardDef {
    pub type_name: String,
    pub display_name: String,
    pub icon: String,
    pub tags: Vec<String>,
    #[serde(skip, default = "TagBits::new")]
    pub tag_bits: TagBits,
    pub color: (u8, u8, u8, u8),
    pub hp: i32,
    pub is_rooted: bool,
    /// 卡的量（堆叠计数），默认 1。
    #[serde(default = "default_quantity")]
    pub quantity: u32,
}

fn default_quantity() -> u32 {
    1
}

impl CardDef {
    pub fn has_tag(&self, tag: &str) -> bool {
        crate::world_rules::card_has_tag(self, tag)
    }

    /// 通过 TagRegistry 直接查询位掩码（无字符串 fallback）。
    pub fn has_tag_from_registry(&self, registry: &TagRegistry, name: &str) -> bool {
        registry
            .name_to_bit
            .get(name)
            .map(|&bit| self.tag_bits.has(bit))
            .unwrap_or(false)
    }

    pub fn color_f32(&self) -> (f32, f32, f32, f32) {
        let (r, g, b, a) = self.color;
        (r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, a as f32 / 255.0)
    }
}

/// 将 CardDef 的 tags 字符串向量转为 tag_bits 位掩码。
/// 仅在 TagRegistry 初始化后调用。
pub fn init_tag_bits(defs: &mut [CardDef], registry: &TagRegistry) {
    for def in defs.iter_mut() {
        def.tag_bits = TagBits::from_tag_names(&def.tags, registry);
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

/// 加载 CardDef 并自动填充 tag_bits（TagRegistry 就位后调用）。
/// TagRegistry 未就位时 tag_bits 保持全零。
pub fn load_card_defs_with_tags(path: impl AsRef<Path>) -> Vec<CardDef> {
    let mut defs = load_card_defs(path);
    if let Some(registry) = crate::world_rules::TAG_REGISTRY.get() {
        for def in &mut defs {
            def.tag_bits = TagBits::from_tag_names(&def.tags, registry);
        }
    }
    defs
}

pub fn load_card_defs_map(path: impl AsRef<Path>) -> HashMap<String, CardDef> {
    card_defs_map(&load_card_defs(path))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn carddef_tagbits_empty_by_default() {
        let def = CardDef {
            type_name: "test".into(),
            display_name: "Test".into(),
            icon: "".into(),
            tags: vec![],
            tag_bits: TagBits::new(),
            color: (255, 255, 255, 255),
            hp: 1,
            is_rooted: false,
            quantity: 1,
        };
        // tag_bits should be all zeros
        for i in 0u16..512 {
            assert!(!def.tag_bits.has(i));
        }
    }

    #[test]
    fn carddef_tagbits_from_registry() {
        // 初始化 TagRegistry
        crate::world_rules::init_tag_registry();

        let defs = load_card_defs_with_tags("assets/card_defs.ron");
        let registry = crate::world_rules::TAG_REGISTRY.get().unwrap();

        // abyss_pool 有 "habitat:aquatic" 标签 — 确认 bit 被设置
        let pool = defs.iter().find(|d| d.type_name == "abyss_pool").unwrap();
        let aquatic_bit = registry.name_to_bit.get("habitat:aquatic")
            .expect("habitat:aquatic 应在 TagRegistry 中");
        assert!(pool.tag_bits.has(*aquatic_bit), "abyss_pool tag_bits 应包含 habitat:aquatic");

        // shallow_water 也有 "habitat:aquatic"
        let sw = defs.iter().find(|d| d.type_name == "shallow_water").unwrap();
        assert!(sw.tag_bits.has(*aquatic_bit), "shallow_water tag_bits 应包含 habitat:aquatic");

        // nanmu_tree 有 "habitat:forest" → bit 非零
        let nanmu = defs.iter().find(|d| d.type_name == "nanmu_tree").unwrap();
        let forest_bit = registry.name_to_bit.get("habitat:forest")
            .expect("habitat:forest 应在 TagRegistry 中");
        assert!(nanmu.tag_bits.has(*forest_bit), "nanmu_tree tag_bits 应包含 habitat:forest");

        // 验证新性状标签格式（habitat:xxx / diet:xxx / growth:xxx）正确注册
        for def in &defs {
            for tag in &def.tags {
                if let Some(&bit) = registry.name_to_bit.get(tag.as_str()) {
                    assert!(def.tag_bits.has(bit), "{} tag_bits 应包含 {} (bit {})", def.type_name, tag, bit);
                }
            }
        }
    }
}
