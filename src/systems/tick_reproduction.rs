use crate::game_constants::{
    DEER_POP_CAP, DEER_REPRODUCE_MIN_GRASS, DEER_REPRODUCE_WOLF_CLEAR_RADIUS, FIELD_MOUSE_POP_CAP,
    FIELD_MOUSE_REPRODUCE_MIN_MICRO, POPULATION_REPRO_CYCLE_SECONDS, PROLIFIC_LITTER_SIZE,
    PROLIFIC_REPRO_CYCLE_SECONDS, RABBIT_POP_CAP, WOLF_DEN_CAPACITY,
};
use crate::world_rules::wolves_near;
use crate::world_state::WorldState;

pub fn tick_reproduction(world: &mut WorldState, delta: f32) {
    world.repro_timer += delta;
    world.rabbit_repro_timer += delta;

    if world.repro_timer >= POPULATION_REPRO_CYCLE_SECONDS {
        world.repro_timer = 0.0;
        try_reproduce_sheep(world);
        try_reproduce_deer(world);
        try_reproduce_wolf(world);
        try_reproduce_fox(world);
        try_reproduce_pheasant(world);
        try_reproduce_field_mouse(world);
        try_reproduce_bamboo_rat(world);
        try_reproduce_water_buffalo(world);
    }
    if world.rabbit_repro_timer >= PROLIFIC_REPRO_CYCLE_SECONDS {
        world.rabbit_repro_timer = 0.0;
        try_reproduce_rabbit(world);
    }
}

fn try_reproduce_sheep(world: &mut WorldState) {
    if world.count_type("sheep") < crate::world_rules::FLOCKING_REPRO_MIN as usize {
        return;
    }
    if world.count_type("lamb") > 0 {
        return;
    }
    if let Some(parent) = world.entities.values().find(|e| e.type_name == "sheep") {
        let nx = (parent.x + 1).min(crate::world_rules::GRID_WIDTH - 1);
        world.spawn("lamb", nx, parent.y);
    }
}

fn try_reproduce_deer(world: &mut WorldState) {
    if world.count_type("deer") >= DEER_POP_CAP as usize {
        return;
    }
    if world.grass_count() < DEER_REPRODUCE_MIN_GRASS as usize {
        return;
    }
    if let Some(deer) = world.entities.values().find(|e| e.type_name == "deer") {
        if !wolves_near(world, deer.x, deer.y, DEER_REPRODUCE_WOLF_CLEAR_RADIUS).is_empty() {
            return;
        }
        world.spawn("deerFawn", deer.x, deer.y);
    }
}

fn try_reproduce_wolf(world: &mut WorldState) {
    if world.count_type("wolfDen") == 0 {
        return;
    }
    if world.count_type("wolfCub") > 0 {
        return;
    }
    if world.wolf_count() >= WOLF_DEN_CAPACITY as usize {
        return;
    }
    if let Some(wolf) = world.entities.values().find(|e| e.type_name == "wolf") {
        world.spawn("wolfCub", wolf.x, wolf.y);
    }
}

fn try_reproduce_fox(world: &mut WorldState) {
    if world.count_type("foxDen") == 0 || world.count_type("foxCub") > 0 {
        return;
    }
    if let Some(fox) = world.entities.values().find(|e| e.type_name == "fox") {
        world.spawn("foxCub", fox.x, fox.y);
    }
}

fn try_reproduce_rabbit(world: &mut WorldState) {
    if world.count_type("rabbit") >= RABBIT_POP_CAP as usize {
        return;
    }
    if let Some((rx, ry)) = world
        .entities
        .values()
        .find(|e| e.type_name == "rabbit")
        .map(|e| (e.x, e.y))
    {
        for _ in 0..PROLIFIC_LITTER_SIZE {
            world.spawn("rabbit", rx, ry);
        }
    }
}

fn try_reproduce_pheasant(world: &mut WorldState) {
    if world.count_type("pheasant") < crate::world_rules::FLOCKING_REPRO_MIN as usize {
        return;
    }
    if let Some(p) = world.entities.values().find(|e| e.type_name == "pheasant") {
        world.spawn("pheasantChick", p.x, p.y);
    }
}

fn try_reproduce_field_mouse(world: &mut WorldState) {
    if world.count_type("fieldMouse") >= FIELD_MOUSE_POP_CAP as usize {
        return;
    }
    let micro_ok = world.bush_microfauna.values().any(|&m| m >= FIELD_MOUSE_REPRODUCE_MIN_MICRO);
    if !micro_ok {
        return;
    }
    if let Some(mouse) = world.entities.values().find(|e| e.type_name == "fieldMouse") {
        world.spawn("fieldMousePup", mouse.x, mouse.y);
    }
}

fn try_reproduce_bamboo_rat(world: &mut WorldState) {
    if world.count_type("bambooRat") >= FIELD_MOUSE_POP_CAP as usize {
        return;
    }
    if world.count_type("tree") == 0 && world.count_type("oak") == 0 && world.count_type("pine") == 0
    {
        return;
    }
    if let Some(rat) = world.entities.values().find(|e| e.type_name == "bambooRat") {
        world.spawn("bambooRat", rat.x, rat.y);
    }
}

fn try_reproduce_water_buffalo(world: &mut WorldState) {
    if world.grass_count() < 3 {
        return;
    }
    if let Some(buf) = world
        .entities
        .values()
        .find(|e| e.type_name == "waterBuffalo")
    {
        world.spawn("waterBuffaloCalf", buf.x, buf.y);
    }
}

/// Legacy entry used by old tests.
pub fn tick_reproduction_legacy(world: &mut WorldState) {
    if crate::world_state::reproduction_allowed(world) {
        try_reproduce_sheep(world);
    }
}
