use clap::{Parser, Subcommand};
use kanske_lib::{
    AppResult,
    wayland_interface::{WaylandState, connect},
};

#[derive(Parser)]
#[command(name = "kanskectl", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List anything you want!
    List,
    /// Reload when needed.
    Reload,
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::List => {
            let _outputs = list_outputs();
        }
        Commands::Reload => println!("Load and reload!"),
    }
}

fn list_outputs() -> AppResult<()> {
    let (_connection, mut event_queue, _qh) = connect::<WaylandState>()?;
    let mut state = WaylandState {
        manager: None,
        heads: Vec::new(),
        serial: None,
    };
    event_queue.roundtrip(&mut state)?;
    event_queue.roundtrip(&mut state)?;

    for head in state.heads {
        print!("{}", head);
    }

    Ok(())
}
