use crate::world_state::demo_world;
use std::time::Instant;

pub fn run() {
    let mut world = demo_world();
    let duration = std::time::Duration::from_secs(30);
    let start = Instant::now();
    let mut ticks: u64 = 0;

    while start.elapsed() < duration {
        world.tick_once();
        ticks += 1;
    }

    let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;
    let avg_tick_ms = elapsed_ms / ticks as f64;
    println!("ticks={ticks} elapsed_ms={elapsed_ms:.2} avg_tick_ms={avg_tick_ms:.4}");
}
