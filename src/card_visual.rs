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

#[derive(Component, Clone)]
pub struct GroupCardMarker {
    pub count: u8,
    pub cell_x: u8,
    pub cell_y: u8,
    pub type_name: String,
}

#[derive(Component)]
pub struct GroupCountText;

#[derive(Component)]
pub struct GroupLabelText;

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

fn flock_groups(world: &crate::world_state::WorldState) -> HashMap<(u8, u8, String), Vec<u64>> {
    let mut groups: HashMap<(u8, u8, String), Vec<u64>> = HashMap::new();
    let mut ids: Vec<_> = world.entities.keys().map(|id| id.0).collect();
    ids.sort_unstable();
    for id in ids {
        if let Some(e) = world.entities.get(&crate::spatial_index::EntityId(id)) {
            if e.in_pool || e.in_tree || e.in_ground || e.in_den || e.in_burrow {
                continue;
            }
            groups
                .entry((e.x, e.y, e.type_name.clone()))
                .or_default()
                .push(id);
        }
    }
    groups
}

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
    mut roots: Query<
        (
            Entity,
            &CardVisual,
            &mut Transform,
            &Children,
            Option<&MoveAnimating>,
            &mut Visibility,
        ),
        Without<GroupCardMarker>,
    >,
    mut group_roots: Query<
        (
            Entity,
            &mut GroupCardMarker,
            &mut Transform,
            &Children,
        ),
        Without<CardVisual>,
    >,
    mut card_texts: ParamSet<(
        Query<&mut Text, With<CardIconText>>,
        Query<&mut Text, With<CardNameText>>,
        Query<&mut Text, With<GroupCountText>>,
        Query<&mut Text, With<GroupLabelText>>,
    )>,
) {
    // #region agent log
    {
        use std::io::Write;
        let path = crate::assets_util::manifest_dir().join("debug-b7ff4e.log");
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
                r#"{{"sessionId":"b7ff4e","hypothesisId":"H1","location":"card_visual.rs:sync_card_visuals:entry","message":"system entered","data":{{"roots_count":{},"group_roots_count":{}}},"timestamp":{}}}"#,
                roots.iter().count(),
                group_roots.iter().count(),
                ts
            );
        }
    }
    // #endregion
    let stacks = stack_indices(&sim.0);
    let groups = flock_groups(&sim.0);
    let grouped_members: HashMap<u64, (u8, u8, String)> = groups
        .iter()
        .filter(|(_, members)| members.len() >= 2)
        .flat_map(|((x, y, type_name), members)| {
            members
                .iter()
                .map(move |id| (*id, (*x, *y, type_name.clone())))
        })
        .collect();

    let icon_font = 22.0 / view.zoom;
    let name_font = 12.0 / view.zoom;
    let badge_font = 10.0 / view.zoom;
    let mut existing: HashMap<u64, Entity> = roots
        .iter()
        .map(|(e, c, _, _, _, _)| (c.entity_id, e))
        .collect();
    let mut existing_groups: HashMap<(u8, u8, String), Entity> = group_roots
        .iter()
        .map(|(e, g, _, _)| ((g.cell_x, g.cell_y, g.type_name.clone()), e))
        .collect();

    for entity in sim.0.entities.values() {
        if entity.in_pool || entity.in_tree || entity.in_ground || entity.in_den || entity.in_burrow {
            existing.remove(&entity.id.0);
            continue;
        }
        let in_group = grouped_members.contains_key(&entity.id.0);
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
            if let Ok((_, _, mut transform, children, animating, mut visibility)) =
                roots.get_mut(entity_id)
            {
                *visibility = if in_group {
                    Visibility::Hidden
                } else {
                    Visibility::Visible
                };
                if animating.is_none() && !in_group {
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
                in_group,
            );
        }
    }

    for entity in existing.values() {
        commands.entity(*entity).despawn_recursive();
    }

    for (key, members) in &groups {
        if members.len() < 2 {
            continue;
        }
        let (cell_x, cell_y, type_name) = key;
        let count = members.len().min(255) as u8;
        let pos = card_world_pos(*cell_x, *cell_y, members[0], Some(&sim.0));
        let def = sim.0.card_defs.get(type_name);
        let style = def
            .map(|d| card_style(type_name, d))
            .unwrap_or_else(|| card_style("grass", &fallback_def()));
        let display_name = def.map(|d| d.display_name.clone()).unwrap_or_else(|| type_name.clone());

        if let Some(group_entity) = existing_groups.remove(key) {
            if let Ok((_, mut marker, mut transform, children)) = group_roots.get_mut(group_entity)
            {
                marker.count = count;
                transform.translation = pos;
                for child in children.iter() {
                    if let Ok(mut text) = card_texts.p2().get_mut(*child) {
                        if text.sections.len() >= 2 {
                            text.sections[1].value = format!(" ({count})");
                            text.sections[0].style.font_size = badge_font;
                            text.sections[1].style.font_size = badge_font;
                        }
                    }
                    if let Ok(mut text) = card_texts.p3().get_mut(*child) {
                        if let Some(section) = text.sections.get_mut(0) {
                            section.value = display_name.clone();
                            section.style.font_size = name_font;
                        }
                    }
                }
            }
        } else {
            spawn_group_card_visual(
                &mut commands,
                &world_root,
                *cell_x,
                *cell_y,
                type_name,
                count,
                def,
                &style,
                &font,
                pos,
                icon_font,
                name_font,
                badge_font,
            );
        }
    }

    for entity in existing_groups.values() {
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

fn spawn_group_card_visual(
    commands: &mut Commands,
    root: &WorldRootEntity,
    cell_x: u8,
    cell_y: u8,
    type_name: &str,
    count: u8,
    def: Option<&CardDef>,
    style: &crate::card_style::CardStyle,
    font: &UiFont,
    pos: Vec3,
    icon_font: f32,
    name_font: f32,
    badge_font: f32,
) {
    let display_name = def.map(|d| d.display_name.clone()).unwrap_or_else(|| type_name.to_string());
    let icon = def.map(|d| d.icon.clone()).unwrap_or_else(|| "?".to_string());
    let border_size = CARD_SIZE + CARD_BORDER_PAD * 2.0;
    let icon_y = 7.0;
    let name_y = -15.0;
    let badge_size = 14.0;

    commands.entity(root.0).with_children(|world| {
        world
            .spawn((
                SpatialBundle {
                    transform: Transform::from_translation(pos),
                    ..default()
                },
                GroupCardMarker {
                    count,
                    cell_x,
                    cell_y,
                    type_name: type_name.to_string(),
                },
            ))
            .with_children(|parent| {
                parent.spawn(SpriteBundle {
                    sprite: Sprite {
                        color: style.border,
                        custom_size: Some(Vec2::splat(border_size)),
                        ..default()
                    },
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
                parent.spawn(SpriteBundle {
                    sprite: Sprite {
                        color: Color::srgb(0.2, 0.7, 0.3),
                        custom_size: Some(Vec2::splat(badge_size)),
                        ..default()
                    },
                    transform: Transform::from_xyz(18.0, -18.0, 0.15),
                    ..default()
                });
                parent.spawn((
                    Text2dBundle {
                        text: Text::from_sections([
                            TextSection::new(
                                "群",
                                TextStyle {
                                    font: font.0.clone(),
                                    font_size: badge_font,
                                    color: Color::WHITE,
                                },
                            ),
                            TextSection::new(
                                format!(" ({count})"),
                                TextStyle {
                                    font: font.0.clone(),
                                    font_size: badge_font,
                                    color: Color::WHITE,
                                },
                            ),
                        ]),
                        transform: Transform::from_xyz(18.0, -18.0, 0.25)
                            .with_scale(Vec3::new(1.0, -1.0, 1.0)),
                        ..default()
                    },
                    GroupCountText,
                ));
                parent.spawn((
                    Text2dBundle {
                        text: Text::from_section(
                            icon,
                            TextStyle {
                                font: font.0.clone(),
                                font_size: icon_font,
                                color: Color::WHITE,
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
                            display_name,
                            TextStyle {
                                font: font.0.clone(),
                                font_size: name_font,
                                color: Color::WHITE,
                            },
                        ),
                        transform: Transform::from_xyz(0.0, name_y, 0.2)
                            .with_scale(Vec3::new(1.0, -1.0, 1.0)),
                        ..default()
                    },
                    GroupLabelText,
                ));
            });
    });
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
    in_group: bool,
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
                    visibility: if in_group {
                        Visibility::Hidden
                    } else {
                        Visibility::Visible
                    },
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
