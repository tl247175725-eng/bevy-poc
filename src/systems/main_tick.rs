use crate::event_registry::EventRegistry;
use crate::meta_actions::MetaAction;
use crate::need_match::data::NeedKind;
use crate::need_match::search::MaterialProperties;
use crate::perception::{perceive_smell, perceive_vision};
use crate::spatial_index::EntityId;
use crate::systems::batch_uniform::{batch_uniform_entity_updates, flush_corpse_decay};
use crate::systems::tick_reactive::{flush_reactive_tick, mark_baseline_reactive_tick, tick_reactive};
use crate::tags::{TagBits, tag};
use crate::world_state::WorldState;

/// Maximum Chebyshev distance any entity can sense across.
/// Vision/smell channels individually clamp to lower per-entity values;
/// this is the outer bound for spatial-index queries.
const MAX_SENSE_RANGE: u8 = crate::meta_values::MAX_SENSE_RANGE;

pub fn main_tick(world: &mut WorldState, delta: f32) {
    world.tick_delta = delta;
    world.tick_count += 1;
    world.elapsed += delta;

    // ===== Phase 1: Perceive（所有实体统一感知，冻结快照） =====
    // 第一步: 为每个实体收集感知贡献 (不可变遍历)
    let mut perception_buffer: Vec<(EntityId, Vec<(NeedKind, f32)>)> = Vec::new();

    // 收集所有实体 id + 位置快照，避免 borrow 冲突
    let snapshots: Vec<(EntityId, u8, u8)> = world
        .entities
        .values()
        .map(|e| (e.id, e.x, e.y))
        .collect();

    for &(observer_id, ox, oy) in &snapshots {
        let mut contributions: Vec<(NeedKind, f32)> = Vec::new();

        let neighbors = world.spatial_index.query_radius_all(ox, oy, MAX_SENSE_RANGE);
        for neighbor_id in neighbors {
            if neighbor_id == observer_id {
                continue;
            }
            let Some(neighbor) = world.entities.get(&neighbor_id) else {
                continue;
            };
            let distance =
                (ox.abs_diff(neighbor.x) + oy.abs_diff(neighbor.y)) as u32; // Manhattan

            // TagRegistry 就位后使用 card_def 中的实际 tag_bits
            let empty_tags = TagBits::new();
            let observer_tag_bits = world
                .card_defs
                .get(&world.entities[&observer_id].type_name)
                .map(|cd| &cd.tag_bits)
                .unwrap_or(&empty_tags);
            let neighbor_tag_bits = world
                .card_defs
                .get(&neighbor.type_name)
                .map(|cd| &cd.tag_bits)
                .unwrap_or(&empty_tags);

            // 视觉感知
            let vision_range = 10u32; // 默认视觉范围
            if let Some(result) = perceive_vision(
                observer_id,
                (ox, oy),
                vision_range,
                observer_tag_bits,
                neighbor_id,
                (neighbor.x, neighbor.y),
                neighbor_tag_bits,
                3,       // default target_size
                false,   // occlusion placeholder
                1.0,     // light_level placeholder
                distance,
            ) {
                contributions.extend(result.need_contributions);
            }

            // 听觉感知 — 仅在声音事件时触发 (当前用占位)
            // 嗅觉感知 — 风向、气味类型占位
            if let Some(result) = perceive_smell(
                15,                 // default smell range
                0.3,                // ambient odor placeholder
                "food",             // placeholder odor_type
                (0.0, 1.0),         // wind direction (north)
                (
                    neighbor.x as f32 - ox as f32,
                    neighbor.y as f32 - oy as f32,
                ),
                distance,
            ) {
                contributions.extend(result.need_contributions);
            }
        }

        // 触觉 — 仅接触触发，当前用占位
        // (无接触事件时不产生触觉感知)

        if !contributions.is_empty() {
            perception_buffer.push((observer_id, contributions));
        }
    }

    // 第二步: 将收集到的贡献应用到实体需求 (可变遍历)
    for (entity_id, contributions) in &perception_buffer {
        if let Some(entity) = world.entities.get_mut(entity_id) {
            apply_perception_to_needs(&mut entity.needs, contributions);
        }
    }

    // ===== Phase 2: Need tick（需求衰减） =====
    for entity in world.entities.values_mut() {
        for need in &mut entity.needs {
            crate::need_match::activation::tick_need(need, delta);
        }
        crate::memory::tick_memory_decay(&mut entity.memory, world.tick_count);
    }

    // ===== Phase 2.5: 饥饿致死检查 =====
    let starvation_deaths: Vec<EntityId> = world.entities.iter()
        .filter_map(|(&id, entity)| {
            // 只检查有 Nutrition 需求的活实体
            if entity.is_corpse { return None; }
            let nutrition = entity.needs.iter()
                .find(|n| matches!(n.kind, NeedKind::Nutrition))?;
            if nutrition.current < 1.0 { return None; } // 还没饿到极限

            // 查标签
            let def = world.card_defs.get(&entity.type_name)?;
            let tags = &def.tag_bits;
            let mass = crate::axioms::consume::estimate_mass_from_tags(tags);
            let is_ecto = tags.has(tag::THERMO_ECTOTHERM.bit);
            let can_torpor = tags.has(tag::METAB_TORPOR.bit);
            let limit = crate::meta_values::fasting_endurance_ticks(mass, is_ecto, can_torpor);

            if entity.starve_days as u64 >= limit {
                Some(id)
            } else {
                None
            }
        })
        .collect();

    for id in starvation_deaths {
        if let Some(entity) = world.entities.get_mut(&id) {
            entity.is_corpse = true;
            entity.hp = 0;
        }
    }

    // starve_days 递增/重置逻辑
    for entity in world.entities.values_mut() {
        if entity.is_corpse { continue; }
        let starving = entity.needs.iter()
            .any(|n| matches!(n.kind, NeedKind::Nutrition) && n.current >= 1.0);
        if starving {
            entity.starve_days += 1;
        } else {
            entity.starve_days = 0;
        }
    }

    // ===== Phase 3: Safety block（安全阻断非安全需求） =====
    for entity in world.entities.values_mut() {
        crate::need_match::activation::apply_safety_block(&mut entity.needs);
    }

    // ===== Phase 4: Decide（匹配引擎） =====
    let entity_ids_phase4: Vec<EntityId> = world.entities.keys().copied().collect();
    for &eid in &entity_ids_phase4 {
        let env = build_env_for_entity(eid, world);
        let Some(entity) = world.entities.get_mut(&eid) else {
            continue;
        };
        if entity.execution.intention.is_none()
            || crate::need_match::execution::is_plan_failed(&entity.execution)
        {
            if let Some(action) = crate::need_match::engine::tick_need_engine(
                &mut entity.needs,
                &mut entity.execution,
                &entity.knowledge,
                &env,
                delta,
                (entity.x, entity.y),
            ) {
                world.pending_actions.push((entity.id, action));
            }
        }
    }

    // ===== Phase 5: Execute（执行元动作） =====
    let entity_ids_phase5: Vec<EntityId> = world.entities.keys().copied().collect();
    for &eid in &entity_ids_phase5 {
        let env = build_env_for_entity(eid, world);
        let Some(entity) = world.entities.get_mut(&eid) else {
            continue;
        };
        if let Some(action) =
            crate::need_match::execution::tick_execution(&mut entity.execution, &env, (entity.x, entity.y))
        {
            world.pending_actions.push((entity.id, action));
        }
    }

    // ===== Phase 6: Apply（统一生效） =====
    let pending = std::mem::take(&mut world.pending_actions);
    for (entity_id, action) in pending {
        apply_meta_action(world, entity_id, action);
    }

    // ================================================================
    // 保留现有基础设施（待逐步迁移到六阶段管线）
    // ================================================================

    crate::bulletin::maybe_update(world);

    batch_uniform_entity_updates(world, delta);
    flush_corpse_decay(world);

    mark_baseline_reactive_tick(world);

    crate::systems::tick_environment::tick_environment(world, delta);

    crate::systems::tick_reproduction::tick_reproduction(world, delta);

    flush_reactive_tick(world, delta);

    if !world.pending_spawn_ecology.is_empty() {
        EventRegistry::flush_spawn_ecology(world);
    }

    let player_ids: Vec<EntityId> = world
        .entities
        .values()
        .filter(|e| {
            !e.is_corpse && world.card_defs.get(&e.type_name)
                .is_some_and(|def| crate::world_rules::card_has_tag(def, "role:player"))
        })
        .map(|e| e.id)
        .collect();
    for id in player_ids {
        crate::player::tick_player_world(world, id, delta);
    }
}

