use clap::{Parser, Subcommand};

/// Podzol - A modpack manager
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init,
    Add,
    Remove,
}

fn main() {
    let Args { command } = Args::parse();

    match command {
        _ => todo!(),
    }
}
