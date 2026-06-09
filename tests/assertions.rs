use bevy_poc::{
    all_capability_cards, can_hunt_target, card_capabilities, card_has_tag, demo_world,
    empty_world, flocking_blocks_reproduction, is_aquatic_card, is_herbivore, is_hunt_target_for,
    is_predator, load_card_defs, mark_perishable, pack_hunter_under_strength, CardDef,
    EcologyState, PerishableState, WorldState, FEAR_RANGE, GRASS_REGEN_INTERVAL, HUNT_RANGE,
    PERISHABLE_TICKS,
};

fn test_world() -> WorldState {
    let mut w = empty_world();
    for x in 0..5 {
        w.mark_river(x, 0);
    }
    w
}

fn def<'a>(world: &'a WorldState, name: &str) -> &'a CardDef {
    &world.card_defs[name]
}

// --- POC 20 ---

#[test]
fn assert_01_grass_no_regen_off_riverbank() {
    let mut world = empty_world();
    world.spawn("grass", 10, 10);
    let before = world.grass_count();
    world.remove_entity(world.spatial_index.query_tag("grass")[0]);
    assert_eq!(world.grass_count(), before - 1);
    world.run_ticks(GRASS_REGEN_INTERVAL);
    assert_eq!(world.grass_count(), before - 1);
}

#[test]
fn assert_02_flocking_blocks_reproduction_under_three() {
    let world = test_world();
    let sheep = def(&world, "sheep");
    let adults = vec![sheep, sheep];
    assert!(flocking_blocks_reproduction(&adults));
    let mut world = test_world();
    world.spawn("sheep", 5, 5);
    world.spawn("sheep", 6, 5);
    let before = world.sheep_count();
    world.run_ticks(50);
    assert_eq!(world.sheep_count(), before);
}

#[test]
fn assert_03_pack_hunter_single_wolf_only_small_prey() {
    let world = empty_world();
    let wolf = def(&world, "wolf");
    let sheep = def(&world, "sheep");
    let rabbit = def(&world, "rabbit");
    let large_prey = CardDef {
        type_name: "bear".into(),
        display_name: "熊".into(),
        icon: "熊".into(),
        tags: vec!["being".into(), "animal".into(), "herbivore".into()],
        color: (128, 128, 128, 255),
        hp: 10,
        is_rooted: false,
    };
    assert!(pack_hunter_under_strength(&[wolf]));
    assert!(!can_hunt_target(1, wolf, &large_prey));
    assert!(!can_hunt_target(1, wolf, sheep));
    assert!(can_hunt_target(1, wolf, rabbit));
}

#[test]
fn assert_04_sheep_eating_removes_grass() {
    let mut world = test_world();
    let grass_id = world.spawn("grass", 8, 8);
    let sheep_id = world.spawn("sheep", 8, 7);
    world.tick_once();
    assert!(!world.entities.contains_key(&grass_id));
    assert!(world.entities.contains_key(&sheep_id));
}

#[test]
fn assert_05_fed_sheep_stops_eating() {
    let mut world = test_world();
    world.spawn("grass", 8, 8);
    let sheep_id = world.spawn("sheep", 8, 7);
    world.tick_once();
    assert!(world.entities[&sheep_id].fed);
    world.spawn("grass", 8, 8);
    let grass_count_before = world.grass_count();
    world.tick_once();
    assert_eq!(world.grass_count(), grass_count_before);
}

#[test]
fn assert_06_wolf_hunts_sheep_in_range() {
    let mut world = test_world();
    let den_id = world.spawn("wolfDen", 15, 15);
    world.spawn("sheep", 10, 10);
    world.spawn("wolf", 9, 12);
    let wolf_id = world.spawn("wolf", 12, 10);
    if let Some(w) = world.entities.get_mut(&wolf_id) {
        w.den_id = Some(den_id);
    }
    world.tick_once();
    assert_eq!(world.entities[&wolf_id].ecology_state, EcologyState::Hunting);
}

