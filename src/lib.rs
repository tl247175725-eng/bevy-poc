// 核心模块——始终编译
pub mod weather;
pub mod weather_grid;
pub mod meta_values;

// 旧渲染/UI层——仅 legacy feature
#[cfg(feature = "legacy")]
pub mod assets_util;
#[cfg(feature = "legacy")]
pub mod axioms;
#[cfg(feature = "legacy")]
pub mod event_registry;
#[cfg(feature = "legacy")]
pub mod capabilities;
#[cfg(feature = "legacy")]
pub mod card_audit;
#[cfg(feature = "legacy")]
pub mod card_def;
#[cfg(feature = "legacy")]
pub mod card_style;
#[cfg(feature = "legacy")]
pub mod card_visual;
#[cfg(feature = "legacy")]
pub mod coords;
#[cfg(feature = "legacy")]
pub mod ecology_log;
#[cfg(feature = "legacy")]
pub mod game_constants;
#[cfg(feature = "legacy")]
pub mod hand_cards;
#[cfg(feature = "legacy")]
pub mod game_ui_panel;
#[cfg(feature = "legacy")]
pub mod grid_render;
#[cfg(feature = "legacy")]
pub mod initial_spawn;
#[cfg(feature = "legacy")]
pub mod interaction;
#[cfg(feature = "legacy")]
pub mod meta_actions;
#[cfg(feature = "legacy")]
pub mod memory;
#[cfg(feature = "legacy")]
pub mod need_match;
#[cfg(feature = "legacy")]
pub mod panel_ui;
#[cfg(feature = "legacy")]
pub mod pathfinding;
#[cfg(feature = "legacy")]
pub mod perception;
#[cfg(feature = "legacy")]
pub mod player;
#[cfg(feature = "legacy")]
pub mod plugins;
#[cfg(feature = "legacy")]
pub mod render;
#[cfg(feature = "legacy")]
pub mod rule_index;
#[cfg(feature = "legacy")]
pub mod selection_info;
#[cfg(feature = "legacy")]
pub mod session_report;
#[cfg(feature = "legacy")]
pub mod sim_clock;
#[cfg(feature = "legacy")]
pub mod sim_events;
#[cfg(feature = "legacy")]
pub mod sim_observer;
#[cfg(feature = "legacy")]
pub mod spatial_index;
#[cfg(feature = "legacy")]
pub mod systems;
#[cfg(feature = "legacy")]
pub mod tags;
#[cfg(feature = "legacy")]
pub mod tag_zh;
#[cfg(feature = "legacy")]
pub mod terrain;
#[cfg(feature = "legacy")]
pub mod terrain_colors;
#[cfg(feature = "legacy")]
pub mod terrain_ecology;
#[cfg(feature = "legacy")]
pub mod ui;
#[cfg(feature = "legacy")]
pub mod ui_interaction;
#[cfg(feature = "legacy")]
pub mod viewport_layout;
#[cfg(feature = "legacy")]
pub mod visual_config;
#[cfg(feature = "legacy")]
pub mod world_rules;
#[cfg(feature = "legacy")]
pub mod world_state;
#[cfg(feature = "legacy")]
pub mod bench;
#[cfg(feature = "legacy")]
pub mod bulletin;
#[cfg(feature = "legacy")]
pub mod smoke_test;
#[cfg(feature = "legacy")]
pub mod world_view;

