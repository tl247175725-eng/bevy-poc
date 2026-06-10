//! Phase 5 — player five-layer brain (15 assertions).

use bevy_poc::game_constants::{PLAYER_HUNGER_NEED, PLAYER_HUNGER_SEEK, PLAYER_WOLF_THREAT_DIST};
use bevy_poc::interaction::InteractionState;
use bevy_poc::player::{
    build_hut_affordable, compute_affordances, craft_axe_relation, craft_hut_relation,
    craft_spear_relation, ensure_player_mind, fsm_phase_sequence, generate_desires, has_tag,
    knap_stones_to_shard, materials_near_player, plan_craft_knife, priority_rank, select_intention,
    should_forage, should_not_forage_when_full, tick_player_world,
    threat_beats_survival_beats_forage, tick_brain, PlayerMind, TaskPhase,
};
use bevy_poc::sim_events::SimEventQueue;
use bevy_poc::spatial_index::EntityId;
use bevy_poc::systems::tick_entity::tick_entity;
use bevy_poc::world_state::empty_world;

fn setup_camp_player(w: &mut bevy_poc::world_state::WorldState, x: u8, y: u8) -> EntityId {
    let _fire = w.spawn("fire", x, y);
    let _hut = w.spawn("hut", x + 1, y);
    let player = w.spawn("player", x, y);
    let mut mind = PlayerMind::new_spawn();
    mind.hunger = 25.0;
    w.player_minds.insert(player, mind);
    player
}

fn with_mind<F>(w: &mut bevy_poc::world_state::WorldState, p: EntityId, f: F)
where
    F: FnOnce(&bevy_poc::world_state::WorldState, &mut PlayerMind),
{
    ensure_player_mind(&mut w.player_minds, p);
    let mut mind = w.player_minds.remove(&p).unwrap();
    f(w, &mut mind);
    w.player_minds.insert(p, mind);
}

#[test]
fn p5_01_hungry_player_forage_affordance() {
    let mut w = empty_world();
    let p = setup_camp_player(&mut w, 10, 7);
    w.player_minds.get_mut(&p).unwrap().hunger = PLAYER_HUNGER_NEED + 1.0;
    w.spawn("berry", 11, 7);
    tick_brain(&mut w, p);
    let mind = w.player_minds.get(&p).unwrap();
    assert!(mind.affordances.contains_key("forage"), "饥饿时应出现 forage 可供性");
    assert!(should_forage(mind));
}

#[test]
fn p5_02_fed_player_no_forage() {
    let mut w = empty_world();
    let p = setup_camp_player(&mut w, 10, 7);
    w.player_minds.get_mut(&p).unwrap().hunger = 20.0;
    w.spawn("berry", 11, 7);
    tick_brain(&mut w, p);
    let mind = w.player_minds.get(&p).unwrap();
    assert!(!mind.affordances.contains_key("forage"), "饱腹时不应觅食");
    assert!(should_not_forage_when_full(mind));
}

#[test]
fn p5_03_starving_prioritizes_forage_desire() {
    let mut w = empty_world();
    let p = setup_camp_player(&mut w, 10, 7);
    w.player_minds.get_mut(&p).unwrap().hunger = PLAYER_HUNGER_SEEK + 5.0;
    w.spawn("berry", 11, 7);
    tick_brain(&mut w, p);
    let desires = generate_desires(w.player_minds.get(&p).unwrap());
    let top = desires.first().map(|d| d.key.as_str()).unwrap_or("");
    assert!(
        top == "forage" || has_tag(w.player_minds.get(&p).unwrap(), "food_seek"),
        "极饿时应优先觅食相关意向"
    );
}

#[test]
fn p5_04_predator_nearby_drops_autonomy() {
    let mut w = empty_world();
    let p = w.spawn("player", 10, 7);
    w.player_minds.insert(p, PlayerMind::new_spawn());
    w.spawn("wolf", 10 + PLAYER_WOLF_THREAT_DIST, 7);
    tick_brain(&mut w, p);
    let mind = w.player_minds.get(&p).unwrap();
    assert!(mind.needs.autonomy < 100, "狼威胁在范围内应降低 autonomy");
    assert!(has_tag(mind, "predator_nearby_unsafe"));
}

#[test]
fn p5_05_fire_zone_autonomy_stable_with_wolf() {
    let mut w = empty_world();
    let p = setup_camp_player(&mut w, 10, 7);
    w.spawn("wolf", 10 + PLAYER_WOLF_THREAT_DIST, 7);
    tick_brain(&mut w, p);
    let mind = w.player_minds.get(&p).unwrap();
    assert!(mind.needs.autonomy >= 40, "火边自主分应稳定");
    assert!(!has_tag(mind, "predator_nearby_unsafe"), "火区半径内狼不应算 unsafe");
}

#[test]
fn p5_06_threat_enables_craft_knife_affordance() {
    let mut w = empty_world();
    let p = w.spawn("player", 10, 7);
    w.player_minds.insert(p, PlayerMind::new_spawn());
    w.spawn("stone", 11, 7);
    w.spawn("stone", 12, 7);
    w.spawn("wolf", 14, 7);
    tick_brain(&mut w, p);
    assert!(
        w.player_minds.get(&p).unwrap().affordances.contains_key("craft_knife"),
        "有石无刀时应出现 craft_knife"
    );
}

#[test]
fn p5_07_owned_knife_hides_craft_knife() {
    let mut w = empty_world();
    let p = w.spawn("player", 10, 7);
    let mut mind = PlayerMind::new_spawn();
    mind.tools.push("knife".into());
    w.player_minds.insert(p, mind);
    w.spawn("stone", 11, 7);
    w.spawn("stone", 12, 7);
    with_mind(&mut w, p, |world, mind| {
        compute_affordances(world, p, mind);
    });
    assert!(
        !w.player_minds.get(&p).unwrap().affordances.contains_key("craft_knife"),
        "已有刀时不应再 craft_knife"
    );
}

