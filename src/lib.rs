pub mod assets_util;
pub mod axioms;
pub mod event_registry;
pub mod capabilities;
pub mod card_audit;
pub mod card_def;
pub mod card_style;
pub mod card_visual;
pub mod coords;
pub mod ecology_log;
pub mod game_constants;
pub mod game_ui_panel;
pub mod grid_render;
pub mod initial_spawn;
pub mod interaction;
pub mod panel_ui;
pub mod pathfinding;
pub mod player;
pub mod plugins;
pub mod render;
pub mod rule_index;
pub mod selection_info;
pub mod session_report;
pub mod sim_clock;
pub mod sim_events;
pub mod sim_observer;
pub mod spatial_index;
pub mod systems;
pub mod tag_zh;
pub mod terrain;
pub mod terrain_colors;
pub mod terrain_ecology;
pub mod ui;
pub mod ui_interaction;
pub mod viewport_layout;
pub mod visual_config;
pub mod world_rules;
pub mod world_state;
pub mod bench;
pub mod bulletin;
pub mod smoke_test;
pub mod world_view;

pub use capabilities::{all_capability_cards, card_capabilities, capability_count};
pub use card_audit::{
    audit_defs, cap_is_registered, card_color_valid, known_dimensions, load_and_audit,
    tag_dimension, tag_is_registered,
};
pub use card_def::{load_card_defs, load_card_defs_map, CardDef};
pub use spatial_index::{EntityId, SpatialIndex};
pub use terrain_colors::{rgba_to_f32, terrain_color, SELECTION_BORDER, cell_color_with_stress, river_stress_label};
pub use rule_index::{
    dual_track_eat, dual_track_graze, dual_track_hunt, rule_index, EcologyAction, RuleIndex,
};
pub use world_rules::*;
pub use game_constants::{
    PERISHABLE_TICKS, POPULATION_REPRO_CYCLE_SECONDS, PROLIFIC_LITTER_SIZE,
    PROLIFIC_REPRO_CYCLE_SECONDS,
};
pub use game_ui_panel::{
    game_ui_panel_system, panel_content_for_test, setup_egui_fonts, setup_ui_font, UiFont,
};
pub use card_visual::{slide_cards, stack_indices, sync_card_visuals};
pub use systems::main_tick::{
    flush_herbivore_tick, flush_reactive_entity_tick, mark_baseline_herbivore_tick,
};
pub use initial_spawn::{initial_card_count, spawn_initial_world};
pub use visual_config::{
    world_height, world_width, CELL_SIZE, PANEL_MIN_WIDTH, PANEL_WIDTH, panel_width_for,
};
pub use world_view::WorldView;
pub use world_state::{demo_world, drain_pending_events, empty_world, EcologyState, Entity, MoveResult, WorldState};
pub use systems::tick_harvest::harvest_at;
pub use systems::tick_environment;
pub use systems::tick_containment::{entities_in_pool, entities_in_tree, entities_underground};

pub use ecology_log::{card_display_name, eco_log};
pub use coords::{
    card_world_pos, cell_center, cursor_to_world, grid_from_cursor, grid_round_trip,
    grid_to_world, grid_to_world_in, world_to_grid, zoom_anchor_invariant, CoordinateSystem,
};
pub use selection_info::{
    build_card_panel, build_cell_panel, build_panel, build_panel_with_stress, entity_state_label,
    panel_text_joined, resolve_selection_card, ui_containment_entries, ContainmentEntry,
    PanelContent, SelectionTarget,
};
pub use tag_zh::{cap_has_zh_mapping, cap_zh, contains_english_tag, tag_has_zh_mapping, tag_zh};
pub use session_report::{session_report_path, SessionReport, TickStats};
pub use terrain::{
    base_terrain_at, cell_elevation, ecology, elevation_visual_offset_y, is_blocked_terrain,
    surface_label, surface_label_with_stress, terrain_at, terrain_label,
};
pub use terrain_colors::cell_color;
pub use terrain_ecology::{MapEcology, ELEV_DARK_RIVER};
pub use ui_interaction::{
    apply_camera_zoom, can_drag_entity, handle_selection_click, select_containment_entry,
    try_place_entity, CameraPanState, DragState, GhostPlaceMode, PlaceResult, SelectionState,
};
pub use viewport_layout::{setup_cameras, ViewportLayout};
pub use world_view::{sync_world_root_transform, WorldRoot, WorldRootEntity};
pub use pathfinding::{find_path, is_blocked_for, PathGrid};
pub use player::{
    compute_affordances, evaluate_needs, find_player_id, plan_craft_knife, select_intention,
    tick_brain, tick_player_world, PlayerMind, PlayerPlugin, TaskPhase,
};
pub use event_registry::EventRegistry;
pub use interaction::{
    apply_hunt_smash, apply_smash_hit, try_ghost_drop, try_harvest, try_impact, try_relation,
    InteractionState, RecipeBook, SmashOutcome,
};
pub use sim_events::{SimEvent, SimEventQueue, WorldFxQueue};
