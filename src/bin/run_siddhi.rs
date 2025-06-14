use std::env;
use std::fs;
use std::thread;
use std::time::Duration;

use siddhi_rust::core::siddhi_manager::SiddhiManager;
use siddhi_rust::core::stream::output::sink::LogSink;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <siddhi_file>", args[0]);
        std::process::exit(1);
    }

    let file = &args[1];
    let content = match fs::read_to_string(file) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to read {}: {}", file, e);
            std::process::exit(1);
        }
    };

    let manager = SiddhiManager::new();
    let runtime = match manager.create_siddhi_app_runtime_from_string(&content) {
        Ok(rt) => rt,
        Err(e) => {
            eprintln!("Failed to create runtime: {}", e);
            std::process::exit(1);
        }
    };

    for stream_id in runtime.stream_junction_map.keys() {
        let _ = runtime.add_callback(stream_id, Box::new(LogSink::new()));
    }

    println!(
        "Running Siddhi app '{}'. Press Ctrl+C to exit",
        runtime.name
    );

    // Keep the main thread alive until interrupted.
    loop {
        thread::sleep(Duration::from_secs(1));
    }
}