// ===== 辅助函数 =====

/// 将 CardDef tags 转为 MaterialProperties（带 tag 派生）
fn card_to_material_properties(
    entity: &crate::world_state::Entity,
    card_defs: &std::collections::HashMap<String, crate::card_def::CardDef>,
) -> crate::need_match::search::MaterialProperties {
    let def = card_defs.get(&entity.type_name);
    let hardness = def.and_then(|d| {
        d.tags
            .iter()
            .find_map(|t| t.strip_prefix("hardness:"))
            .and_then(|v| v.parse().ok())
    });
    let density = def.and_then(|d| {
        d.tags
            .iter()
            .find_map(|t| t.strip_prefix("density:"))
            .and_then(|v| v.parse().ok())
    });
    let flammability = def.and_then(|d| {
        d.tags
            .iter()
            .find_map(|t| t.strip_prefix("flammability:"))
            .and_then(|v| v.parse().ok())
    });
    let mass_kg = def
        .and_then(|d| {
            d.tags
                .iter()
                .find_map(|t| t.strip_prefix("mass:"))
                .and_then(|v| v.parse().ok())
        })
        .unwrap_or(1.0);
    let spark_on_strike = def.map_or(false, |d| d.tags.iter().any(|t| t == "spark_on_strike"));
    let edge_present = def.map_or(false, |d| d.tags.iter().any(|t| t == "edge"));
    let tags: Vec<String> = def.map(|d| d.tags.clone()).unwrap_or_default();

    crate::need_match::search::MaterialProperties {
        hardness,
        density,
        flammability,
        spark_on_strike,
        edge_present,
        mass_kg,
        tags,
    }
}

