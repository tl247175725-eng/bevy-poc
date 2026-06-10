use std::collections::HashMap;

use bevy::prelude::*;

use crate::card_def::CardDef;
use crate::card_style::card_style;
use crate::coords::card_world_pos;
use crate::grid_render::SimWorld;
use crate::game_ui_panel::{spawn_text2d, UiFont};
use crate::visual_config::{CARD_BORDER_PAD, CARD_SIZE, STACK_OFFSET_Y};
use crate::ui_interaction::{DragState, GhostPlaceMode};
use crate::world_state::Entity as SimEntity;
use crate::world_view::{WorldRootEntity, WorldView};
use bevy::prelude::Entity;

pub const CARD_SLIDE_SPEED: f32 = 8.0;
pub const CARD_SPRINT_SLIDE_SPEED: f32 = 15.0;

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct CardVisual {
    pub entity_id: u64,
    /// Current rendered position (lerped each frame).
    pub visual_pos: Vec3,
    /// Destination from simulation grid (updated by `sync_card_visuals`).
    pub target_pos: Vec3,
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct CardIconText;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct CardNameText;

// #region agent log
fn agent_debug_log(hypothesis_id: &str, location: &str, message: &str, data_json: &str) {
    use std::io::Write;
    let path = crate::assets_util::manifest_dir().join("debug-1db2b3.log");
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
    {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);
        let _ = writeln!(
            f,
            r#"{{"sessionId":"1db2b3","hypothesisId":"{hypothesis_id}","location":"{location}","message":"{message}","data":{data_json},"timestamp":{ts}}}"#
        );
    }
}
// #endregion

pub fn stack_indices(world: &crate::world_state::WorldState) -> HashMap<(u8, u8), Vec<u64>> {
    let mut cells: HashMap<(u8, u8), Vec<u64>> = HashMap::new();
    let mut ids: Vec<_> = world.entities.keys().map(|id| id.0).collect();
    ids.sort_unstable();
    for id in ids {
        if let Some(e) = world.entities.get(&crate::spatial_index::EntityId(id)) {
            if e.in_pool || e.in_tree || e.in_ground || e.in_den || e.in_burrow {
                continue;
            }
            cells.entry((e.x, e.y)).or_default().push(id);
        }
    }
    for list in cells.values_mut() {
        list.sort_unstable();
    }
    cells
}

pub fn sync_card_visuals(
    mut commands: Commands,
    sim: Res<SimWorld>,
    font: Res<UiFont>,
    view: Res<WorldView>,
    drag: Res<DragState>,
    ghost: Res<GhostPlaceMode>,
    world_root: Res<WorldRootEntity>,
    mut roots: Query<(Entity, &mut CardVisual, &mut Transform, &Children)>,
    mut card_fonts: ParamSet<(
        Query<&mut TextFont, With<CardIconText>>,
        Query<&mut TextFont, With<CardNameText>>,
    )>,
) {
    let stacks = stack_indices(&sim.0);

    let icon_font = 22.0 / view.zoom;
    let name_font = 12.0 / view.zoom;
    let mut existing: HashMap<u64, Entity> = roots
        .iter()
        .map(|(e, c, _, _)| (c.entity_id, e))
        .collect();

    for entity in sim.0.entities.values() {
        if entity.in_pool || entity.in_tree || entity.in_ground || entity.in_den || entity.in_burrow {
            existing.remove(&entity.id.0);
            continue;
        }
        let stack_index = stacks
            .get(&(entity.x, entity.y))
            .filter(|list| list.len() > 1)
            .and_then(|list| list.iter().position(|id| *id == entity.id.0))
            .unwrap_or(0) as f32;
        let mut pos = card_world_pos(entity.x, entity.y, entity.id.0, Some(&sim.0));
        if stack_index > 0.0 {
            pos.y += stack_index * STACK_OFFSET_Y;
        }
        // Z-layer: structures above terrain, creatures above structures
        let def = sim.0.card_defs.get(&entity.type_name);
        if let Some(d) = def {
            if d.type_name == "mountain" || d.type_name == "stone" {
                pos.z += 1.0; // structures on top of terrain
            }
            if crate::world_rules::card_has_tag(d, "being") {
                pos.z += 2.0; // creatures on top of everything
            }
        }
        let style = def
            .map(|d| card_style(&entity.type_name, d))
            .unwrap_or_else(|| card_style("grass", &fallback_def()));

        let skip_position = entity_dragged(entity.id.0, &drag, &ghost);

        if let Some(entity_id) = existing.remove(&entity.id.0) {
            if let Ok((_, mut cv, mut transform, children)) = roots.get_mut(entity_id) {
                if !skip_position {
                    cv.target_pos = pos;
                }
                transform.translation = cv.visual_pos;
                for child in children.iter() {
                    if let Ok(mut tf) = card_fonts.p0().get_mut(*child) {
                        tf.font_size = icon_font;
                    }
                    if let Ok(mut tf) = card_fonts.p1().get_mut(*child) {
                        tf.font_size = name_font;
                    }
                }
            }
        } else {
            spawn_card_visual(
                &mut commands,
                &world_root,
                entity,
                def,
                &style,
                &font,
                pos,
                icon_font,
                name_font,
            );
        }
    }

    for entity in existing.values() {
        commands.entity(*entity).despawn_recursive();
    }

    // #region agent log
    static LOGGED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    if !LOGGED.swap(true, std::sync::atomic::Ordering::Relaxed) {
        agent_debug_log(
            "H1",
            "card_visual.rs:sync_card_visuals",
            "sync_card_visuals completed without query conflict",
            &format!(
                r#"{{"zoom":{},"card_roots":{}}}"#,
                view.zoom,
                roots.iter().count()
            ),
        );
    }
    // #endregion
}

