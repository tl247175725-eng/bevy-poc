use bevy_poc::assets_util::ensure_ui_font;

fn install_crash_logger() {
    std::panic::set_hook(Box::new(|info| {
        let log_path = bevy_poc::assets_util::manifest_dir().join("bevy-poc-crash.log");
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
        {
            use std::io::Write;
            let _ = writeln!(file, "{info}");
        }
        eprintln!("{info}");
    }));
}

fn main() {
    install_crash_logger();

    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--bench") {
        bevy_poc::bench::run();
        return;
    }
    if args.iter().any(|a| a == "--smoke-test") {
        bevy_poc::smoke_test::run();
        return;
    }

    ensure_ui_font();

    bevy::prelude::App::new()
        .add_plugins(bevy_poc::plugins::AppPlugin)
        .run();
}
