use std::collections::HashMap;

use bevy::prelude::*;

use crate::card_def::CardDef;
use crate::card_style::card_style;
use crate::coords::card_world_pos;
use crate::grid_render::SimWorld;
use crate::panel_ui::UiFont;
use crate::visual_config::{CARD_BORDER_PAD, CARD_SIZE, STACK_OFFSET_Y};
use crate::render::move_animation::MoveAnimating;
use crate::world_state::Entity as SimEntity;
use crate::world_view::{WorldRootEntity, WorldView};
use bevy::prelude::Entity;

#[derive(Component)]
pub struct CardVisual {
    pub entity_id: u64,
}

#[derive(Component)]
pub struct CardIconText;

#[derive(Component)]
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
    world_root: Res<WorldRootEntity>,
    mut roots: Query<(
        Entity,
        &CardVisual,
        &mut Transform,
        &Children,
        Option<&MoveAnimating>,
    )>,
    mut card_texts: ParamSet<(
        Query<&mut Text, With<CardIconText>>,
        Query<&mut Text, With<CardNameText>>,
    )>,
) {
    let stacks = stack_indices(&sim.0);
    let icon_font = 22.0 / view.zoom;
    let name_font = 12.0 / view.zoom;
    let mut existing: HashMap<u64, Entity> =
        roots.iter().map(|(e, c, _, _, _)| (c.entity_id, e)).collect();

    for entity in sim.0.entities.values() {
        if entity.in_pool || entity.in_tree || entity.in_ground || entity.in_den || entity.in_burrow {
            existing.remove(&entity.id.0);
            continue;
        }
        let stack_index = stacks
            .get(&(entity.x, entity.y))
            .and_then(|list| list.iter().position(|id| *id == entity.id.0))
            .unwrap_or(0) as f32;
        let mut pos = card_world_pos(entity.x, entity.y, entity.id.0, Some(&sim.0));
        pos.y += stack_index * STACK_OFFSET_Y;
        let def = sim.0.card_defs.get(&entity.type_name);
        let style = def
            .map(|d| card_style(&entity.type_name, d))
            .unwrap_or_else(|| card_style("grass", &fallback_def()));

        if let Some(entity_id) = existing.remove(&entity.id.0) {
            if let Ok((_, _, mut transform, children, animating)) = roots.get_mut(entity_id) {
                if animating.is_none() {
                    transform.translation = pos;
                }
                for child in children.iter() {
                    if let Ok(mut text) = card_texts.p0().get_mut(*child) {
                        if let Some(section) = text.sections.get_mut(0) {
                            section.style.font_size = icon_font;
                        }
                    }
                    if let Ok(mut text) = card_texts.p1().get_mut(*child) {
                        if let Some(section) = text.sections.get_mut(0) {
                            section.style.font_size = name_font;
                        }
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
                    Text2dBundle {
                        text: Text::from_section(
                            icon,
                            TextStyle {
                                font: font.0.clone(),
                                font_size: icon_font,
                                color: style.text,
                            },
                        ),
                        transform: Transform::from_xyz(0.0, icon_y, 0.2)
                            .with_scale(Vec3::new(1.0, -1.0, 1.0)),
                        ..default()
                    },
                    CardIconText,
                ));
                parent.spawn((
                    Text2dBundle {
                        text: Text::from_section(
                            name,
                            TextStyle {
                                font: font.0.clone(),
                                font_size: name_font,
                                color: style.text,
                            },
                        ),
                        transform: Transform::from_xyz(0.0, name_y, 0.2)
                            .with_scale(Vec3::new(1.0, -1.0, 1.0)),
                        ..default()
                    },
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