/// 为指定实体构建感知环境 (entity_id, position, material_properties)
fn build_env_for_entity(
    entity_id: EntityId,
    world: &crate::world_state::WorldState,
) -> Vec<(
    u32,
    (u8, u8),
    crate::need_match::search::MaterialProperties,
)> {
    let Some(entity) = world.entities.get(&entity_id) else {
        return Vec::new();
    };
    let neighbors = world.spatial_index.query_radius_all_z(
        entity.x,
        entity.y,
        entity.z,
        MAX_SENSE_RANGE,
    );
    neighbors
        .iter()
        .filter_map(|&nid| {
            if nid == entity_id {
                return None;
            }
            let neighbor = world.entities.get(&nid)?;
            let props = card_to_material_properties(neighbor, &world.card_defs);
            Some((nid.0 as u32, (neighbor.x, neighbor.y), props))
        })
        .collect()
}

/// 将感知结果贡献应用到实体需求急迫度
fn apply_perception_to_needs(needs: &mut [crate::need_match::data::NeedState], contributions: &[(NeedKind, f32)]) {
    for (kind, urgency_delta) in contributions {
        for need in needs.iter_mut() {
            if need.kind == *kind {
                // 累加急迫度 — 感知推高需求
                need.urgency = (need.urgency + urgency_delta).min(1.0);
            }
        }
    }
}

