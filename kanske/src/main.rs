use std::{path::PathBuf, process};

use kanske_lib::{
    AppResult, KanskeState,
    applier::find_and_apply_profile,
    parser::{ast::Config, config_parser::parse_file},
};
use wayland_client::{Connection, EventQueue, QueueHandle};

const CONFIG_FILE: &str = "./test.txt";

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn run() -> AppResult<()> {
    let config = parse_file(PathBuf::from(CONFIG_FILE))?;
    let (mut state, mut event_queue, queue_handle) = wayland_setup()?;

    println!("Monitoring for display changes...");

    event_loop(&mut state, &mut event_queue, &queue_handle, &config)
}

fn wayland_setup() -> AppResult<(
    KanskeState,
    EventQueue<KanskeState>,
    QueueHandle<KanskeState>,
)> {
    let wayland_connection = Connection::connect_to_env()?;
    let mut event_queue = wayland_connection.new_event_queue();
    let queue_handle: QueueHandle<KanskeState> = event_queue.handle();

    let mut state = KanskeState {
        manager: None,
        heads: Vec::new(),
        serial: None,
    };
    wayland_connection.display().get_registry(&queue_handle, ());

    // Decision: Making two roundtrips will give us the initial state of
    // the app. It's not pretty but gets the job done, and seems to be the
    // prefered option for many production apps. For now it's good
    // enough, might be handled in a while !ready loop later.
    event_queue.roundtrip(&mut state)?;
    event_queue.roundtrip(&mut state)?;

    Ok((state, event_queue, queue_handle))
}

fn event_loop(
    state: &mut KanskeState,
    event_queue: &mut EventQueue<KanskeState>,
    queue_handle: &QueueHandle<KanskeState>,
    config: &Config,
) -> AppResult<()> {
    let mut last_serial = state.serial;

    loop {
        event_queue.blocking_dispatch(state)?;
        if state.serial != last_serial {
            println!("\n=== Display hotplug detected! ===");
            println!("Previous serial: {:?}", last_serial);
            println!("New serial: {:?}", state.serial);
            println!("Number of heads: {}", state.heads.len());
            last_serial = state.serial;
            find_and_apply_profile(state, queue_handle, config)?;
        }
    }
}

fn _print_heads(state: &KanskeState) {
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
