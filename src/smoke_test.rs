//! Headless smoke test — `cargo run --release -- --smoke-test`
//! Runs 1000 ticks, checks health criteria, outputs PASS/FAIL.

use crate::game_constants::TICK_SECONDS;
use crate::initial_spawn::spawn_initial_world;
use crate::systems::main_tick::main_tick;
use crate::world_rules::{GRID_HEIGHT, GRID_WIDTH};
use std::time::Instant;

pub fn run() {
    println!("SMOKE: starting 1000-tick headless run...");
    let start = Instant::now();
    let mut world = spawn_initial_world();
    world.set_causal_mode(true);
    let initial_count = world.entities.len();
    let tick_delta = TICK_SECONDS;

    let mut failures: Vec<String> = Vec::new();
    let mut moved_herbivores = 0u32;
    let mut max_tick_ms = 0f64;

    for _ in 0..1000 {
        let tick_start = Instant::now();
        main_tick(&mut world, tick_delta);
        world.drain_pending_events();
        let tick_ms = tick_start.elapsed().as_secs_f64() * 1000.0;
        if tick_ms > max_tick_ms {
            max_tick_ms = tick_ms;
        }
        // Count herbivores that moved this tick (needs_grazing_tick was cleared = they were ticked)
        for e in world.entities.values() {
            if !e.needs_grazing_tick && !e.is_corpse {
                let Some(def) = world.card_defs.get(&e.type_name) else { continue; };
                if crate::world_rules::card_has_tag(def, "herbivore")
                    || crate::world_rules::card_has_tag(def, "omnivore.small")
                {
                    moved_herbivores += 1;
                    break; // count once per tick
                }
            }
        }
    }

    let elapsed = start.elapsed();
    let final_count = world.entities.len();

    // === health checks ===

    // 1. Entity count in range
    if final_count < 30 {
        failures.push(format!("entity count {} too low (ecosystem collapsed)", final_count));
    }
    if final_count > 900 {
        failures.push(format!("entity count {} too high (ecosystem exploded)", final_count));
    }

    // 2. Herbivore baseline tick working
    if moved_herbivores < 100 {
        failures.push(format!("herbivore tick triggered only {} ticks (need >=100/1000)", moved_herbivores));
    }

    // 3. No out-of-bounds cards
    let mut oob = 0;
    for e in world.entities.values() {
        if e.x >= GRID_WIDTH || e.y >= GRID_HEIGHT { oob += 1; }
    }
    if oob > 0 {
        failures.push(format!("{} entities out of bounds", oob));
    }

    // 4. Tick performance
    if max_tick_ms > 5.0 {
        failures.push(format!("max tick {:.2}ms > 5ms threshold", max_tick_ms));
    }

    // 5. Predators alive
    let predators = world.entities.values()
        .filter(|e| !e.is_corpse)
        .filter(|e| world.card_defs.get(&e.type_name)
            .map(|d| crate::world_rules::card_has_tag(d, "predator") || crate::world_rules::card_has_tag(d, "mesopredator"))
            .unwrap_or(false))
        .count();
    if predators == 0 {
        failures.push("all predators dead".into());
    }

    // 6. Herbivores alive
    let herbivores = world.entities.values()
        .filter(|e| !e.is_corpse)
        .filter(|e| world.card_defs.get(&e.type_name)
            .map(|d| crate::world_rules::card_has_tag(d, "herbivore") || crate::world_rules::card_has_tag(d, "omnivore.small"))
            .unwrap_or(false))
        .count();
    if herbivores == 0 {
        failures.push("all herbivores dead".into());
    }

    // 7. NaN check
    for e in world.entities.values() {
        if e.x as f32 != e.x as f32 || e.y as f32 != e.y as f32 {
            failures.push(format!("NaN coordinate on entity {:?}", e.id));
        }
    }

    // 8. Event queue not leaking
    let pending = world.pending_events.len();
    if pending > 256 {
        failures.push(format!("event queue pending={}", pending));
    }

    // === report ===
    println!();
    if failures.is_empty() {
        println!("SMOKE: PASS");
    } else {
        println!("SMOKE: FAIL ({}) failures:", failures.len());
        for f in &failures {
            println!("  - {}", f);
        }
    }
    println!("  entities: {}→{} | predators: {} | herbivores: {} | max_tick: {:.2}ms | elapsed: {:.1}s",
        initial_count, final_count, predators, herbivores, max_tick_ms, elapsed.as_secs_f64());
}