/// 统一应用元动作 — 更新世界状态
fn apply_meta_action(world: &mut WorldState, entity_id: EntityId, action: MetaAction) {
    match action {
        MetaAction::Move { dx, dy } => {
            // 计算目标坐标 + 地形阻力（不可变借用块）
            let move_info = {
                let Some(entity) = world.entities.get(&entity_id) else { return };
                let new_x = ((entity.x as i16) + dx).clamp(0, 255) as u8;
                let new_y = ((entity.y as i16) + dy).clamp(0, 255) as u8;
                let terrain = crate::terrain::terrain_at(world, new_x, new_y);
                let cost = world.card_defs.get(&entity.type_name)
                    .map(|d| crate::axioms::move_check::terrain_resistance(terrain, &d.tag_bits));
                (new_x, new_y, cost)
            };
            let (new_x, new_y, cost) = move_info;

            match cost {
                Some(crate::axioms::move_check::TerrainCost::Lethal) => {
                    // 能进但掉血
                    if let Some(e) = world.entities.get_mut(&entity_id) {
                        e.x = new_x;
                        e.y = new_y;
                        e.hp = e.hp.saturating_sub(crate::axioms::move_check::lethal_terrain_damage());
                        world.spatial_index.move_entity(entity_id, new_x, new_y);
                    }
                }
                _ => {
                    // 正常移动（无 card_def 的实体也走这条路）
                    if let Some(e) = world.entities.get_mut(&entity_id) {
                        e.x = new_x;
                        e.y = new_y;
                        world.spatial_index.move_entity(entity_id, new_x, new_y);
                    }
                }
            }
        }
        MetaAction::Strike { target } => {
            // 同构 Strike 伤害计算 — strike 公理
            // 在块作用域内完成所有不可变借用，再释放借用后执行可变操作
            let damage: i32;
            {
                let Some(attacker_entity) = world.entities.get(&entity_id) else { return };
                let Some(target_entity) = world.entities.get(&target) else { return };

                let Some(attacker_def) = world.card_defs.get(&attacker_entity.type_name) else { return };
                let attacker_tags = &attacker_def.tag_bits;
                let mass = crate::axioms::consume::estimate_mass_from_tags(attacker_tags);

                let Some(target_def) = world.card_defs.get(&target_entity.type_name) else { return };
                let target_tags = &target_def.tag_bits;

                let force = crate::axioms::strike::strike_force(attacker_tags, mass, target_tags);

                // 伤害 = 力 / 1000 (N→伤害单位, 经验换算)
                damage = (force / 1000.0).ceil() as i32;
            }

            if let Some(target_e) = world.entities.get_mut(&target) {
                target_e.hp = target_e.hp.saturating_sub(damage.max(1));
            }
        }
        MetaAction::Pause { .. } => {
            // 无副作用
        }
        MetaAction::Consume { target } => {
            use crate::axioms::consume;
            use crate::meta_values::baseline_energy;
            use crate::tags::tag;

            // 在块作用域内完成所有不可变借用，然后释放借用再执行可变操作
            let target_mass: f32;
            let metab_rate: f32;
            let consumable: bool;
            {
                let Some(actor_entity) = world.entities.get(&entity_id) else { return };
                let Some(target_entity) = world.entities.get(&target) else { return };

                let Some(actor_def) = world.card_defs.get(&actor_entity.type_name) else { return };
                let Some(target_def) = world.card_defs.get(&target_entity.type_name) else { return };
                let actor_tags = &actor_def.tag_bits;
                let target_tags = &target_def.tag_bits;

                let registry = match crate::world_rules::TAG_REGISTRY.get() {
                    Some(r) => r,
                    None => return,
                };
                if !consume::can_digest(actor_tags, target_tags, target_entity.is_corpse, registry) {
                    return; // 不可消化，静默跳过
                }

                target_mass = consume::estimate_mass_from_tags(target_tags);
                metab_rate = if actor_tags.has(tag::METAB_HIGH.bit) { 1.5 }
                    else if actor_tags.has(tag::METAB_LOW.bit) { 0.5 }
                    else { 1.0 };
                consumable = true;
            }

            if consumable {
                let energy = baseline_energy(target_mass, metab_rate);

                // 更新食用者 Nutrition need
                if let Some(actor) = world.entities.get_mut(&entity_id) {
                    for need in &mut actor.needs {
                        if need.kind == NeedKind::Nutrition {
                            need.current = (need.current - energy * crate::meta_values::DIGESTION_EFFICIENCY).max(0.0);
                        }
                    }
                }

                // 标记目标被消耗
                if let Some(target_e) = world.entities.get_mut(&target) {
                    target_e.consumed = true;
                }
            }
        }
        MetaAction::Hide { cover_id } => {
            if let Some(entity) = world.entities.get_mut(&entity_id) {
                entity.host_cover_id = Some(cover_id);
                entity.in_cover = true;
            }
        }
        MetaAction::Emerge => {
            if let Some(entity) = world.entities.get_mut(&entity_id) {
                entity.host_cover_id = None;
                entity.in_cover = false;
            }
        }
        MetaAction::Signal { .. } => {
            // 信号传播 — 后续 handoff 实现
        }
        MetaAction::Receive { .. } => {
            // 接收解码 — 后续 handoff 实现
        }
        MetaAction::Reproduce { .. } => {
            // 繁殖 — 后续 handoff 实现
        }
        _ => {
            // 其余元动作暂时 stub
        }
    }
}

/// 公开包装——供 fast_tick 等外部模块调用 apply_meta_action
pub fn apply_meta_action_public(world: &mut WorldState, entity_id: EntityId, action: MetaAction) {
    apply_meta_action(world, entity_id, action);
}

// ===== 兼容别名 =====

pub fn mark_baseline_herbivore_tick(world: &mut WorldState) {
    mark_baseline_reactive_tick(world);
}

pub fn flush_herbivore_tick(world: &mut WorldState, delta: f32) {
    flush_reactive_tick(world, delta);
}

pub fn flush_reactive_entity_tick(world: &mut WorldState, id: EntityId, delta: f32) {
    tick_reactive(world, id, delta);
}
