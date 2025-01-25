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
use toml_edit::{DocumentMut, InlineTable};

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
    Add { name: String },
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
        Commands::Add { name } => {
            let manifest_src = fs::read_to_string("podzol.toml")?;
            let mut document: DocumentMut = manifest_src.parse()?;
            let manifest: Manifest = toml_edit::de::from_document(document.clone())?;

            let versions = client
                .get_project_versions(
                    &name,
                    &manifest.enviroment.minecraft,
                    &manifest.enviroment.loaders,
                )
                .await?;

            // FIXME: use a proper strategy to choose
            let version = &versions[0];
            let version_number = &version.version_number;

            let mut mod_table = InlineTable::new();
            mod_table.insert("version", version_number.into());
            mod_table.insert("side", "both".into());
            document["mods"][&name] = mod_table.into();

            fs::write("podzol.toml", document.to_string())?;

            println!("Added {name} {version_number} to mods");
        }
        Commands::Export => {
            let manifest: Manifest = toml_edit::de::from_slice(&fs::read("podzol.toml")?)?;

            let mut writer = ZipFileWriter::with_tokio(
                File::create(format!(
                    "{}-{}.mrpack",
                    manifest.pack.name, manifest.pack.version
                ))
                .await?,
            );

            let metadata = manifest.into_metadata(&client).await?;

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
