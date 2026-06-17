use crate::tags::{TagBits, tag};

/// Terrain resistance level for movement.
pub enum TerrainCost {
    Free,          // 1 tick to cross
    Slow,          // 2 ticks
    Hard,          // 4 ticks
    Lethal,        // can enter, but takes damage every tick
}

/// Derive terrain resistance from target cell terrain type + entity tags.
pub fn terrain_resistance(
    terrain_type: &str,
    entity_tags: &TagBits,
) -> TerrainCost {
    let is_aquatic = entity_tags.has(tag::HAB_AQUATIC.bit)
        || entity_tags.has(tag::CAP_SWIM.bit);
    let is_large = entity_tags.has(tag::SIZE_LARGE.bit)
        || entity_tags.has(tag::SIZE_HUGE.bit);
    let is_fish = entity_tags.has(tag::PLAN_FISH.bit);
    let is_serpentine = entity_tags.has(tag::PLAN_SERPENTINE.bit);

    match terrain_type {
        "abyss_pool" => {
            if is_fish { TerrainCost::Free }
            else if is_aquatic { TerrainCost::Slow }
            else { TerrainCost::Lethal }
        }
        "shallow_water" => {
            if is_fish || is_aquatic { TerrainCost::Free }
            else if is_large { TerrainCost::Slow }      // large animals wade
            else { TerrainCost::Hard }                    // small animals struggle
        }
        "wetland" => {
            if is_aquatic || is_serpentine { TerrainCost::Free }
            else if is_large { TerrainCost::Free }        // large animals unaffected
            else { TerrainCost::Slow }                    // small animals slowed by mud
        }
        "grassland" => {
            if is_fish { TerrainCost::Lethal }            // fish suffocate on land
            else { TerrainCost::Free }
        }
        "broadleaf_forest" => {
            if is_fish { TerrainCost::Lethal }            // fish suffocate on land
            else if is_large { TerrainCost::Slow }        // large animals slowed by trees
            else { TerrainCost::Free }
        }
        "foothills" => {
            if is_fish { TerrainCost::Lethal }
            else { TerrainCost::Slow }                    // hills slow everyone
        }
        "cliff" => {
            if is_fish { TerrainCost::Lethal }
            else { TerrainCost::Hard }                    // cliffs are very hard to traverse
        }
        _ => {
            if is_fish { TerrainCost::Lethal }            // unknown land terrain lethal for fish
            else { TerrainCost::Free }
        }
    }
}

/// Resistance -> movement tick cost.
pub fn move_cost_ticks(cost: &TerrainCost) -> u32 {
    match cost {
        TerrainCost::Free => 1,
        TerrainCost::Slow => 2,
        TerrainCost::Hard => 4,
        TerrainCost::Lethal => 1, // can enter, but takes damage
    }
}

/// Damage per tick on lethal terrain.
pub fn lethal_terrain_damage() -> i32 {
    // can later be driven by meta_values
    2
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tags::{TagBits, tag};

    fn make_tags(bits: &[u16]) -> TagBits {
        let mut t = TagBits::new();
        for &b in bits {
            t.set(b);
        }
        t
    }

    #[test]
    fn fish_in_deep_water_free() {
        let tags = make_tags(&[tag::PLAN_FISH.bit]);
        assert!(matches!(terrain_resistance("abyss_pool", &tags), TerrainCost::Free));
    }

    #[test]
    fn fish_on_grassland_lethal() {
        let tags = make_tags(&[tag::PLAN_FISH.bit]);
        assert!(matches!(terrain_resistance("grassland", &tags), TerrainCost::Lethal));
    }

    #[test]
    fn deer_on_grassland_free() {
        // Deer: quadruped, no aquatic/swim tags
        let tags = make_tags(&[tag::PLAN_QUADRUPED.bit]);
        assert!(matches!(terrain_resistance("grassland", &tags), TerrainCost::Free));
    }

    #[test]
    fn deer_in_deep_water_lethal() {
        let tags = make_tags(&[tag::PLAN_QUADRUPED.bit]);
        assert!(matches!(terrain_resistance("abyss_pool", &tags), TerrainCost::Lethal));
    }

    #[test]
    fn large_animal_in_shallow_water_slow() {
        let tags = make_tags(&[tag::PLAN_QUADRUPED.bit, tag::SIZE_LARGE.bit]);
        assert!(matches!(terrain_resistance("shallow_water", &tags), TerrainCost::Slow));
    }

    #[test]
    fn small_animal_in_shallow_water_hard() {
        // No SIZE_LARGE or SIZE_HUGE, no aquatic/swim
        let tags = make_tags(&[tag::PLAN_QUADRUPED.bit]);
        assert!(matches!(terrain_resistance("shallow_water", &tags), TerrainCost::Hard));
    }

    #[test]
    fn move_cost_ticks_values() {
        assert_eq!(move_cost_ticks(&TerrainCost::Free), 1);
        assert_eq!(move_cost_ticks(&TerrainCost::Slow), 2);
        assert_eq!(move_cost_ticks(&TerrainCost::Hard), 4);
        assert_eq!(move_cost_ticks(&TerrainCost::Lethal), 1);
    }

    #[test]
    fn lethal_damage_is_two() {
        assert_eq!(lethal_terrain_damage(), 2);
    }
}
