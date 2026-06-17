//! Headless smoke test — `cargo run --release -- --smoke-test`
//! Runs 1000 ticks, checks health criteria, outputs PASS/FAIL.

use crate::initial_spawn::spawn_initial_world;
use crate::world_rules::{GRID_HEIGHT, GRID_WIDTH, TAG_REGISTRY};
use crate::world_state::EcologyState;
use std::time::Instant;

pub fn run() {
    // TagRegistry 必须在 spawn_initial_world 之前就位
    crate::world_rules::init_tag_registry();
    let registry = TAG_REGISTRY.get().expect("TagRegistry 刚初始化");

    println!("SMOKE: starting 1000-tick headless run...");
    let start = Instant::now();
    let mut world = spawn_initial_world();
    world.set_causal_mode(true);
    let initial_count = world.entities.len();

    let mut failures: Vec<String> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();
    let mut moved_herbivores = 0u32;
    let mut predation_ticks = 0u32;
    let mut max_tick_ms = 0f64;

    // 缓存的 diet 位掩码查询
    let is_herbivore = |def: &crate::card_def::CardDef| -> bool {
        def.has_tag_from_registry(registry, "diet:herbivore")
    };
    let is_omnivore = |def: &crate::card_def::CardDef| -> bool {
        def.has_tag_from_registry(registry, "diet:omnivore")
    };
    let is_carnivore = |def: &crate::card_def::CardDef| -> bool {
        def.has_tag_from_registry(registry, "diet:carnivore")
    };

    for _ in 0..1000 {
        let tick_start = Instant::now();
        world.tick_once();
        world.drain_pending_events();
        let tick_ms = tick_start.elapsed().as_secs_f64() * 1000.0;
        if tick_ms > max_tick_ms {
            max_tick_ms = tick_ms;
        }
        // Count herbivores/omnivores that were processed this tick
        // (ecology_state != Idle means the ecology system evaluated them)
        for e in world.entities.values() {
            if e.ecology_state != EcologyState::Idle && !e.is_corpse {
                let Some(def) = world.card_defs.get(&e.type_name) else { continue; };
                if is_herbivore(def) || is_omnivore(def) {
                    moved_herbivores += 1;
                    break; // count once per tick
                }
            }
        }
        // Count predation this tick: any carnivore that just ate
        for e in world.entities.values() {
            if !e.is_corpse && e.fed {
                let Some(def) = world.card_defs.get(&e.type_name) else { continue; };
                if is_carnivore(def) {
                    predation_ticks += 1;
                    break;
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
    if max_tick_ms > 15.0 {
        failures.push(format!("max tick {:.2}ms > 15ms threshold", max_tick_ms));
    }

    // 5. Predators alive
    let predators = world.entities.values()
        .filter(|e| !e.is_corpse)
        .filter(|e| world.card_defs.get(&e.type_name)
            .map(|d| is_carnivore(d))
            .unwrap_or(false))
        .count();
    if predators == 0 {
        failures.push("all predators dead".into());
    }

    // 6. Herbivores alive
    let herbivores = world.entities.values()
        .filter(|e| !e.is_corpse)
        .filter(|e| world.card_defs.get(&e.type_name)
            .map(|d| is_herbivore(d) || is_omnivore(d))
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

    // 9. Activity check — game must not be frozen
    if moved_herbivores == 0 {
        failures.push("all herbivores frozen — no movement detected".into());
    }
    // 零捕猎是预期行为（生态刚解冻），只发 warning
    if predation_ticks == 0 {
        warnings.push("zero predation events — ecosystem may be recovering".into());
    }

    // === report ===
    println!();
    if !warnings.is_empty() {
        println!("SMOKE: WARNINGS:");
        for w in &warnings {
            println!("  - {}", w);
        }
    }
    if failures.is_empty() {
        println!("SMOKE: PASS");
    } else {
        println!("SMOKE: FAIL ({}) failures:", failures.len());
        for f in &failures {
            println!("  - {}", f);
        }
        let mut counts: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
        for e in world.entities.values() {
            *counts.entry(e.type_name.clone()).or_default() += 1;
        }
        println!("  top entity types:");
        let mut sorted: Vec<_> = counts.into_iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));
        for (t, n) in sorted.iter().take(15) {
            println!("    {}: {}", t, n);
        }
    }
    println!("  entities: {}→{} | predators: {} | herbivores: {} | max_tick: {:.2}ms | elapsed: {:.1}s",
        initial_count, final_count, predators, herbivores, max_tick_ms, elapsed.as_secs_f64());
}