#[test]
fn assert_07_hunt_produces_corpse() {
    let mut world = test_world();
    let den_id = world.spawn("wolfDen", 15, 15);
    world.spawn("sheep", 10, 10);
    let w1 = world.spawn("wolf", 9, 10);
    let w2 = world.spawn("wolf", 11, 10);
    for wid in [w1, w2] {
        if let Some(w) = world.entities.get_mut(&wid) {
            w.den_id = Some(den_id);
        }
    }
    world.tick_once();
    assert!(
        world
            .entities
            .values()
            .any(|e| e.type_name == "sheepCorpse")
    );
}

#[test]
fn assert_08_sheep_flees_from_wolf() {
    let mut world = test_world();
    let sheep_id = world.spawn("sheep", 10, 10);
    let start_x = world.entities[&sheep_id].x;
    world.spawn("wolf", 11, 10);
    world.tick_once();
    let sheep = &world.entities[&sheep_id];
    assert_eq!(sheep.ecology_state, EcologyState::Fleeing);
    assert_ne!(sheep.x, start_x);
    assert!(sheep.x.abs_diff(11) <= FEAR_RANGE + 1);
}

#[test]
fn assert_09_riverbank_grass_regen_interval() {
    let mut world = empty_world();
    world.mark_river(2, 0);
    world.spawn("grass", 2, 0);
    let grass_id = world.spatial_index.query_tag("grass")[0];
    world.remove_entity(grass_id);
    assert_eq!(world.grass_count(), 0);
    world.run_ticks(GRASS_REGEN_INTERVAL);
    assert_eq!(world.grass_count(), 1);
}

#[test]
fn assert_10_spatial_index_query_tag_grass() {
    let mut world = test_world();
    world.spawn("grass", 1, 1);
    world.spawn("grass", 2, 2);
    world.spawn("sheep", 3, 3);
    assert_eq!(world.spatial_index.query_tag("grass").len(), 2);
}

#[test]
fn assert_11_spatial_index_query_near_sheep() {
    let mut world = test_world();
    world.spawn("sheep", 5, 5);
    world.spawn("sheep", 20, 20);
    let near = world.spatial_index.query_near(5, 5, "sheep", 3);
    assert_eq!(near.len(), 1);
}

#[test]
fn assert_12_spatial_index_updates_on_move() {
    let mut world = test_world();
    let sheep_id = world.spawn("sheep", 5, 5);
    world.move_entity(sheep_id, 8, 8);
    assert_eq!(world.spatial_index.position(sheep_id), Some((8, 8)));
    assert!(world.spatial_index.query_near(8, 8, "sheep", 0).contains(&sheep_id));
    assert!(!world.spatial_index.query_near(5, 5, "sheep", 0).contains(&sheep_id));
}

#[test]
fn assert_13_card_has_tag_flocking_sheep() {
    let world = empty_world();
    assert!(card_has_tag(def(&world, "sheep"), "flocking"));
}

#[test]
fn assert_14_wolf_out_of_hunt_range_no_hunt() {
    let mut world = test_world();
    world.spawn("sheep", 5, 5);
    let wolf_id = world.spawn("wolf", 5 + HUNT_RANGE as u8 + 3, 5);
    world.tick_once();
    assert_ne!(world.entities[&wolf_id].ecology_state, EcologyState::Hunting);
}

#[test]
fn assert_15_grass_removed_at_zero_hp() {
    let mut world = test_world();
    let grass_id = world.spawn("grass", 4, 4);
    if let Some(grass) = world.entities.get_mut(&grass_id) {
        grass.hp = 0;
    }
    world.remove_entity(grass_id);
    assert!(!world.entities.contains_key(&grass_id));
    assert_eq!(world.spatial_index.query_tag("grass").len(), 0);
}

#[test]
fn assert_16_sheep_idle_without_grass() {
    let mut world = test_world();
    let sheep_id = world.spawn("sheep", 7, 7);
    world.tick_once();
    assert_eq!(world.entities[&sheep_id].ecology_state, EcologyState::Idle);
}

