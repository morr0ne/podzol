use std::{collections::HashMap, fs, sync::Arc};

use anyhow::Result;
use async_zip::{base::write::ZipFileWriter, Compression, ZipEntryBuilder};
use clap::{Parser, Subcommand};
use mrpack::{Game, Metadata};
use tokio::fs::File;

use reqwest::Client;
use rustls::crypto::aws_lc_rs;
use rustls_platform_verifier::BuilderVerifierExt;
use serde::{Deserialize, Serialize};

mod manifest;
mod mrpack;

use manifest::Manifest;

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
        Commands::Export => {
            let manifest: Manifest = toml_edit::de::from_slice(&fs::read("podzol.toml")?)?;

            let mut writer = ZipFileWriter::with_tokio(File::create("pack.mrpack").await?);

            // let dependencies = HashMap::from([("loader", manifest.pack.loader)]);
            let dependencies = HashMap::from([("minecraft".to_string(), manifest.pack.minecraft)]);

            let metadata = Metadata {
                format_version: 1,
                game: Game::Minecraft,
                version_id: "1.0.0".to_string(),
                name: "Mati qol ".to_string(),
                summary: None,
                files: vec![],
                dependencies,
            };

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
