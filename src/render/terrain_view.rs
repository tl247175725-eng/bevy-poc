//! Terrain cell color/label refresh when sim or river stress changes.

use bevy::prelude::*;

use crate::grid_render::{SimWorld, TerrainCell, TerrainLabel};
use crate::panel_ui::UiFont;
use crate::sim_clock::SimClock;
use crate::terrain::surface_label_with_stress;
use crate::terrain_colors::{cell_color_with_stress, rgba_to_f32};

const LABEL_MUTED: Color = Color::srgba(0.2, 0.14, 0.08, 0.55);

#[derive(Resource, Default)]
pub struct TerrainVisualRevision {
    pub last_stress_key: i32,
    pub last_sim_tick: u64,
}

pub fn sync_terrain_visuals(
    sim: Res<SimWorld>,
    clock: Res<SimClock>,
    font: Res<UiFont>,
    mut revision: ResMut<TerrainVisualRevision>,
    mut cells: Query<(&TerrainCell, &mut Sprite, &Children, &Transform)>,
    mut labels: Query<&mut Text, With<TerrainLabel>>,
) {
    let stress_key = clock.river_stress.round() as i32;
    let tick_key = sim.0.tick_count;
    if revision.last_stress_key == stress_key && revision.last_sim_tick == tick_key {
        return;
    }
    revision.last_stress_key = stress_key;
    revision.last_sim_tick = tick_key;

    for (cell, mut sprite, children, _transform) in &mut cells {
        let gx = cell.x;
        let gy = cell.y;
        let (r, g, b, a) = rgba_to_f32(cell_color_with_stress(
            &sim.0,
            gx,
            gy,
            clock.river_stress,
        ));
        sprite.color = Color::srgba(r, g, b, a);

        let label_text =
            surface_label_with_stress(&sim.0, gx, gy, clock.river_stress).unwrap_or_default();
        for child in children.iter() {
            if let Ok(mut text) = labels.get_mut(*child) {
                if let Some(section) = text.sections.first_mut() {
                    section.value = label_text.clone();
                    section.style.font = font.0.clone();
                    section.style.color = LABEL_MUTED;
                }
            }
        }
    }
}
