//! Godot `main.tscn` layout: `WorldArea` (left, clip) + egui `SidePanel` (right, 360px).

use bevy::prelude::*;
use bevy::render::camera::{ScalingMode, Viewport};
use bevy::window::PrimaryWindow;

use crate::visual_config::{world_area_width_for, WORLD_BG};
use crate::world_view::WorldView;

/// Renders the world into the left `WorldArea` viewport (clip_contents equivalent).
#[derive(Component)]
pub struct WorldCamera;

#[derive(Resource, Clone, Copy)]
pub struct ViewportLayout {
    pub window_width: f32,
    pub window_height: f32,
    pub scale_factor: f32,
}

impl Default for ViewportLayout {
    fn default() -> Self {
        Self {
            window_width: 1280.0,
            window_height: 720.0,
            scale_factor: 1.0,
        }
    }
}

impl ViewportLayout {
    pub fn world_area_width(&self) -> f32 {
        world_area_width_for(self.window_width)
    }

    pub fn world_area_size(&self) -> Vec2 {
        Vec2::new(self.world_area_width(), self.window_height)
    }

    pub fn cursor_in_world_area(&self, cursor: Vec2) -> bool {
        cursor.x >= 0.0
            && cursor.x < self.world_area_width()
            && cursor.y >= 0.0
            && cursor.y < self.window_height
    }
}

pub fn apply_viewport_layout(
    window: &Window,
    layout: &mut ViewportLayout,
    world_cam: &mut Query<
        (&mut Camera, &mut OrthographicProjection, &mut Transform),
        With<WorldCamera>,
    >,
    view: &WorldView,
) {
    layout.window_width = window.width();
    layout.window_height = window.height();
    layout.scale_factor = window.scale_factor();

    let area_w = layout.world_area_width();
    let area_h = layout.window_height;
    let physical_w = (area_w * layout.scale_factor).round() as u32;
    let physical_h = (area_h * layout.scale_factor).round() as u32;

    for (mut camera, mut projection, mut transform) in world_cam {
        camera.viewport = Some(Viewport {
            physical_position: UVec2::ZERO,
            physical_size: UVec2::new(physical_w.max(1), physical_h.max(1)),
            ..default()
        });
        projection.scaling_mode = ScalingMode::Fixed {
            width: area_w,
            height: area_h,
        };
        projection.scale = 1.0;
        transform.translation = Vec3::new(area_w * 0.5, area_h * 0.5, 0.0);
    }

    let _ = view;
}

pub fn sync_viewport_layout(
    window: Query<&Window, (With<PrimaryWindow>, Changed<Window>)>,
    mut layout: ResMut<ViewportLayout>,
    view: Res<WorldView>,
    mut world_cam: Query<
        (&mut Camera, &mut OrthographicProjection, &mut Transform),
        With<WorldCamera>,
    >,
) {
    let Ok(window) = window.get_single() else {
        return;
    };
    apply_viewport_layout(window, &mut layout, &mut world_cam, &view);
}

pub fn bootstrap_viewport_layout(
    window: Query<&Window, With<PrimaryWindow>>,
    mut layout: ResMut<ViewportLayout>,
    view: Res<WorldView>,
    mut world_cam: Query<
        (&mut Camera, &mut OrthographicProjection, &mut Transform),
        With<WorldCamera>,
    >,
) {
    let Ok(window) = window.get_single() else {
        return;
    };
    apply_viewport_layout(window, &mut layout, &mut world_cam, &view);
}

pub fn setup_cameras(mut commands: Commands) {
    let bg = Color::srgb(WORLD_BG.0, WORLD_BG.1, WORLD_BG.2);
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                order: 0,
                clear_color: ClearColorConfig::Custom(bg),
                ..default()
            },
            projection: OrthographicProjection {
                near: -1000.0,
                far: 1000.0,
                ..OrthographicProjection::default_2d()
            },
            ..default()
        },
        WorldCamera,
    ));
}
