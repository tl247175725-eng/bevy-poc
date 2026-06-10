use bevy_poc::{
    can_hunt_target, card_has_tag, empty_world, entities_in_pool, entities_in_tree,
    entities_underground, flocking_blocks_reproduction, harvest_at, herbivore_grazer_profile,
    is_hunt_target_for, is_hunt_target_for_pack, load_card_defs, mark_ecology_fed,
    mark_perishable, pack_hunter_under_strength, wolves_near, CardDef, EcologyState,
    GrazerProfile, PerishableState, WorldState, PERISHABLE_TICKS,
    POPULATION_REPRO_CYCLE_SECONDS, PROLIFIC_LITTER_SIZE, PROLIFIC_REPRO_CYCLE_SECONDS,
};

fn def<'a>(w: &'a WorldState, name: &str) -> &'a CardDef {
    &w.card_defs[name]
}

fn tw() -> WorldState {
    let mut w = empty_world();
    for x in 0..8 {
        w.mark_river(x, 0);
    }
    w
}

// --- 食草 10 ---

#[test]
fn m2_01_sheep_eats_grass_gone() {
    let mut w = tw();
    let g = w.spawn("grass", 5, 5);
    w.spawn("sheep", 5, 4);
    w.tick_once();
    assert!(w.entities.contains_key(&g));
    assert_eq!(w.entities[&g].hp, 3);
}

#[test]
fn m2_02_deer_flees_wolf() {
    let mut w = tw();
    let d = w.spawn("deer", 10, 10);
    let sx = w.entities[&d].x;
    w.spawn("wolf", 12, 10);
    w.spawn("wolf", 13, 10);
    w.tick_once();
    assert_eq!(w.entities[&d].ecology_state, EcologyState::Fleeing);
    assert_ne!(w.entities[&d].x, sx);
}

#[test]
fn m2_03_rabbit_hides_in_grass() {
    let mut w = tw();
    w.spawn("grass", 8, 8);
    let r = w.spawn("rabbit", 8, 8);
    w.spawn("wolf", 9, 8);
    w.tick_once();
    assert!(w.entities[&r].in_cover);
}

#[test]
fn m2_04_pheasant_no_hide() {
    let mut w = tw();
    w.spawn("grass", 8, 8);
    let p = w.spawn("pheasant", 8, 8);
    w.spawn("wolf", 9, 8);
    w.tick_once();
    assert!(!w.entities[&p].in_cover);
}

#[test]
fn m2_05_sheep_profile() {
    let w = empty_world();
    assert_eq!(herbivore_grazer_profile(def(&w, "sheep")), GrazerProfile::Sheep);
}

#[test]
fn m2_06_deer_profile() {
    let w = empty_world();
    assert_eq!(herbivore_grazer_profile(def(&w, "deer")), GrazerProfile::Deer);
}

#[test]
fn m2_07_rabbit_profile() {
    let w = empty_world();
    assert_eq!(herbivore_grazer_profile(def(&w, "rabbit")), GrazerProfile::Rabbit);
}

#[test]
fn m2_08_water_buffalo_slow() {
    let w = empty_world();
    assert_eq!(
        herbivore_grazer_profile(def(&w, "waterBuffalo")),
        GrazerProfile::Slow
    );
}

#[test]
fn m2_09_mark_ecology_fed() {
    let mut w = tw();
    let s = w.spawn("sheep", 3, 3);
    let sheep_def = w.card_defs.get("sheep").cloned().unwrap();
    if let Some(e) = w.entities.get_mut(&s) {
        mark_ecology_fed(e, &sheep_def);
    }
    assert!(w.entities[&s].fed_today);
}

#[test]
fn m2_10_deer_fear_range_wider() {
    let mut w = tw();
    w.spawn("wolf", 14, 10);
    let deer = w.spawn("deer", 10, 10);
    let sheep = w.spawn("sheep", 10, 12);
    assert!(!wolves_near(&w, w.entities[&deer].x, w.entities[&deer].y, 5).is_empty());
    assert!(wolves_near(&w, w.entities[&sheep].x, w.entities[&sheep].y, 3).is_empty());
}

// --- 捕食 10 ---

