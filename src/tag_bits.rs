//! TagBits — u64 bitmask for O(1) tag queries.
//! All boolean tags (predator, herbivore, etc.) get one bit.
//! KV tags (meat_yield:3, max_starve:10) remain in `CardDef.tags: Vec<String>`.

use std::collections::HashMap;
use std::sync::LazyLock;

// ============================================================================
// TagBits / TagBit types
// ============================================================================

/// A single tag bit (power-of-two u64).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TagBit(pub u64);

/// A bitmask of boolean tags.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TagBits(pub u64);

impl TagBits {
    /// Test whether a given bit is set.
    #[inline]
    pub fn has(self, bit: TagBit) -> bool {
        self.0 & bit.0 != 0
    }

    /// Test by string — looks up the string→bit mapping.
    /// Returns false for unknown or KV-style tags.
    #[inline]
    pub fn has_str(self, tag: &str) -> bool {
        tag_to_bit(tag).is_some_and(|b| self.has(b))
    }

    /// Build from a slice of string tags.
    pub fn from_tags(tags: &[String]) -> Self {
        let mut bits = 0u64;
        for t in tags {
            if let Some(bit) = tag_to_bit(t) {
                bits |= bit.0;
            }
        }
        TagBits(bits)
    }
}

// ============================================================================
// Tag bit constants (one bit per boolean tag used in card_has_tag)
// ============================================================================

// --- Role / ecological niche ---
pub const TAG_BEING: TagBit = TagBit(1 << 0);
pub const TAG_ANIMAL: TagBit = TagBit(1 << 1);
pub const TAG_PREDATOR: TagBit = TagBit(1 << 2);
pub const TAG_MESOPREDATOR: TagBit = TagBit(1 << 3);
pub const TAG_HERBIVORE: TagBit = TagBit(1 << 4);
pub const TAG_ACTOR: TagBit = TagBit(1 << 5);
pub const TAG_PLAYER: TagBit = TagBit(1 << 6);

// --- Body / size ---
pub const TAG_JUVENILE: TagBit = TagBit(1 << 7);
pub const TAG_SMALL_PREY: TagBit = TagBit(1 << 8);
pub const TAG_LARGE_PREY: TagBit = TagBit(1 << 9);
pub const TAG_SMALL_HERBIVORE: TagBit = TagBit(1 << 10);
pub const TAG_TOUGH: TagBit = TagBit(1 << 11);
pub const TAG_BODY_TINY: TagBit = TagBit(1 << 12);
pub const TAG_BODY_SMALL: TagBit = TagBit(1 << 13);
pub const TAG_BODY_MEDIUM: TagBit = TagBit(1 << 14);
pub const TAG_BODY_LARGE: TagBit = TagBit(1 << 15);
pub const TAG_BODY_HUGE: TagBit = TagBit(1 << 16);

// --- Locomotion / medium ---
pub const TAG_AQUATIC: TagBit = TagBit(1 << 17);
pub const TAG_BURROWER: TagBit = TagBit(1 << 18);
pub const TAG_SESSILE: TagBit = TagBit(1 << 19);
pub const TAG_FLOATING: TagBit = TagBit(1 << 20);

// --- Social / grouping ---
pub const TAG_FLOCKING: TagBit = TagBit(1 << 21);
pub const TAG_PACK_HUNTER: TagBit = TagBit(1 << 22);

// --- Diet / food ---
pub const TAG_GRASS: TagBit = TagBit(1 << 23);
pub const TAG_OMNIVORE_SMALL: TagBit = TagBit(1 << 24);
pub const TAG_PERISHABLE: TagBit = TagBit(1 << 25);
pub const TAG_CORPSE: TagBit = TagBit(1 << 26);
pub const TAG_FOOD_SOURCE: TagBit = TagBit(1 << 27);
pub const TAG_PROFIFIC: TagBit = TagBit(1 << 28);
pub const TAG_PRIMARY_PRODUCER: TagBit = TagBit(1 << 29);

// --- Cover / shelter ---
pub const TAG_COVER_USER: TagBit = TagBit(1 << 30);
pub const TAG_COVER_SMALL: TagBit = TagBit(1 << 31);
pub const TAG_DEN_WOLF: TagBit = TagBit(1 << 32);

// --- Terrain / flora ---
pub const TAG_BUSH: TagBit = TagBit(1 << 33);
pub const TAG_NUT_PRODUCER: TagBit = TagBit(1 << 34);
pub const TAG_CONE_PRODUCER: TagBit = TagBit(1 << 35);
pub const TAG_FOREST: TagBit = TagBit(1 << 36);
pub const TAG_UNDERGROUND_CROP: TagBit = TagBit(1 << 37);

// --- Item / weapon ---
pub const TAG_SHARP: TagBit = TagBit(1 << 38);
pub const TAG_SCAVENGER: TagBit = TagBit(1 << 39);

// --- Misc ---
pub const TAG_CAMP_ANCHOR: TagBit = TagBit(1 << 40);
pub const TAG_HEAT: TagBit = TagBit(1 << 41);
pub const TAG_CELL_OVERLAY: TagBit = TagBit(1 << 42);
pub const TAG_TRAIT_FRAIL: TagBit = TagBit(1 << 43);

// --- Forage targets ---
pub const TAG_FORAGES_BUSH: TagBit = TagBit(1 << 44);
pub const TAG_FORAGES_UNDERGROUND: TagBit = TagBit(1 << 45);

// --- Harvest ---
pub const TAG_HARVEST_PRODUCT_FISH_MEAT: TagBit = TagBit(1 << 46);
pub const TAG_FILTER_FEEDER: TagBit = TagBit(1 << 47);

