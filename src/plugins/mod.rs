//! Application plugins — sim/render/input/ui separation.

use bevy::prelude::*;
use bevy::render::texture::ImagePlugin;
use bevy::window::WindowResizeConstraints;
use bevy_egui::{EguiPlugin, EguiSet};
use bevy_tweening::TweeningPlugin;

use crate::assets_util::assets_dir;
use crate::game_ui_panel::{game_ui_panel_system, setup_egui_fonts, setup_ui_font};
use crate::grid_render::{setup_grid, setup_sim_world, setup_world_root, sync_selection_border};
use crate::render::overlays::{setup_fx_layer, sync_ghost_preview, sync_hover_ring, sync_rain_overlay, sync_world_fx};
use crate::pathfinding::PathGrid;
use crate::event_registry::EventRegistry;
use crate::player::{PlayerBrainResource, PlayerPlugin};
use crate::sim_clock::{advance_sim_ticks, SimClock};
use crate::sim_events::{
    drain_sim_events, MoveAnimEvent, MoveAnimPlayback, MoveAnimationsComplete, SimEventQueue,
    WorldFxQueue,
};
use crate::interaction::InteractionState;
use crate::ui::minimap::minimap_panel_system;
use crate::ui_interaction::{
    handle_pointer_input, handle_view_pan, handle_view_zoom, update_drag_follow, CameraPanState,
    DragState, GhostPlaceMode, SelectionState,
};
use crate::viewport_layout::{
    bootstrap_viewport_layout, setup_cameras, sync_viewport_layout, ViewportLayout,
};
use crate::world_view::{sync_world_root_transform, WorldView};

pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "方寸商国·桃花源记".into(),
                        resolution: (1280.0, 720.0).into(),
                        resizable: true,
                        resize_constraints: WindowResizeConstraints {
                            min_width: 864.0,
                            min_height: 576.0,
                            ..default()
                        },
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    file_path: assets_dir().to_string_lossy().into(),
                    ..default()
                }),
        )
        .add_plugins(EguiPlugin)
        .add_plugins(TweeningPlugin)
        .add_plugins((
            SimPlugin,
            PlayerPlugin,
            RenderPlugin,
            InputPlugin,
            UiPlugin,
        ));
    }
}

pub struct SimPlugin;

impl Plugin for SimPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SimClock>()
            .init_resource::<SimEventQueue>()
            .init_resource::<WorldFxQueue>()
            .init_resource::<MoveAnimPlayback>()
            .add_event::<MoveAnimEvent>()
            .add_event::<MoveAnimationsComplete>()
            .init_resource::<InteractionState>()
            .init_resource::<PathGrid>()
            .init_resource::<EventRegistry>()
            .init_resource::<PlayerBrainResource>()
            .init_resource::<crate::session_report::TickStats>()
            .init_resource::<crate::session_report::SessionWallClock>()
            .add_systems(Startup, setup_sim_world)
            .add_systems(Update, (advance_sim_ticks, drain_sim_events))
            .add_systems(
                Update,
                crate::session_report::write_session_report_on_exit,
            );
    }
}

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<crate::render::terrain_view::TerrainVisualRevision>()
            .add_systems(
            Startup,
            (
                setup_world_root,
                setup_ui_font,
                setup_cameras,
                setup_grid,
                setup_fx_layer,
            )
                .chain(),
        )
        .add_systems(
            Update,
            (
                crate::render::move_animation::process_move_queue,
                sync_world_root_transform,
                crate::card_visual::sync_card_visuals,
                crate::render::terrain_view::sync_terrain_visuals,
                crate::render::card_view::sync_card_overlays,
                sync_selection_border,
                sync_hover_ring,
                sync_ghost_preview,
                sync_rain_overlay,
                sync_world_fx,
                crate::render::move_animation::on_move_anim_completed,
            )
                .chain(),
        );
    }
}

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldView>()
            .init_resource::<SelectionState>()
            .init_resource::<DragState>()
            .init_resource::<GhostPlaceMode>()
            .init_resource::<CameraPanState>()
            .add_systems(
                Update,
                (
                    handle_pointer_input,
                    update_drag_follow,
                    crate::ui_interaction::update_ghost_follow,
                    handle_view_zoom,
                    handle_view_pan,
                ),
            );
    }
}

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ViewportLayout>()
            .add_systems(Startup, (setup_egui_fonts, bootstrap_viewport_layout))
            .add_systems(
                Update,
                (
                    sync_viewport_layout,
                    game_ui_panel_system.after(EguiSet::InitContexts),
                    minimap_panel_system.after(EguiSet::InitContexts),
                ),
            );
    }
}
