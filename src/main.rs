use std::fs;

use anyhow::Result;
use clap::{Parser, Subcommand};

mod manifest;
use manifest::Manifest;
use reqwest::Client;
use serde::{Deserialize, Serialize};

/// Podzol - A modpack package manager
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new podzol project in the specified directory
    Init,
    /// Add a mod to the manifest
    Add { r#mod: String },
    /// Remove a mod from the manifest
    Remove,
    /// Exports the project
    Export,
}

#[derive(Debug, Deserialize, Serialize)]
struct Version {}

#[tokio::main]
async fn main() -> Result<()> {
    let Args { command } = Args::parse();

    let client = Client::new();

    match command {
        Commands::Add { r#mod } => {
            let manifest: Manifest = toml_edit::de::from_slice(&fs::read("podzol.toml")?)?;

            // dbg!(manifest);

            let res: Vec<Version> = client
                .get(format!("https://api.modrinth.com/v2/project/{mod}/version"))
                .query(&[
                    ("loaders", format!(r#"["{}"]"#, manifest.pack.loader)),
                    (
                        "game_versions",
                        format!(r#"["{}"]"#, manifest.pack.minecraft),
                    ),
                ])
                .send()
                .await?
                .json()
                .await?;

            println!("Adding..")
        }
        _ => todo!(),
    }

    Ok(())
}
