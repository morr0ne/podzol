use std::fs;

use anyhow::Result;
use async_zip::{base::write::ZipFileWriter, Compression, ZipEntryBuilder};
use clap::{Parser, Subcommand};
use tokio::fs::File;

mod manifest;
mod modrinth;
mod mrpack;

use manifest::Manifest;
use modrinth::Client;

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

#[tokio::main]
async fn main() -> Result<()> {
    let Args { command } = Args::parse();

    let client = Client::new()?;

    match command {
        Commands::Add { r#mod } => {
            let manifest: Manifest = toml_edit::de::from_slice(&fs::read("podzol.toml")?)?;

            let version = client
                .get_project_versions(
                    &r#mod,
                    &manifest.enviroment.minecraft,
                    &manifest.enviroment.loaders,
                )
                .await?;

            dbg!(version);

            println!("Adding..")
        }
        Commands::Export => {
            let manifest: Manifest = toml_edit::de::from_slice(&fs::read("podzol.toml")?)?;

            let metadata = manifest.into_metadata(&client).await?;

            let mut writer = ZipFileWriter::with_tokio(File::create("pack.mrpack").await?);

            let data = serde_json::to_vec(&metadata)?;
            let entry = ZipEntryBuilder::new(
                "modrinth.index.json".to_string().into(),
                Compression::Deflate,
            );

            writer.write_entry_whole(entry, &data).await?;

            writer.close().await?;
        }
        _ => todo!(),
    }

    Ok(())
}