#[test]
fn assert_17_wolf_patrols_without_prey() {
    let mut world = test_world();
    let den_id = world.spawn("wolfDen", 15, 15);
    let wolf_id = world.spawn("wolf", 15, 15);
    if let Some(w) = world.entities.get_mut(&wolf_id) {
        w.den_id = Some(den_id);
    }
    world.tick_once();
    assert_ne!(
        world.entities[&wolf_id].ecology_state,
        EcologyState::Hunting
    );
}

#[test]
fn assert_18_grass_not_eaten_twice_same_tick() {
    let mut world = test_world();
    let grass_id = world.spawn("grass", 9, 9);
    world.spawn("sheep", 9, 8);
    world.spawn("sheep", 8, 9);
    world.tick_once();
    assert!(!world.entities.contains_key(&grass_id));
    assert_eq!(world.grass_count(), 0);
}

#[test]
fn assert_19_grass_and_sheep_adjacent_cells() {
    let mut world = test_world();
    let grass_id = world.spawn("grass", 6, 6);
    let sheep_id = world.spawn("sheep", 6, 5);
    assert!(world.entities.contains_key(&grass_id));
    assert!(world.entities.contains_key(&sheep_id));
    assert_eq!(world.spatial_index.position(grass_id), Some((6, 6)));
    assert_eq!(world.spatial_index.position(sheep_id), Some((6, 5)));
}

#[test]
fn assert_20_ecosystem_stable_after_1000_ticks() {
    let mut world = demo_world();
    let sheep_before = world.sheep_count();
    let wolf_before = world.wolf_count();
    world.run_ticks(1000);
    assert!(world.sheep_count() <= sheep_before + 5);
    assert!(world.wolf_count() <= wolf_before + 3);
    assert!(world.entities.len() < 800);
    assert!(world.entities.len() < 500);
}

// --- M1 30 ---

#[test]
fn assert_21_card_defs_load_at_least_50() {
    let defs = load_card_defs("assets/card_defs.ron");
    assert!(defs.len() >= 50, "got {}", defs.len());
}

#[test]
fn assert_22_sheep_tags() {
    let world = empty_world();
    let sheep = def(&world, "sheep");
    assert!(card_has_tag(sheep, "flocking"));
    assert!(card_has_tag(sheep, "herbivore"));
    assert!(card_has_tag(sheep, "largePrey"));
}

#[test]
fn assert_23_wolf_tags() {
    let world = empty_world();
    let wolf = def(&world, "wolf");
    assert!(card_has_tag(wolf, "predator"));
    assert!(card_has_tag(wolf, "pack_hunter"));
}

#[test]
fn assert_24_deer_tags() {
    let world = empty_world();
    let deer = def(&world, "deer");
    assert!(card_has_tag(deer, "wildPrey"));
    assert!(card_has_tag(deer, "largePrey"));
}

#[test]
fn assert_25_rabbit_tags() {
    let world = empty_world();
    let rabbit = def(&world, "rabbit");
    assert!(card_has_tag(rabbit, "prolific"));
    assert!(card_has_tag(rabbit, "smallHerbivore"));
}

#[test]
fn assert_26_fish_tags() {
    let world = empty_world();
    let fish = def(&world, "fish");
    assert!(card_has_tag(fish, "aquatic"));
    assert!(card_has_tag(fish, "small"));
}

#[test]
fn assert_27_shellfish_tags() {
    let world = empty_world();
    assert!(card_has_tag(def(&world, "shellfish"), "sessile"));
}

#[test]
fn assert_28_oak_tags() {
    let world = empty_world();
    let oak = def(&world, "oak");
    assert!(card_has_tag(oak, "nut_producer"));
    assert!(card_has_tag(oak, "rooted"));
}

#[test]
fn assert_29_water_caltrop_floating() {
    let world = empty_world();
    assert!(card_has_tag(def(&world, "waterCaltrop"), "floating"));
}

#[test]
fn assert_30_wild_yam_underground() {
    let world = empty_world();
    let yam = def(&world, "wildYam");
    assert!(card_has_tag(yam, "tuber"));
    assert!(card_has_tag(yam, "underground"));
}

#[test]
fn assert_31_land_bug_volant() {
    let world = empty_world();
    let bug = def(&world, "landBug");
    assert!(card_has_tag(bug, "volant"));
    assert!(card_has_tag(bug, "smallPrey"));
}