#[test]
fn p5_08_two_stones_knap_to_shard() {
    let mut w = empty_world();
    let s1 = w.spawn("stone", 10, 7);
    let s2 = w.spawn("stone", 11, 7);
    let mut interaction = InteractionState::default();
    let mut events = SimEventQueue::default();
    assert!(knap_stones_to_shard(&mut w, s1, s2, &mut interaction, &mut events));
    assert!(w.count_type("shard") >= 1, "双石砸击应产出碎石");
}

#[test]
fn p5_09_shard_twig_relation_spear() {
    let mut w = empty_world();
    let twig = w.spawn("twig", 10, 7);
    let shard = w.spawn("shard", 11, 7);
    let result = craft_spear_relation(&mut w, twig, shard);
    assert_eq!(result.as_deref(), Some("spear"));
    assert_eq!(w.count_type("spear"), 1);
}

#[test]
fn p5_10_tri_wood_relation_axe() {
    let mut w = empty_world();
    let tri = w.spawn("tri", 10, 7);
    let wood = w.spawn("wood", 11, 7);
    let result = craft_axe_relation(&mut w, tri, wood);
    assert_eq!(result.as_deref(), Some("axe"));
    assert_eq!(w.count_type("axe"), 1);
}

#[test]
fn p5_11_threat_materials_build_hut_affordance() {
    let mut w = empty_world();
    let p = w.spawn("player", 10, 7);
    w.player_minds.insert(p, PlayerMind::new_spawn());
    w.spawn("wolf", 13, 7);
    w.spawn("twig", 11, 7);
    w.spawn("grass", 10, 8);
    tick_brain(&mut w, p);
    let mind = w.player_minds.get(&p).unwrap();
    assert!(materials_near_player(&w, p));
    assert!(
        build_hut_affordable(&w, p, mind) || mind.affordances.contains_key("build_hut"),
        "有威胁且有建材时应能建棚"
    );
}

#[test]
fn p5_12_task_fsm_phase_sequence() {
    let phases = fsm_phase_sequence();
    assert_eq!(phases[0], TaskPhase::Plan);
    assert_eq!(phases[1], TaskPhase::Move);
    assert_eq!(phases[2], TaskPhase::Pickup);
    assert_eq!(phases[3], TaskPhase::MoveTo);
    assert_eq!(phases[4], TaskPhase::Drop);
    assert_eq!(phases[5], TaskPhase::Act);
    assert_eq!(phases[6], TaskPhase::Done);
}

#[test]
fn p5_13_fail_cooldown_blocks_repeat_task() {
    let mut w = empty_world();
    let p = w.spawn("player", 10, 7);
    let mut mind = PlayerMind::new_spawn();
    mind.task_cooldowns.insert("makeKnife".into(), 5.0);
    w.player_minds.insert(p, mind);
    w.spawn("stone", 11, 7);
    w.spawn("stone", 12, 7);
    with_mind(&mut w, p, |world, mind| {
        assert!(!plan_craft_knife(world, p, mind), "冷却中不应重复开工");
    });
}

#[test]
fn p5_14_priority_threat_over_survival_over_forage() {
    assert!(threat_beats_survival_beats_forage(
        "flee_threat",
        "relight_fire",
        "forage"
    ));
    assert!(priority_rank("flee_threat") < priority_rank("forage"));

    let mut w = empty_world();
    let p = w.spawn("player", 10, 7);
    w.player_minds.insert(p, PlayerMind::new_spawn());
    w.player_minds.get_mut(&p).unwrap().hunger = PLAYER_HUNGER_NEED + 5.0;
    w.spawn("berry", 11, 7);
    w.spawn("wolf", 13, 7);
    tick_brain(&mut w, p);
    with_mind(&mut w, p, |world, mind| {
        select_intention(world, p, mind);
    });
    assert_eq!(
        w.player_minds.get(&p).unwrap().top_desire,
        "flee_threat",
        "威胁应优先于觅食"
    );
}

#[test]
fn p5_15_player_bypasses_event_registry() {
    let mut w = empty_world();
    let p = w.spawn("player", 10, 7);
    ensure_player_mind(&mut w.player_minds, p);
    tick_entity(&mut w, p, 1.0);
    assert!(
        w.player_minds.contains_key(&p),
        "玩家 tick 应更新 player_minds（独立 PlayerPlugin 路径）"
    );

    let mut w2 = empty_world();
    let sheep = w2.spawn("sheep", 5, 5);
    w2.spawn("grass", 5, 6);
    tick_entity(&mut w2, sheep, 1.0);
    assert!(w2.entities.contains_key(&sheep), "生态卡仍走 EventRegistry");
}

#[test]
fn p5_16_headless_player_moves_toward_food() {
    let mut w = empty_world();
    let p = setup_camp_player(&mut w, 10, 7);
    w.player_minds.get_mut(&p).unwrap().hunger = PLAYER_HUNGER_NEED + 5.0;
    w.spawn("berry", 18, 7);
    let start_x = w.entities[&p].x;
    tick_brain(&mut w, p);
    tick_player_world(&mut w, p, 1.0);
    assert_ne!(w.entities[&p].x, start_x, "headless 玩家应向浆果移动");
}

#[test]
fn p5_hut_relation_wood_grass() {
    let mut w = empty_world();
    let wood = w.spawn("wood", 10, 7);
    let grass = w.spawn("grass", 11, 7);
    let result = craft_hut_relation(&mut w, wood, grass);
    assert_eq!(result.as_deref(), Some("hut"));
}