// --- Composition (mass modifier) ---
pub const TAG_DENSE: TagBit = TagBit(1 << 48);
pub const TAG_LIGHT: TagBit = TagBit(1 << 49);

// --- Additional perception / behavior ---
pub const TAG_GRAZER: TagBit = TagBit(1 << 50);
pub const TAG_ROOTED: TagBit = TagBit(1 << 51);
pub const TAG_VOLANT: TagBit = TagBit(1 << 52);

// ============================================================================
// String → TagBit mapping
// ============================================================================

fn tag_to_bit_registry() -> &'static HashMap<&'static str, TagBit> {
    static REGISTRY: LazyLock<HashMap<&'static str, TagBit>> = LazyLock::new(|| {
        HashMap::from([
            ("being", TAG_BEING),
            ("animal", TAG_ANIMAL),
            ("predator", TAG_PREDATOR),
            ("mesopredator", TAG_MESOPREDATOR),
            ("herbivore", TAG_HERBIVORE),
            ("actor", TAG_ACTOR),
            ("player", TAG_PLAYER),
            ("juvenile", TAG_JUVENILE),
            ("smallPrey", TAG_SMALL_PREY),
            ("largePrey", TAG_LARGE_PREY),
            ("smallHerbivore", TAG_SMALL_HERBIVORE),
            ("tough", TAG_TOUGH),
            ("body.tiny", TAG_BODY_TINY),
            ("body.small", TAG_BODY_SMALL),
            ("body.medium", TAG_BODY_MEDIUM),
            ("body.large", TAG_BODY_LARGE),
            ("body.huge", TAG_BODY_HUGE),
            ("aquatic", TAG_AQUATIC),
            ("burrower", TAG_BURROWER),
            ("sessile", TAG_SESSILE),
            ("floating", TAG_FLOATING),
            ("flocking", TAG_FLOCKING),
            ("pack_hunter", TAG_PACK_HUNTER),
            ("grass", TAG_GRASS),
            ("omnivore.small", TAG_OMNIVORE_SMALL),
            ("perishable", TAG_PERISHABLE),
            ("corpse", TAG_CORPSE),
            ("foodSource", TAG_FOOD_SOURCE),
            ("prolific", TAG_PROFIFIC),
            ("primary_producer", TAG_PRIMARY_PRODUCER),
            ("cover_user", TAG_COVER_USER),
            ("cover.small", TAG_COVER_SMALL),
            ("den.wolf", TAG_DEN_WOLF),
            ("bush", TAG_BUSH),
            ("nut_producer", TAG_NUT_PRODUCER),
            ("cone_producer", TAG_CONE_PRODUCER),
            ("forest", TAG_FOREST),
            ("underground_crop", TAG_UNDERGROUND_CROP),
            ("sharp", TAG_SHARP),
            ("scavenger", TAG_SCAVENGER),
            ("camp.anchor", TAG_CAMP_ANCHOR),
            ("heat", TAG_HEAT),
            ("cell.overlay", TAG_CELL_OVERLAY),
            ("trait:frail", TAG_TRAIT_FRAIL),
            ("forages:bush", TAG_FORAGES_BUSH),
            ("forages:underground", TAG_FORAGES_UNDERGROUND),
            ("harvest_product:fishMeat", TAG_HARVEST_PRODUCT_FISH_MEAT),
            ("filter_feeder", TAG_FILTER_FEEDER),
            ("dense", TAG_DENSE),
            ("light", TAG_LIGHT),
            ("grazer", TAG_GRAZER),
            ("rooted", TAG_ROOTED),
            ("volant", TAG_VOLANT),
        ])
    });
    &REGISTRY
}

/// Convert a string tag name to its bit, if registered.
pub fn tag_to_bit(tag: &str) -> Option<TagBit> {
    tag_to_bit_registry().get(tag).copied()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_tags_is_zero() {
        let bits = TagBits::from_tags(&[]);
        assert_eq!(bits.0, 0);
    }

    #[test]
    fn known_tags_map_to_bits() {
        let bits = TagBits::from_tags(&[
            "predator".into(),
            "herbivore".into(),
        ]);
        assert!(bits.has(TAG_PREDATOR));
        assert!(bits.has(TAG_HERBIVORE));
        assert!(!bits.has(TAG_ANIMAL));
    }

    #[test]
    fn unknown_tags_ignored() {
        let bits = TagBits::from_tags(&["meat_yield:3".into(), "max_starve:5".into()]);
        assert_eq!(bits.0, 0);
    }

    #[test]
    fn has_str_works() {
        let bits = TagBits::from_tags(&["predator".into()]);
        assert!(bits.has_str("predator"));
        assert!(!bits.has_str("herbivore"));
        assert!(!bits.has_str("nonexistent"));
    }

    #[test]
    fn body_size_tags() {
        let bits = TagBits::from_tags(&["body.tiny".into()]);
        assert!(bits.has(TAG_BODY_TINY));
        assert!(!bits.has(TAG_BODY_SMALL));

        let bits2 = TagBits::from_tags(&["body.large".into()]);
        assert!(bits2.has(TAG_BODY_LARGE));
        assert!(!bits2.has(TAG_BODY_TINY));
    }

    #[test]
    fn all_bits_fit_in_u64() {
        // The highest bit index should be < 64
        let bits = TAG_VOLANT;
        assert!(bits.0.count_ones() == 1);
        assert!(bits.0.trailing_zeros() < 64);
    }
}
