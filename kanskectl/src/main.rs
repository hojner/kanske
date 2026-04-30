use clap::{Parser, Subcommand};
use kanske_lib::{AppResult, wayland_interface::wayland_setup};

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
    let (state, _connection, _event_queue, _queue_handle) = wayland_setup()?;
    // dbg!(&state);

    for head in state.heads {
        print!("{}", head);
    }

    Ok(())
}