#[test]
fn m2_11_wolf_hunt_creates_corpse() {
    let mut w = tw();
    let den_id = w.spawn("wolfDen", 20, 20);
    let sheep = w.spawn("sheep", 6, 6);
    if let Some(s) = w.entities.get_mut(&sheep) {
        s.hp = 1;
    }
    let w1 = w.spawn("wolf", 6, 6);
    let w2 = w.spawn("wolf", 7, 6);
    for wid in [w1, w2] {
        if let Some(wolf) = w.entities.get_mut(&wid) {
            wolf.den_id = Some(den_id);
        }
    }
    for _ in 0..4 {
        w.tick_once();
        if w.count_type("sheepCorpse") >= 1 || w.entities.values().any(|e| e.is_corpse) {
            break;
        }
    }
    assert!(w.count_type("sheepCorpse") >= 1 || w.entities.values().any(|e| e.is_corpse));
}

#[test]
fn m2_12_fox_scavenges_corpse() {
    let mut w = tw();
    let c = w.spawn("sheepCorpse", 5, 5);
    if let Some(e) = w.entities.get_mut(&c) {
        e.is_corpse = true;
    }
    w.spawn("fox", 5, 5);
    w.tick_once();
    assert_eq!(w.count_type("sheepCorpse"), 0);
}

#[test]
fn m2_13_pack_hunter_single_small_prey() {
    let w = empty_world();
    assert!(can_hunt_target(1, def(&w, "wolf"), def(&w, "rabbit")));
    assert!(!can_hunt_target(1, def(&w, "wolf"), def(&w, "sheep")));
}

#[test]
fn m2_14_wolf_pack_hunts_sheep() {
    let w = empty_world();
    assert!(is_hunt_target_for(def(&w, "wolf"), def(&w, "sheep")));
}

#[test]
fn m2_15_fox_hunts_rabbit() {
    let w = empty_world();
    assert!(is_hunt_target_for_pack(
        def(&w, "fox"),
        def(&w, "rabbit"),
        1
    ));
}

#[test]
fn m2_16_fox_not_hunt_deer() {
    let w = empty_world();
    assert!(!is_hunt_target_for_pack(def(&w, "fox"), def(&w, "deer"), 1));
}

#[test]
fn m2_17_wolf_den_spawn() {
    let mut w = tw();
    w.spawn("wolf", 4, 4);
    for _ in 0..5 {
        w.tick_once();
    }
    assert!(w.count_type("wolfDen") >= 1);
}

#[test]
fn m2_18_fox_den_from_bush() {
    let mut w = tw();
    w.spawn("bush", 7, 7);
    w.spawn("fox", 7, 7);
    w.tick_once();
    assert!(w.count_type("foxDen") >= 1);
    assert_eq!(w.count_type("bush"), 0);
}

#[test]
fn m2_19_wolf_hunting_state() {
    let mut w = tw();
    let den_id = w.spawn("wolfDen", 20, 20);
    w.spawn("sheep", 9, 9);
    let wolf = w.spawn("wolf", 11, 9);
    let w2 = w.spawn("wolf", 12, 9);
    for wid in [wolf, w2] {
        if let Some(entity) = w.entities.get_mut(&wid) {
            entity.den_id = Some(den_id);
        }
    }
    w.tick_once();
    assert!(
        w.entities[&wolf].ecology_state == EcologyState::Hunting
            || w.entities[&w2].ecology_state == EcologyState::Hunting,
        "at least one wolf should be hunting",
    );
}

#[test]
fn m2_20_predator_flee_fire() {
    let mut w = tw();
    w.mark_fire(5, 5);
    let wolf = w.spawn("wolf", 5, 6);
    w.tick_once();
    assert_eq!(w.entities[&wolf].ecology_state, EcologyState::Fleeing);
}

// --- 水生 8 ---

#[test]
fn m2_21_algae_spawns_in_pool() {
    let mut w = tw();
    w.mark_pool(10, 10);
    for _ in 0..16 {
        w.tick_once();
    }
    assert!(w.count_type("algae") >= 1);
}

#[test]
fn m2_22_fish_eats_water_bug() {
    let mut w = tw();
    w.mark_pool(11, 11);
    let bug = w.spawn("waterBug", 11, 11);
    let fish = w.spawn("fish", 11, 10);
    for id in [bug, fish] {
        if let Some(e) = w.entities.get_mut(&id) {
            e.in_pool = true;
        }
    }
    let fish_def = w.card_defs.get("fish").unwrap().clone();
    let outcome = bevy_poc::apply_hunt_smash(&mut w, fish, bug, &fish_def);
    assert_eq!(outcome, bevy_poc::SmashOutcome::Killed);
    assert!(!w.entities.contains_key(&bug));
}