#[cfg(feature = "legacy")]
pub use capabilities::{all_capability_cards, card_capabilities, capability_count};
#[cfg(feature = "legacy")]
pub use card_audit::{
    audit_defs, cap_is_registered, card_color_valid, known_dimensions, load_and_audit,
    tag_dimension, tag_is_registered,
};
#[cfg(feature = "legacy")]
pub use card_def::{load_card_defs, load_card_defs_map, CardDef};
#[cfg(feature = "legacy")]
pub use spatial_index::{EntityId, SpatialIndex};
#[cfg(feature = "legacy")]
pub use terrain_colors::{rgba_to_f32, terrain_color, SELECTION_BORDER, cell_color_with_stress, river_stress_label};
#[cfg(feature = "legacy")]
pub use rule_index::{
    rule_index, EcologyAction, RuleIndex,
};
#[cfg(feature = "legacy")]
pub use world_rules::*;
#[cfg(feature = "legacy")]
pub use game_constants::{
    PERISHABLE_TICKS, POPULATION_REPRO_CYCLE_SECONDS, PROLIFIC_LITTER_SIZE,
    PROLIFIC_REPRO_CYCLE_SECONDS,
};
#[cfg(feature = "legacy")]
pub use game_ui_panel::{
    game_ui_panel_system, panel_content_for_test, setup_egui_fonts, setup_ui_font, UiFont,
};
#[cfg(feature = "legacy")]
pub use card_visual::{slide_cards, stack_indices, sync_card_visuals};
#[cfg(feature = "legacy")]
pub use systems::main_tick::{
    flush_herbivore_tick, flush_reactive_entity_tick, mark_baseline_herbivore_tick,
};
#[cfg(feature = "legacy")]
pub use initial_spawn::{initial_card_count, spawn_initial_world};
#[cfg(feature = "legacy")]
pub use visual_config::{
    world_height, world_width, CELL_SIZE, PANEL_MIN_WIDTH, PANEL_WIDTH, panel_width_for,
};
#[cfg(feature = "legacy")]
pub use world_view::WorldView;
#[cfg(feature = "legacy")]
pub use world_state::{demo_world, drain_pending_events, empty_world, EcologyState, Entity, MoveResult, WorldState};
#[cfg(feature = "legacy")]
pub use systems::tick_harvest::harvest_at;
#[cfg(feature = "legacy")]
pub use systems::tick_environment;
#[cfg(feature = "legacy")]
pub use systems::tick_containment::{entities_in_pool, entities_in_tree, entities_underground};

#[cfg(feature = "legacy")]
pub use ecology_log::{card_display_name, eco_log};
#[cfg(feature = "legacy")]
pub use coords::{
    card_world_pos, cell_center, cursor_to_world, grid_from_cursor, grid_round_trip,
    grid_to_world, grid_to_world_in, world_to_grid, zoom_anchor_invariant, CoordinateSystem,
};
#[cfg(feature = "legacy")]
pub use selection_info::{
    build_card_panel, build_cell_panel, build_panel, build_panel_with_stress, entity_state_label,
    panel_text_joined, resolve_selection_card, ui_containment_entries, ContainmentEntry,
    PanelContent, SelectionTarget,
};
#[cfg(feature = "legacy")]
pub use tag_zh::{cap_has_zh_mapping, cap_zh, contains_english_tag, tag_has_zh_mapping, tag_zh};
#[cfg(feature = "legacy")]
pub use session_report::{session_report_path, SessionReport, TickStats};
#[cfg(feature = "legacy")]
pub use terrain::{
    base_terrain_at, cell_elevation, ecology, elevation_visual_offset_y, is_blocked_terrain,
    surface_label, surface_label_with_stress, terrain_at, terrain_label,
};
#[cfg(feature = "legacy")]
pub use terrain_colors::cell_color;
#[cfg(feature = "legacy")]
pub use terrain_ecology::{MapEcology, ELEV_DARK_RIVER};
#[cfg(feature = "legacy")]
pub use ui_interaction::{
    apply_camera_zoom, can_drag_entity, handle_selection_click, select_containment_entry,
    try_place_entity, CameraPanState, DragState, GhostPlaceMode, PlaceResult, SelectionState,
};
#[cfg(feature = "legacy")]
pub use viewport_layout::{setup_cameras, ViewportLayout};
#[cfg(feature = "legacy")]
pub use world_view::{sync_world_root_transform, WorldRoot, WorldRootEntity};
#[cfg(feature = "legacy")]
pub use pathfinding::{find_path, is_blocked_for, PathGrid};
#[cfg(feature = "legacy")]
pub use player::{
    compute_affordances, evaluate_needs, find_player_id, plan_craft_knife, select_intention,
    tick_brain, tick_player_world, PlayerMind, PlayerPlugin, TaskPhase,
};
#[cfg(feature = "legacy")]
pub use event_registry::EventRegistry;
#[cfg(feature = "legacy")]
pub use interaction::{
    apply_hunt_smash, apply_smash_hit, try_ghost_drop, try_harvest, try_impact, try_relation,
    InteractionState, RecipeBook, SmashOutcome,
};
#[cfg(feature = "legacy")]
pub use sim_events::{SimEvent, SimEventQueue, WorldFxQueue};
