//! Tool craft tasks — Godot `tool_tasks.gd` (makeKnife / makeSpear / makeAxe).

use crate::interaction::{try_impact, try_relation, InteractionState};
use crate::sim_events::SimEventQueue;
use crate::spatial_index::EntityId;
use crate::systems::movement::move_toward;
use crate::world_state::WorldState;

use super::brain_world::{is_neighbor, sorted_by_distance};
use super::state::{PlayerMind, PlayerTask, TaskPhase};
use super::state::{set_task_cooldown, task_on_cooldown};

const TASK_FAIL_COOLDOWN: f32 = 8.0;

pub fn plan_craft_knife(world: &WorldState, player_id: EntityId, mind: &mut PlayerMind) -> bool {
    if task_on_cooldown(mind, "makeKnife") {
        return false;
    }
    let stones = sorted_by_distance(world, world.entities[&player_id].x, world.entities[&player_id].y, "stone");
    if stones.len() < 2 {
        return false;
    }
    mind.task = Some(PlayerTask {
        task_type: "makeKnife".into(),
        phase: TaskPhase::Plan,
        stone1: Some(stones[0].0),
        stone2: Some(stones[1].0),
        ..Default::default()
    });
    mind.goal_text = "制作小刀".into();
    mind.state_label = "制作工具".into();
    true
}

pub fn advance_knife_task(
    world: &mut WorldState,
    player_id: EntityId,
    mind: &mut PlayerMind,
    interaction: &mut InteractionState,
    events: &mut SimEventQueue,
) -> bool {
    let Some(mut task) = mind.task.clone() else {
        return false;
    };
    if task.task_type != "makeKnife" {
        return false;
    }

    let (px, py) = {
        let e = &world.entities[&player_id];
        (e.x, e.y)
    };

    let stone1 = task.stone1.map(EntityId);
    let stone2 = task.stone2.map(EntityId);

    match task.phase {
        TaskPhase::Plan => {
            task.phase = TaskPhase::Move;
            mind.task = Some(task);
            return true;
        }
        TaskPhase::Move => {
            if let Some(s1) = stone1 {
                if let Some(e) = world.entities.get(&s1) {
                    if is_neighbor(px, py, e.x, e.y) {
                        task.phase = TaskPhase::Pickup;
                    } else {
                        move_toward(world, player_id, px, py, e.x, e.y);
                    }
                }
            }
            mind.task = Some(task);
            return true;
        }
        TaskPhase::Pickup => {
            if let Some(s1) = stone1 {
                if let Some(e) = world.entities.get_mut(&player_id) {
                    e.carrying = Some(s1);
                }
                task.phase = TaskPhase::MoveTo;
            }
            mind.task = Some(task);
            return true;
        }
        TaskPhase::MoveTo => {
            if let Some(s2) = stone2 {
                if let Some(e) = world.entities.get(&s2) {
                    if is_neighbor(px, py, e.x, e.y) {
                        task.phase = TaskPhase::Drop;
                    } else {
                        move_toward(world, player_id, px, py, e.x, e.y);
                    }
                }
            }
            mind.task = Some(task);
            return true;
        }
        TaskPhase::Drop => {
            if let Some(e) = world.entities.get_mut(&player_id) {
                e.carrying = None;
            }
            task.phase = TaskPhase::Act;
            mind.task = Some(task);
            return true;
        }
        TaskPhase::Act => {
            if let (Some(s1), Some(s2)) = (stone1, stone2) {
                if world.entities.contains_key(&s1) && world.entities.contains_key(&s2) {
                    let _ = try_impact(world, s1, s2, interaction, events);
                    let _ = try_impact(world, s1, s2, interaction, events);
                }
            }
            task.phase = TaskPhase::Done;
            mind.task = Some(task);
            return true;
        }
        TaskPhase::Done => {
            mind.tools.push("knife".into());
            mind.task = None;
            mind.state_label = "工具完成".into();
            return true;
        }
        TaskPhase::Fail => {
            set_task_cooldown(mind, "makeKnife", TASK_FAIL_COOLDOWN);
            mind.task = None;
            return false;
        }
    }
}

pub fn craft_spear_relation(world: &mut WorldState, twig: EntityId, shard: EntityId) -> Option<String> {
    let (tx, ty) = {
        let shard_e = world.entities.get(&shard)?;
        (shard_e.x, shard_e.y)
    };
    let twig_type = world.entities.get(&twig)?.type_name.clone();
    let shard_type = world.entities.get(&shard)?.type_name.clone();
    let book = InteractionState::default().book;
    try_relation(world, &twig_type, &shard_type, tx, ty, &InteractionState { book, hit_counts: Default::default() })
}

pub fn craft_axe_relation(world: &mut WorldState, tri: EntityId, wood: EntityId) -> Option<String> {
    let (tx, ty) = {
        let wood_e = world.entities.get(&wood)?;
        (wood_e.x, wood_e.y)
    };
    let tri_type = world.entities.get(&tri)?.type_name.clone();
    let wood_type = world.entities.get(&wood)?.type_name.clone();
    let book = InteractionState::default().book;
    try_relation(world, &tri_type, &wood_type, tx, ty, &InteractionState { book, hit_counts: Default::default() })
}

pub fn knap_stones_to_shard(
    world: &mut WorldState,
    stone1: EntityId,
    stone2: EntityId,
    interaction: &mut InteractionState,
    events: &mut SimEventQueue,
) -> bool {
    try_impact(world, stone1, stone2, interaction, events)
        && try_impact(world, stone1, stone2, interaction, events)
}

pub fn fsm_phase_sequence() -> &'static [TaskPhase] {
    &[
        TaskPhase::Plan,
        TaskPhase::Move,
        TaskPhase::Pickup,
        TaskPhase::MoveTo,
        TaskPhase::Drop,
        TaskPhase::Act,
        TaskPhase::Done,
    ]
}