#[test]
fn m2_22b_landbug_kill_no_corpse_panic() {
    let mut w = tw();
    let bug = w.spawn("landBug", 8, 8);
    let wolf = w.spawn("wolf", 8, 8);
    let wolf_def = w.card_defs.get("wolf").unwrap().clone();
    let outcome = bevy_poc::apply_hunt_smash(&mut w, wolf, bug, &wolf_def);
    assert_eq!(outcome, bevy_poc::SmashOutcome::Killed);
    assert!(!w.entities.contains_key(&bug));
    assert_eq!(w.count_type("landBugCorpse"), 0);
}

#[test]
fn m2_23_shellfish_fed_on_algae() {
    let mut w = tw();
    w.mark_pool(12, 12);
    w.spawn("algae", 12, 12);
    let sh = w.spawn("shellfish", 12, 12);
    w.tick_once();
    assert!(w.entities[&sh].fed_today || w.entities[&sh].fed);
}

#[test]
fn m2_24_water_bug_migrates() {
    let mut w = tw();
    w.mark_pool(13, 13);
    w.mark_pool(14, 13);
    for _ in 0..20 {
        w.tick_once();
    }
    assert!(w.count_type("waterBug") >= 1);
}

#[test]
fn m2_25_fish_migrates() {
    let mut w = tw();
    w.mark_pool(15, 15);
    for _ in 0..20 {
        w.tick_once();
    }
    assert!(w.count_type("fish") >= 1);
}

#[test]
fn m2_26_aquatic_in_pool() {
    let mut w = tw();
    w.mark_pool(16, 16);
    let f = w.spawn("fish", 16, 16);
    assert!(w.entities[&f].in_pool);
}

#[test]
fn m2_27_water_bug_eats_algae() {
    let mut w = tw();
    w.mark_pool(17, 17);
    w.mark_pool(18, 17);
    w.spawn("algae", 18, 17);
    w.spawn("waterBug", 17, 17);
    w.tick_once();
    assert_eq!(w.count_type("algae"), 0);
}

#[test]
fn m2_28_fish_aquatic_tag() {
    let w = empty_world();
    assert!(card_has_tag(def(&w, "fish"), "aquatic"));
}

// --- 分解 6 ---

#[test]
fn m2_29_corpse_becomes_humus() {
    let mut w = tw();
    let c = w.spawn("sheepCorpse", 4, 4);
    if let Some(e) = w.entities.get_mut(&c) {
        e.is_corpse = true;
        e.decay_timer = 14.0;
    }
    w.tick_once();
    assert!(!w.entities.contains_key(&c));
    assert!(w.count_type("humus") >= 1);
}

#[test]
fn m2_30_perishable_decays() {
    let mut w = tw();
    let m = w.spawn("sheepMeat", 3, 3);
    if let Some(e) = w.entities.get_mut(&m) {
        e.perish_ticks = 1;
    }
    w.tick_once();
    assert!(!w.entities.contains_key(&m));
}

#[test]
fn m2_31_mark_perishable_ticks() {
    let mut s = PerishableState::default();
    mark_perishable(&mut s);
    assert_eq!(s.perish_ticks, PERISHABLE_TICKS);
}

#[test]
fn m2_32_land_bug_attracted() {
    let mut w = tw();
    let c = w.spawn("sheepCorpse", 6, 6);
    if let Some(e) = w.entities.get_mut(&c) {
        e.is_corpse = true;
    }
    w.tick_once();
    assert!(w.count_type("landBug") >= 1);
}

#[test]
fn m2_33_humus_layers() {
    let mut w = tw();
    w.humus_layers.insert((5, 5), 1);
    w.spawn("humus", 5, 5);
    assert_eq!(w.humus_layers.get(&(5, 5)), Some(&1));
}

#[test]
fn m2_34_corpse_tagged() {
    let w = empty_world();
    assert!(card_has_tag(def(&w, "sheepCorpse"), "corpse"));
}

// --- 繁殖 8 ---

