//! 元动作执行器 — 每个 MetaAction 的执行函数
//!
//! 调用公理验证，操作 WorldState，返回 ActionResult。

use crate::meta_actions::{ActionResult, MetaAction};
use crate::spatial_index::EntityId;
use crate::world_state::WorldState;

pub fn execute(world: &mut WorldState, action: &MetaAction, actor_id: EntityId) -> ActionResult {
    match action {
        MetaAction::Move { dx, dy } => exec_move(world, actor_id, *dx, *dy),
        MetaAction::Strike { target } => exec_strike(world, actor_id, *target),
        MetaAction::Consume { target } => exec_consume(world, actor_id, *target),
        MetaAction::Combine { .. } => ActionResult::Invalid,
        MetaAction::Release { .. } => ActionResult::Invalid,
        MetaAction::Wait { .. } => ActionResult::Invalid,
        MetaAction::Hide { .. } => ActionResult::Invalid,
        MetaAction::Emerge => ActionResult::Invalid,
    }
}

fn exec_move(world: &mut WorldState, id: EntityId, dx: i16, dy: i16) -> ActionResult {
    let (x, y) = match world.entities.get(&id) {
        Some(e) => (e.x, e.y),
        None => return ActionResult::Invalid,
    };
    let nx = (x as i16 + dx).clamp(0, 23) as u8;
    let ny = (y as i16 + dy).clamp(0, 35) as u8;
    match world.move_entity(id, nx, ny) {
        crate::world_state::MoveResult::Moved => ActionResult::Success,
        crate::world_state::MoveResult::Blocked => ActionResult::Blocked {
            reason: format!("compose or traverse blocked ({x},{y})→({nx},{ny})"),
        },
        crate::world_state::MoveResult::NoOp => ActionResult::Invalid,
    }
}

fn exec_strike(world: &mut WorldState, source: EntityId, target: EntityId) -> ActionResult {
    use crate::interaction::smash::damage_entity_hp;
    if source == target {
        return ActionResult::Invalid;
    }
    if damage_entity_hp(world, target, 1) {
        ActionResult::Killed { corpse_spawned: true }
    } else {
        ActionResult::Success
    }
}

fn exec_consume(world: &mut WorldState, eater: EntityId, target: EntityId) -> ActionResult {
    use crate::axioms::TransformAction;
    let (src_profile, eater_profile) = {
        let t = world.entities.get(&target);
        let e = world.entities.get(&eater);
        match (t, e) {
            (Some(t), Some(e)) => (t.profile.clone(), e.profile.clone()),
            _ => return ActionResult::Invalid,
        }
    };
    let result = crate::axioms::AxiomEngine::transform(
        &src_profile, &eater_profile, TransformAction::Eat,
    );
    world.remove_entity(target);
    if let Some(e) = world.entities.get_mut(&eater) {
        e.profile.energy = e.profile.energy.saturating_add(result.energy_received);
    }
    ActionResult::Consumed { energy_gained: result.energy_received }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world_state::empty_world;

    #[test]
    fn move_execute_east() {
        let mut w = empty_world();
        let id = w.spawn("sheep", 5, 5);
        assert_eq!(exec_move(&mut w, id, 1, 0), ActionResult::Success);
        assert_eq!(w.entities[&id].x, 6);
    }

    #[test]
    fn strike_damages() {
        let mut w = empty_world();
        let wolf = w.spawn("wolf", 5, 5);
        let sheep = w.spawn("sheep", 6, 5);
        let hp = w.entities[&sheep].hp;
        let r = exec_strike(&mut w, wolf, sheep);
        assert!(matches!(r, ActionResult::Success | ActionResult::Killed { .. }));
        assert!(w.entities[&sheep].hp < hp || !w.entities.contains_key(&sheep));
    }

    #[test]
    fn consume_removes_target() {
        let mut w = empty_world();
        let sheep = w.spawn("sheep", 5, 5);
        let grass = w.spawn("grass", 5, 5);
        // 移开 grass 上的 sheep，把 sheep 放旁边
        w.entities.get_mut(&sheep).unwrap().x = 5;
        w.entities.get_mut(&sheep).unwrap().y = 6;
        let r = exec_consume(&mut w, sheep, grass);
        assert!(matches!(r, ActionResult::Consumed { .. }));
        assert!(!w.entities.contains_key(&grass), "grass should be consumed");
    }
}
