use bevy_poc::demo_world;
fn main() {
    let mut w = demo_world();
    w.run_ticks(1000);
    eprintln!("entities={}", w.entities.len());
}