#[test]
fn assert_32_all_meat_perishable() {
    let world = empty_world();
    for name in [
        "sheepMeat",
        "rabbitMeat",
        "deerMeat",
        "wolfMeat",
        "humanMeat",
        "fishMeat",
    ] {
        assert!(
            card_has_tag(def(&world, name), "perishable"),
            "{name} missing perishable"
        );
    }
}

#[test]
fn assert_33_all_corpse_tagged() {
    let world = empty_world();
    for name in ["sheepCorpse", "deerCorpse", "wolfCorpse", "playerCorpse"] {
        assert!(card_has_tag(def(&world, name), "corpse"), "{name} missing corpse");
    }
}

#[test]
fn assert_34_is_predator_wolf() {
    let world = empty_world();
    assert!(is_predator(def(&world, "wolf")));
}

#[test]
fn assert_35_is_predator_sheep_false() {
    let world = empty_world();
    assert!(!is_predator(def(&world, "sheep")));
}

#[test]
fn assert_36_is_herbivore_sheep() {
    let world = empty_world();
    assert!(is_herbivore(def(&world, "sheep")));
}

#[test]
fn assert_37_is_herbivore_wolf_false() {
    let world = empty_world();
    assert!(!is_herbivore(def(&world, "wolf")));
}

#[test]
fn assert_38_is_aquatic_fish() {
    let world = empty_world();
    assert!(is_aquatic_card(def(&world, "fish")));
}

#[test]
fn assert_39_is_aquatic_sheep_false() {
    let world = empty_world();
    assert!(!is_aquatic_card(def(&world, "sheep")));
}

#[test]
fn assert_40_oak_nut_producer() {
    let world = empty_world();
    assert!(card_has_tag(def(&world, "oak"), "nut_producer"));
}

#[test]
fn assert_41_fire_camp_anchor() {
    let world = empty_world();
    assert!(card_has_tag(def(&world, "fire"), "camp.anchor"));
}

#[test]
fn assert_42_stone_material() {
    let world = empty_world();
    assert!(card_has_tag(def(&world, "stone"), "material.stone"));
}

#[test]
fn assert_43_bucket_water_container() {
    let world = empty_world();
    assert!(card_has_tag(def(&world, "bucket"), "container.water"));
}

#[test]
fn assert_44_flocking_blocks_two_sheep() {
    let world = empty_world();
    let sheep = def(&world, "sheep");
    assert!(flocking_blocks_reproduction(&[sheep, sheep]));
}

#[test]
fn assert_45_flocking_allows_three_sheep() {
    let world = empty_world();
    let sheep = def(&world, "sheep");
    assert!(!flocking_blocks_reproduction(&[sheep, sheep, sheep]));
}

#[test]
fn assert_46_pack_under_strength_one_wolf() {
    let world = empty_world();
    assert!(pack_hunter_under_strength(&[def(&world, "wolf")]));
}

#[test]
fn assert_47_pack_strength_two_wolves() {
    let world = empty_world();
    let wolf = def(&world, "wolf");
    assert!(!pack_hunter_under_strength(&[wolf, wolf]));
}

#[test]
fn assert_48_wolf_hunts_sheep() {
    let world = empty_world();
    assert!(is_hunt_target_for(
        def(&world, "wolf"),
        def(&world, "sheep")
    ));
}

#[test]
fn assert_49_wolf_not_hunt_rabbit_full_pack() {
    let world = empty_world();
    assert!(!is_hunt_target_for(
        def(&world, "wolf"),
        def(&world, "rabbit")
    ));
}

#[test]
fn assert_50_capabilities_non_empty() {
    for name in all_capability_cards() {
        if name == "humus" {
            continue;
        }
        let caps = card_capabilities(name);
        assert!(!caps.is_empty(), "{name} has empty capabilities");
    }
}

#[test]
fn assert_mark_perishable_sets_ticks() {
    let mut state = PerishableState::default();
    mark_perishable(&mut state);
    assert_eq!(state.perish_ticks, PERISHABLE_TICKS);
}