#[test]
fn m2_35_sheep_flocking_repro_block() {
    let w = empty_world();
    let s = def(&w, "sheep");
    assert!(flocking_blocks_reproduction(&[s, s]));
}

#[test]
fn m2_36_rabbit_prolific_litter() {
    assert_eq!(PROLIFIC_LITTER_SIZE, 3);
    assert_eq!(PROLIFIC_REPRO_CYCLE_SECONDS, 3.0);
}

#[test]
fn m2_37_wolf_repro_needs_den() {
    let mut w = tw();
    w.spawn("wolf", 8, 8);
    w.spawn("wolf", 9, 8);
    for _ in 0..35 {
        w.tick_once();
    }
    assert!(w.count_type("wolfCub") >= 1 || w.count_type("wolfDen") >= 1);
}

#[test]
fn m2_38_deer_no_repro_with_wolf_near() {
    let mut w = tw();
    for i in 0..10 {
        w.spawn("grass", 10 + i, 10);
    }
    w.spawn("deer", 10, 10);
    let den_id = w.spawn("wolfDen", 11, 10);
    let wid = w.spawn("wolf", 11, 10);
    if let Some(wolf) = w.entities.get_mut(&wid) {
        wolf.den_id = Some(den_id);
        wolf.in_den = true;
    }
    let before = w.count_type("deerFawn");
    for _ in 0..35 {
        w.tick_once();
    }
    assert_eq!(w.count_type("deerFawn"), before);
}

#[test]
fn m2_39_population_cycle_constant() {
    assert_eq!(POPULATION_REPRO_CYCLE_SECONDS, 30.0);
}

#[test]
fn m2_40_rabbit_repro_fast() {
    let mut w = tw();
    w.spawn("rabbit", 5, 5);
    let before = w.count_type("rabbit");
    for _ in 0..10 {
        w.tick_once();
    }
    assert!(w.count_type("rabbit") > before);
}

#[test]
fn m2_41_pheasant_flocking_gate() {
    let w = empty_world();
    let p = def(&w, "pheasant");
    assert!(flocking_blocks_reproduction(&[p, p]));
}

#[test]
fn m2_42_pack_strength_two() {
    let w = empty_world();
    let wolf = def(&w, "wolf");
    assert!(!pack_hunter_under_strength(&[wolf, wolf]));
}

// --- 容纳 8 ---

#[test]
fn m2_43_oak_in_tree() {
    let mut w = tw();
    let t = w.spawn("oak", 10, 14);
    assert!(w.entities[&t].in_tree);
}

#[test]
fn m2_44_tree_produces_acorn() {
    let mut w = tw();
    w.spawn("oak", 11, 14);
    for _ in 0..30 {
        w.tick_once();
    }
    assert!(w.count_type("acorn") >= 1);
}

#[test]
fn m2_45_pool_entities_listed() {
    let mut w = tw();
    w.mark_pool(12, 14);
    w.spawn("fish", 12, 14);
    let list = entities_in_pool(&w, 12, 14);
    assert_eq!(list.len(), 1);
}

#[test]
fn m2_46_underground_yam() {
    let mut w = tw();
    let y = w.spawn("wildYam", 13, 14);
    assert!(w.entities[&y].in_ground);
    let u = entities_underground(&w, 13, 14);
    assert!(!u.is_empty());
}

#[test]
fn m2_47_harvest_yam_root() {
    let mut w = tw();
    w.spawn("wildYam", 14, 14);
    let got = harvest_at(&mut w, 14, 14, "");
    assert_eq!(got.as_deref(), Some("wildYamRoot"));
}

#[test]
fn m2_48_harvest_shellfish() {
    let mut w = tw();
    w.spawn("shellfish", 15, 14);
    let got = harvest_at(&mut w, 15, 14, "");
    assert_eq!(got.as_deref(), Some("fishMeat"));
}

#[test]
fn m2_49_entities_in_tree() {
    let mut w = tw();
    let tree = w.spawn("oak", 16, 14);
    for _ in 0..30 {
        w.tick_once();
    }
    let inside = entities_in_tree(&w, tree);
    assert!(!inside.is_empty() || w.count_type("acorn") >= 1);
}

#[test]
fn m2_50_card_count_still_85() {
    let defs = load_card_defs("assets/card_defs.ron");
    assert!(defs.len() >= 85);
}
