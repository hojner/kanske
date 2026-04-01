use std::{collections::HashSet, path::PathBuf, process};

use kanske_lib::{
    AppResult, KanskeState,
    error::KanskeError,
    matcher::find_matching_profile,
    parser::{
        ast::{Config, OutputCommand, OutputDesc},
        config_parser::parse_file,
    },
    wayland_interface::HeadInfo,
};
use wayland_client::{Connection, QueueHandle, protocol::wl_output};
use wayland_protocols_wlr::output_management::v1::client::{
    zwlr_output_configuration_head_v1::ZwlrOutputConfigurationHeadV1,
    zwlr_output_head_v1::AdaptiveSyncState,
};

const CONFIG_FILE: &str = "./test.txt";

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn config_parse() -> AppResult<Config> {
    let config = parse_file(PathBuf::from(CONFIG_FILE))?;
    Ok(config)
}

fn run() -> AppResult<()> {
    let config = config_parse()?;

    let wayland_connection = Connection::connect_to_env()?;
    let display = wayland_connection.display();
    let mut event_queue = wayland_connection.new_event_queue();
    let queue_handle: QueueHandle<KanskeState> = event_queue.handle();

    let _registry = display.get_registry(&queue_handle, ());

    let mut state = KanskeState {
        manager: None,
        heads: Vec::new(),
        serial: None,
    };

    println!("Fetching initial monitor information...");

    // Decision: Making two roundtrips will give us the initial state of
    // the app. It's not pretty but gets the job done, and seems to be the
    // prefered option for many production apps. For now it's good
    // enough, might be handled in a while !ready loop later.
    event_queue.roundtrip(&mut state)?;
    event_queue.roundtrip(&mut state)?;

    find_and_apply_profile(&mut state, &queue_handle, &config)?;

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

fn find_and_apply_profile(
    state: &mut KanskeState,
    qh: &QueueHandle<KanskeState>,
    config: &Config,
) -> AppResult<()> {
    if let Some(profile) = find_matching_profile(&state.heads, config) {
        let mut used_indicies: HashSet<usize> = HashSet::new();
        let manager = state
            .manager
            .as_ref()
            .ok_or(KanskeError::ManagerNotAvailable)?;
        let serial = state.serial.ok_or(KanskeError::NoConfiguration)?;
        let output_configuration = manager.create_configuration(serial, &qh, ());

        for output in profile
            .outputs
            .iter()
            .filter(|f| matches!(f.desc, OutputDesc::Name(_)))
        {
            let position = state
                .heads
                .iter()
                .enumerate()
                .find(|(i, h)| !used_indicies.contains(i) && output.desc.matches(&h.name))
                .map(|(i, _)| i)
                .ok_or(KanskeError::NoConfiguration)?;
            used_indicies.insert(position);
            let current_head = &state.heads[position];
            let head_config = output_configuration.enable_head(&current_head.head, qh, ());
            for command in output.commands.iter() {
                apply_command(&head_config, &command, &current_head)?;
            }
        }
        for output in profile
            .outputs
            .iter()
            .filter(|f| matches!(f.desc, OutputDesc::Any))
        {
            let position = (0..state.heads.len())
                .find(|i| !used_indicies.contains(i))
                .ok_or(KanskeError::NoConfiguration)?;
            used_indicies.insert(position);
            let current_head = &state.heads[position];
            let head_config = output_configuration.enable_head(&current_head.head, qh, ());
            for command in output.commands.iter() {
                apply_command(&head_config, &command, &current_head)?;
            }
        }
    } else {
        return Ok(());
    }

    Ok(())
}
fn apply_command(
    head_config: &ZwlrOutputConfigurationHeadV1,
    command: &OutputCommand,
    current_head: &HeadInfo,
) -> AppResult<()> {
    match command {
        OutputCommand::Mode {
            width,
            height,
            frequency,
        } => {
            let mode_info = current_head
                .modes
                .iter()
                .find(|m| {
                    m.width == *width as i32
                        && m.height == *height as i32
                        && if let Some(f) = frequency {
                            m.refresh == (f * 1000.0) as i32
                        } else {
                            todo!();
                        }
                })
                .ok_or(KanskeError::NoConfiguration)?;

            head_config.set_mode(&mode_info.mode);
        }
        OutputCommand::Position { x, y } => {
            head_config.set_position(*x, *y);
        }
        OutputCommand::Scale(scale) => {
            head_config.set_scale(*scale as f64);
        }
        OutputCommand::Transform(transform) => {
            head_config.set_transform(wl_output::Transform::from(*transform))
        }
        OutputCommand::AdaptiveSync(b) => match b {
            true => {
                head_config.set_adaptive_sync(AdaptiveSyncState::Enabled);
            }
            false => {
                head_config.set_adaptive_sync(AdaptiveSyncState::Disabled);
            }
        },
        _ => {}
    }
    Ok(())
}
