//! Godot `WorldManager` view transform — `_view_zoom` + `_view_offset` on `WorldRoot`.

use bevy::prelude::*;

use crate::visual_config::{world_height, world_width};

pub const VIEW_ZOOM_MIN: f32 = 0.4;
pub const VIEW_ZOOM_MAX: f32 = 3.0;
pub const VIEW_ZOOM_STEP: f32 = 0.12;

#[derive(Resource, Clone, Copy)]
pub struct WorldView {
    pub zoom: f32,
    pub offset: Vec2,
}

impl Default for WorldView {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            offset: Vec2::ZERO,
        }
    }
}

impl WorldView {
    pub fn clamp_zoom(zoom: f32) -> f32 {
        zoom.clamp(VIEW_ZOOM_MIN, VIEW_ZOOM_MAX)
    }

    pub fn base_position(area_size: Vec2, zoom: f32) -> Vec2 {
        let w = world_width() * zoom;
        let h = world_height() * zoom;
        Vec2::new(
            (area_size.x - w).max(0.0) * 0.5,
            (area_size.y - h).max(0.0) * 0.5,
        )
    }

    pub fn root_translation(&self, area_size: Vec2) -> Vec2 {
        Self::base_position(area_size, self.zoom) + self.offset
    }

    /// Bevy `Transform` for `WorldRoot` (Godot screen origin + Y-flip offset).
    pub fn root_bevy_translation(&self, area_size: Vec2) -> Vec2 {
        let root = self.root_translation(area_size);
        Vec2::new(root.x, root.y + area_size.y)
    }

    pub fn clamp_offset(&mut self, area_size: Vec2) {
        let w = world_width() * self.zoom;
        let h = world_height() * self.zoom;
        let max_ox = (w - area_size.x).max(0.0);
        let max_oy = (h - area_size.y).max(0.0);
        self.offset.x = self.offset.x.clamp(-max_ox, max_ox);
        self.offset.y = self.offset.y.clamp(-max_oy, max_oy);
    }

    /// Area-local cursor (top-left origin) → Godot-style world coords (Y down).
    pub fn area_to_world(&self, area_pos: Vec2, area_size: Vec2) -> Vec2 {
        let position = self.root_translation(area_size);
        Vec2::new(
            (area_pos.x - position.x) / self.zoom,
            (area_pos.y - position.y) / self.zoom,
        )
    }

    pub fn zoom_wheel(&mut self, wheel_dir: i32, area_mouse: Vec2, area_size: Vec2) {
        if wheel_dir == 0 {
            return;
        }
        let old_zoom = self.zoom;
        self.zoom = Self::clamp_zoom(self.zoom + wheel_dir as f32 * VIEW_ZOOM_STEP * self.zoom);
        if (self.zoom - old_zoom).abs() < f32::EPSILON {
            return;
        }
        let position = Self::base_position(area_size, old_zoom) + self.offset;
        let world_pt = Vec2::new(
            (area_mouse.x - position.x) / old_zoom,
            (area_mouse.y - position.y) / old_zoom,
        );
        let new_base = Self::base_position(area_size, self.zoom);
        self.offset = Vec2::new(
            area_mouse.x - new_base.x - world_pt.x * self.zoom,
            area_mouse.y - new_base.y - world_pt.y * self.zoom,
        );
        self.clamp_offset(area_size);
    }

    pub fn pan_by(&mut self, delta: Vec2, area_size: Vec2) {
        self.offset += delta;
        self.clamp_offset(area_size);
    }
}

#[derive(Component)]
pub struct WorldRoot;

#[derive(Resource, Clone, Copy)]
pub struct WorldRootEntity(pub Entity);

pub fn sync_world_root_transform(
    view: Res<WorldView>,
    layout: Res<crate::viewport_layout::ViewportLayout>,
    mut roots: Query<&mut Transform, With<WorldRoot>>,
) {
    let area = layout.world_area_size();
    let translation = view.root_bevy_translation(area);
    for mut transform in &mut roots {
        transform.translation = translation.extend(0.0);
        // Y flip: Godot world Y grows downward; Bevy grows upward.
        transform.scale = Vec3::new(view.zoom, -view.zoom, 1.0);
    }
}