pub fn entity_dragged(entity_id: u64, drag: &DragState, ghost: &GhostPlaceMode) -> bool {
    (drag.dragging && drag.entity_id.is_some_and(|id| id.0 == entity_id))
        || (ghost.dragging && ghost.entity_id.is_some_and(|id| id.0 == entity_id))
}

/// Lerp `visual_pos` toward `target_pos` each frame (replaces bevy_tweening).
pub fn slide_cards(
    time: Res<Time>,
    drag: Res<DragState>,
    ghost: Res<GhostPlaceMode>,
    mut cards: Query<(&mut Transform, &mut CardVisual, Option<&crate::render::move_animation::MoveAnimating>)>,
) {
    let dt = time.delta_secs();
    for (mut transform, mut cv, animating) in &mut cards {
        if entity_dragged(cv.entity_id, &drag, &ghost) {
            continue;
        }
        let speed = animating
            .map(|a| a.lerp_speed)
            .unwrap_or(CARD_SLIDE_SPEED);
        let t = (dt * speed).clamp(0.0, 1.0);
        cv.visual_pos = cv.visual_pos.lerp(cv.target_pos, t);
        transform.translation = cv.visual_pos;
    }
}

fn fallback_def() -> CardDef {
    CardDef {
        type_name: "?".into(),
        display_name: "?".into(),
        icon: "?".into(),
        tags: vec![],
        color: (200, 200, 200, 255),
        hp: 1,
        is_rooted: false,
    }
}

fn spawn_card_visual(
    commands: &mut Commands,
    root: &WorldRootEntity,
    entity: &SimEntity,
    def: Option<&CardDef>,
    style: &crate::card_style::CardStyle,
    font: &UiFont,
    pos: Vec3,
    icon_font: f32,
    name_font: f32,
) {
    let (icon, name) = labels(entity, def);
    let border_size = CARD_SIZE + CARD_BORDER_PAD * 2.0;
    // Godot: icon label at (0,4) 50×28; name at (0, CARD_SIZE-18) 50×16.
    let icon_y = 7.0;
    let name_y = -15.0;

    commands.entity(root.0).with_children(|world| {
        world
            .spawn((
                SpatialBundle {
                    transform: Transform::from_translation(pos),
                    ..default()
                },
                CardVisual {
                    entity_id: entity.id.0,
                    visual_pos: pos,
                    target_pos: pos,
                },
            ))
            .with_children(|parent| {
                parent.spawn(SpriteBundle {
                    sprite: Sprite {
                        color: style.border,
                        custom_size: Some(Vec2::splat(border_size)),
                        ..default()
                    },
                    transform: Transform::from_xyz(0.0, 0.0, 0.0),
                    ..default()
                });
                parent.spawn(SpriteBundle {
                    sprite: Sprite {
                        color: style.bg,
                        custom_size: Some(Vec2::splat(CARD_SIZE)),
                        ..default()
                    },
                    transform: Transform::from_xyz(0.0, 0.0, 0.01),
                    ..default()
                });
                parent.spawn((
                    spawn_text2d(
                        icon,
                        font,
                        icon_font,
                        style.text,
                        Transform::from_xyz(0.0, icon_y, 0.2)
                            .with_scale(Vec3::new(1.0, -1.0, 1.0)),
                    ),
                    CardIconText,
                ));
                parent.spawn((
                    spawn_text2d(
                        name,
                        font,
                        name_font,
                        style.text,
                        Transform::from_xyz(0.0, name_y, 0.2)
                            .with_scale(Vec3::new(1.0, -1.0, 1.0)),
                    ),
                    CardNameText,
                ));
            });
    });
}

fn labels(entity: &SimEntity, def: Option<&CardDef>) -> (String, String) {
    if let Some(d) = def {
        (
            d.icon.clone(),
            d.display_name.clone(),
        )
    } else {
        (entity.type_name.chars().take(1).collect(), entity.type_name.clone())
    }
}
