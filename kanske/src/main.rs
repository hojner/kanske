use std::{path::PathBuf, process};

use kanske_lib::{
    AppResult, AppState, KanskeError,
    parser::{ast::Config, config_parser::parse_file},
};
use wayland_client::Connection;

const CONFIG_FILE: &str = "./test.txt";

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

async fn config_parse() -> AppResult<Config> {
    let config = parse_file(PathBuf::from(CONFIG_FILE)).await?;
    Ok(config)
}

async fn run() -> AppResult<()> {
    if let Err(e) = config_parse().await {
        eprintln!("Config parse error: {}", e);
    }

    let conn = match Connection::connect_to_env() {
        Ok(c) => c,
        Err(e) => return Err(KanskeError::WaylandConnectError(e)),
    };
    let display = conn.display();
    let mut event_queue = conn.new_event_queue();
    let queue_handle = event_queue.handle();

    let _registry = display.get_registry(&queue_handle, ());

    let mut state = AppState {
        manager: None,
        heads: Vec::new(),
        serial: None,
    };

    println!("Fetching initial monitor information...");
    event_queue.roundtrip(&mut state)?;

    println!("Monitoring for display changes...");
    let mut last_serial = state.serial;

    loop {
        event_queue.blocking_dispatch(&mut state)?;

        if state.serial != last_serial {
            println!("\n=== Display hotplug detected! ===");
            println!("Previous serial: {:?}", last_serial);
            println!("New serial: {:?}", state.serial);
            println!("Number of heads: {}", state.heads.len());
            last_serial = state.serial;
        }
    }
}

fn _print_heads(state: &AppState) {
    println!("\n=== Monitors ===");
    println!("{}", state.heads.len());
    for (i, head) in state.heads.iter().enumerate() {
        println!("\nMonitor {}:", i);
        println!("  Name: {}", head.name);
        println!("  Description: {}", head.description);
        println!("  Enabled: {}", head.enabled);

        if let Some(mode) = &head.current_mode {
            println!(
                "  Current Mode: {}x{} @ {:.2}Hz",
                mode.width,
                mode.height,
                mode.refresh as f32 / 1000.0
            );
        }

        println!("  Available Modes:");
        for mode in &head.modes {
            println!(
                "    {}x{} @ {:.2}Hz",
                mode.width,
                mode.height,
                mode.refresh as f32 / 1000.0
            );
        }
    }
    println!();
}
