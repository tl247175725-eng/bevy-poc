use bevy::prelude::*;

use crate::coords::{card_world_pos, cell_center};
use crate::initial_spawn::spawn_initial_world;
use crate::panel_ui::UiFont;
use crate::terrain::surface_label;
use crate::terrain_colors::{cell_color, rgba_to_f32};
use crate::visual_config::{CELL_SIZE, SELECTION_RING, SELECTION_RING_SIZE, SELECTION_RING_WIDTH};
use crate::world_rules::{GRID_HEIGHT, GRID_WIDTH};
use crate::world_state::WorldState;
use crate::world_view::{WorldRoot, WorldRootEntity};

pub use crate::visual_config::{CARD_SIZE, CELL_SIZE as VISUAL_CELL_SIZE};

const GRID_LINE: Color = Color::srgba(0.2, 0.14, 0.08, 0.22);
const LABEL_MUTED: Color = Color::srgba(0.2, 0.14, 0.08, 0.55);

#[derive(Resource)]
pub struct SimWorld(pub WorldState);

#[derive(Component)]
pub struct GridMesh;

#[derive(Component)]
pub struct TerrainCell {
    pub x: u8,
    pub y: u8,
}

#[derive(Component)]
pub struct TerrainLabel;

#[derive(Component)]
pub struct SelectionOutline;

fn cell_center_local(x: u8, y: u8) -> Vec3 {
    cell_center(x, y)
}

pub fn setup_world_root(mut commands: Commands) {
    let entity = commands.spawn((SpatialBundle::default(), WorldRoot)).id();
    commands.insert_resource(WorldRootEntity(entity));
}

pub fn setup_sim_world(mut commands: Commands) {
    commands.insert_resource(SimWorld(spawn_initial_world()));
}

pub fn setup_grid(
    mut commands: Commands,
    sim: Res<SimWorld>,
    root: Res<WorldRootEntity>,
    font: Res<UiFont>,
) {
    commands.entity(root.0).with_children(|world| {
        world
            .spawn((SpatialBundle::default(), GridMesh))
            .with_children(|cells| {
                for y in 0..GRID_HEIGHT {
                    for x in 0..GRID_WIDTH {
                        spawn_terrain_cell(cells, &sim.0, x, y, &font);
                    }
                }
            });
    });
}

fn spawn_terrain_cell(
    cells: &mut ChildBuilder,
    world: &WorldState,
    x: u8,
    y: u8,
    font: &UiFont,
) {
    let center = cell_center_local(x, y);
    let (r, g, b, a) = rgba_to_f32(cell_color(world, x, y));
    cells
        .spawn((
            TerrainCell { x, y },
            SpriteBundle {
                sprite: Sprite {
                    color: Color::srgba(r, g, b, a),
                    custom_size: Some(Vec2::splat(CELL_SIZE)),
                    ..default()
                },
                transform: Transform::from_translation(center),
                ..default()
            },
        ))
        .with_children(|cell| {
            let hw = CELL_SIZE * 0.5;
            for (offset, size) in [
                (Vec2::new(0.0, hw - 0.5), Vec2::new(CELL_SIZE, 1.0)),
                (Vec2::new(0.0, -hw + 0.5), Vec2::new(CELL_SIZE, 1.0)),
                (Vec2::new(-hw + 0.5, 0.0), Vec2::new(1.0, CELL_SIZE)),
                (Vec2::new(hw - 0.5, 0.0), Vec2::new(1.0, CELL_SIZE)),
            ] {
                cell.spawn(SpriteBundle {
                    sprite: Sprite {
                        color: GRID_LINE,
                        custom_size: Some(size),
                        ..default()
                    },
                    transform: Transform::from_translation(offset.extend(0.05)),
                    ..default()
                });
            }

            if let Some(text) = surface_label(world, x, y) {
                cell.spawn((
                    TerrainLabel,
                    Text2dBundle {
                        text: Text::from_section(
                            text,
                            TextStyle {
                                font: font.0.clone(),
                                font_size: 11.0,
                                color: LABEL_MUTED,
                            },
                        ),
                        transform: Transform::from_xyz(0.0, hw - 12.0, 0.1)
                            .with_scale(Vec3::new(1.0, -1.0, 1.0)),
                        ..default()
                    },
                ));
            }
        });
}

pub use crate::coords::{grid_to_world, grid_to_world_in, world_to_grid};

pub fn sync_selection_border(
    mut commands: Commands,
    selection: Res<crate::ui_interaction::SelectionState>,
    outline: Query<Entity, With<SelectionOutline>>,
    root: Res<WorldRootEntity>,
    sim: Res<SimWorld>,
) {
    if !selection.is_changed() {
        return;
    }
    for entity in &outline {
        commands.entity(entity).despawn_recursive();
    }
    if !selection.border_visible {
        return;
    }
    let Some(target) = &selection.target else {
        return;
    };
    let center = card_world_pos(target.cell_x, target.cell_y, 0, Some(&sim.0));
    let color = Color::srgb(SELECTION_RING.0, SELECTION_RING.1, SELECTION_RING.2);
    let size = SELECTION_RING_SIZE;
    let hw = size / 2.0;
    let t = SELECTION_RING_WIDTH;
    commands.entity(root.0).with_children(|world| {
        world
            .spawn((
                SpatialBundle {
                    transform: Transform::from_translation(Vec3::new(center.x, center.y, 20.0)),
                    ..default()
                },
                SelectionOutline,
            ))
            .with_children(|parent| {
                for (offset, sz) in [
                    (Vec2::new(0.0, hw - t / 2.0), Vec2::new(size, t)),
                    (Vec2::new(0.0, -hw + t / 2.0), Vec2::new(size, t)),
                    (Vec2::new(-hw + t / 2.0, 0.0), Vec2::new(t, size)),
                    (Vec2::new(hw - t / 2.0, 0.0), Vec2::new(t, size)),
                ] {
                    parent.spawn(SpriteBundle {
                        sprite: Sprite {
                            color,
                            custom_size: Some(sz),
                            ..default()
                        },
                        transform: Transform::from_translation(offset.extend(0.0)),
                        ..default()
                    });
                }
            });
    });
}
