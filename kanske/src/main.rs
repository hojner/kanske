use std::path::PathBuf;

use kanske_lib::{AppResult, AppState, parser::block_parser::parse_file};
// use wayland_client::Connection;

// struct OutputConfig {
//     profile: bool,
//     name: Arc<str>,
//     outputs: Arc<[Params]>,
// }

async fn config_parse() -> AppResult<()> {
    //     let test_config = "profile work-A {
    //     output DP-1 enable mode 3440x1440@60.00Hz position 0,0 scale 1.0
    //     output eDP-1 disable
    // }";
    // let test_str = /*DP-1 */"enable scale 1.0 mode 3440x1440@60.00Hz position 3,5";
    // println!("{:?}", test_str);
    let output = parse_file(PathBuf::from("./test.txt")).await?;
    dbg!(output);
    // for i in output.iter() {
    //     dbg!(i);
    // }
    Ok(())
}

// fn count_outputs(state: &AppState) -> AppResult<usize> {
//     return Ok(state.heads.len());
// }

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

#[tokio::main]
async fn main() -> AppResult<()> {
    config_parse().await?;
    // let conn = match Connection::connect_to_env() {
    //     Ok(c) => c,
    //     Err(e) => return Err(KanskeError::WaylandConnectError(e)),
    // };
    // let display = conn.display();
    // let mut event_queue = conn.new_event_queue();
    // let qh = event_queue.handle();

    // let _registry = display.get_registry(&qh, ());

    // let mut state = AppState {
    //     manager: None,
    //     heads: Vec::new(),
    //     serial: None,
    // };

    // println!("Fetching monitor information...");
    // event_queue.roundtrip(&mut state).unwrap();
    // event_queue.roundtrip(&mut state).unwrap();

    // print_heads(&state);

    Ok(())
}
