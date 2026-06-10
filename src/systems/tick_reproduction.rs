use crate::game_constants::{
    FIELD_MOUSE_REPRODUCE_MIN_MICRO, POPULATION_REPRO_CYCLE_SECONDS, PROLIFIC_REPRO_CYCLE_SECONDS,
};
use crate::world_state::WorldState;

#[derive(Clone, Debug)]
struct ReproParams {
    offspring: String,
    prolific: bool,
    pop_cap: Option<usize>,
    min_pop: Option<usize>,
    litter: usize,
    require_grass: Option<usize>,
    predator_clear_radius: Option<u8>,
    require_den_type: Option<String>,
    block_offspring: Option<String>,
    require_microfauna: bool,
    require_tree: bool,
    spawn_adjacent: bool,
}

fn parse_repro_params(tags: &[String]) -> Option<ReproParams> {
    let offspring = tags
        .iter()
        .find_map(|t| t.strip_prefix("repro_offspring:"))
        .map(str::to_string)?;
    let mut params = ReproParams {
        offspring,
        prolific: tags.iter().any(|t| t == "repro_prolific"),
        pop_cap: None,
        min_pop: None,
        litter: 1,
        require_grass: None,
        predator_clear_radius: None,
        require_den_type: None,
        block_offspring: None,
        require_microfauna: false,
        require_tree: false,
        spawn_adjacent: false,
    };
    for tag in tags {
        if let Some(v) = tag.strip_prefix("repro_pop_cap:") {
            params.pop_cap = v.parse().ok();
        } else if let Some(v) = tag.strip_prefix("repro_min_pop:") {
            params.min_pop = v.parse().ok();
        } else if let Some(v) = tag.strip_prefix("repro_litter:") {
            params.litter = v.parse().unwrap_or(1).max(1);
        } else if let Some(v) = tag.strip_prefix("repro_require_grass:") {
            params.require_grass = v.parse().ok();
        } else if let Some(v) = tag.strip_prefix("repro_predator_clear:") {
            params.predator_clear_radius = v.parse().ok();
        } else if let Some(v) = tag.strip_prefix("repro_require_den:") {
            params.require_den_type = Some(v.to_string());
        } else if let Some(v) = tag.strip_prefix("repro_block_offspring:") {
            params.block_offspring = Some(v.to_string());
        } else if tag == "repro_require_microfauna" {
            params.require_microfauna = true;
        } else if tag == "repro_require_tree" {
            params.require_tree = true;
        } else if tag == "repro_spawn:adjacent" {
            params.spawn_adjacent = true;
        }
    }
    Some(params)
}

pub fn tick_reproduction(world: &mut WorldState, delta: f32) {
    world.repro_timer += delta;
    world.rabbit_repro_timer += delta;

    if world.repro_timer >= POPULATION_REPRO_CYCLE_SECONDS {
        world.repro_timer = 0.0;
        let parents: Vec<(String, ReproParams)> = world
            .card_defs
            .values()
            .filter_map(|def| {
                let params = parse_repro_params(&def.tags)?;
                (!params.prolific).then_some((def.type_name.clone(), params))
            })
            .collect();
        for (parent_type, params) in parents {
            try_reproduce(world, &parent_type, &params);
        }
    }
    if world.rabbit_repro_timer >= PROLIFIC_REPRO_CYCLE_SECONDS {
        world.rabbit_repro_timer = 0.0;
        let parents: Vec<(String, ReproParams)> = world
            .card_defs
            .values()
            .filter_map(|def| {
                let params = parse_repro_params(&def.tags)?;
                params.prolific.then_some((def.type_name.clone(), params))
            })
            .collect();
        for (parent_type, params) in parents {
            try_reproduce(world, &parent_type, &params);
        }
    }
}

fn try_reproduce(world: &mut WorldState, parent_type: &str, params: &ReproParams) {
    if let Some(min_pop) = params.min_pop {
        if world.count_type(parent_type) < min_pop {
            return;
        }
    }

    if let Some(cap) = params.pop_cap {
        if world.count_type(parent_type) >= cap {
            return;
        }
    }

    if let Some(ref block) = params.block_offspring {
        if world.count_type(block) > 0 {
            return;
        }
    }

    if let Some(grass_needed) = params.require_grass {
        if world.count_by_tag("grass") < grass_needed {
            return;
        }
    }

    if params.require_tree
        && world.count_type("tree") == 0
        && world.count_type("oak") == 0
        && world.count_type("pine") == 0
    {
        return;
    }

    if params.require_microfauna
        && !world
            .bush_microfauna
            .values()
            .any(|&m| m >= FIELD_MOUSE_REPRODUCE_MIN_MICRO)
    {
        return;
    }

    if let Some(ref den_type) = params.require_den_type {
        if world.count_type(den_type) == 0 {
            return;
        }
    }

    let Some(parent) = world
        .entities
        .values()
        .find(|e| e.type_name == parent_type && !e.is_corpse)
    else {
        return;
    };

    if let Some(radius) = params.predator_clear_radius {
        if !world
            .query_near_filtered(parent.x, parent.y, "predator", radius, parent.id)
            .is_empty()
        {
            return;
        }
    }

    let (px, py) = (parent.x, parent.y);
    let (sx, sy) = if params.spawn_adjacent {
        ((px + 1).min(crate::world_rules::GRID_WIDTH - 1), py)
    } else {
        (px, py)
    };
    for _ in 0..params.litter {
        world.spawn(&params.offspring, sx, sy);
    }
}

/// Legacy entry used by old tests.
pub fn tick_reproduction_legacy(world: &mut WorldState) {
    if crate::world_state::reproduction_allowed(world) {
        if let Some(def) = world.card_defs.get("sheep").cloned() {
            if let Some(params) = parse_repro_params(&def.tags) {
                try_reproduce(world, "sheep", &params);
            }
        }
    }
}
