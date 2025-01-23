use std::{collections::HashMap, fs, sync::Arc};

use anyhow::Result;
use async_zip::{base::write::ZipFileWriter, Compression, ZipEntryBuilder};
use clap::{Parser, Subcommand};
use itertools::Itertools;
use tokio::fs::File;

use reqwest::Client;
use rustls::crypto::aws_lc_rs;
use rustls_platform_verifier::BuilderVerifierExt;

mod manifest;
mod modrinth;
mod mrpack;

use manifest::{Manifest, Mod};
use modrinth::Version;
use mrpack::{Game, Metadata};

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

    let client = Client::builder()
        .use_preconfigured_tls(
            rustls::ClientConfig::builder_with_provider(Arc::new(aws_lc_rs::default_provider()))
                .with_safe_default_protocol_versions()?
                .with_platform_verifier()
                .with_no_client_auth(),
        )
        .build()?;

    match command {
        Commands::Add { r#mod } => {
            let manifest: Manifest = toml_edit::de::from_slice(&fs::read("podzol.toml")?)?;

            let loaders = manifest
                .enviroment
                .loaders
                .iter()
                .format_with(",", |(loader, _), f| f(&format_args!("\"{loader}\"")));

            let _res: Vec<Version> = client
                .get(format!("https://api.modrinth.com/v2/project/{mod}/version"))
                .query(&[
                    ("loaders", format!("[{loaders}]")),
                    (
                        "game_versions",
                        format!(r#"["{}"]"#, manifest.enviroment.minecraft),
                    ),
                ])
                .send()
                .await?
                .json()
                .await?;

            println!("Adding..")
        }
        Commands::Export => {
            let manifest: Manifest = toml_edit::de::from_slice(&fs::read("podzol.toml")?)?;

            for (name, m) in manifest.mods {
                match m {
                    Mod::Version(version) => {
                        let res: Version = client
                            .get(format!(
                                "https://api.modrinth.com/v2/project/{name}/version/{version}"
                            ))
                            .send()
                            .await?
                            .json()
                            .await?;

                        dbg!(res);
                    }
                }
            }

            let dependencies: HashMap<String, String> = manifest
                .enviroment
                .loaders
                .into_iter()
                .map(|(loader, version)| (loader.as_mrpack().to_string(), version))
                .chain(std::iter::once((
                    "minecraft".to_string(),
                    manifest.enviroment.minecraft,
                )))
                .collect();

            let metadata = Metadata {
                format_version: 1,
                game: Game::Minecraft,
                version_id: manifest.pack.version,
                name: manifest.pack.name,
                summary: manifest.pack.description,
                files: vec![],
                dependencies,
            };

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
