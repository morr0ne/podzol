use std::{collections::HashMap, env::current_dir, fmt::Display, fs, str::FromStr};

use anyhow::{anyhow, Result};
use async_zip::base::write::ZipFileWriter;
use clap::{Parser, Subcommand};
use tokio::fs::File;

mod manifest;
mod modrinth;
mod mrpack;

use manifest::{Loader, Manifest};
use modrinth::{Client, VersionType};
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
    Init {
        /// The name of this modpack (default to the directory name)
        #[arg(short, long)]
        name: Option<String>,
        /// The minecraft version (defaults to latest)
        #[arg(short, long)]
        version: Option<String>,
        /// A compatible loader
        #[arg(short, long)]
        loader: Option<Loader>,
    },
    /// Add a project to the manifest
    Add {
        #[arg(required = true, num_args = 1..)]
        projects: Vec<String>,
        #[arg(long = "type", short = 't', default_value = "mod")]
        project_type: ProjectType,
    },
    /// Remove a mod from the manifest
    Remove,
    /// Exports the project
    Export,
}

#[derive(Clone)]
enum ProjectType {
    Mod,
    ResourcePack,
    Shader,
}

impl ProjectType {
    pub const fn as_table(&self) -> &'static str {
        match self {
            Self::Mod => "mods",
            Self::ResourcePack => "resource-packs",
            Self::Shader => "shaders",
        }
    }
}

impl FromStr for ProjectType {
    type Err = String;

    fn from_str(project_type: &str) -> Result<Self, Self::Err> {
        match project_type {
            "mod" | "mods" => Ok(Self::Mod),
            "resource-pack" | "resource-packs" | "resource" | "resources" => Ok(Self::ResourcePack),
            "shader" | "shaders"=> Ok(Self::Shader),
            _ => Err(format!(
                "Unknown type '{project_type}'. Supported project types are: mod, resource-pack, shader",
            )),
        }
    }
}

impl Display for ProjectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Mod => write!(f, "mod"),
            Self::ResourcePack => write!(f, "resource-pack"),
            Self::Shader => write!(f, "shader"),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let Args { command } = Args::parse();

    let client = Client::new()?;

    match command {
        Commands::Add {
            projects,
            project_type,
        } => {
            let manifest_src = fs::read_to_string("podzol.toml")?;
            let mut document: DocumentMut = manifest_src.parse()?;
            let manifest: Manifest = toml_edit::de::from_document(document.clone())?;

            for name in projects {
                let project = client.get_project(&name).await?;

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
                mod_table.insert("side", project.get_side().to_string().into());
                document[project_type.as_table()][&name] = mod_table.into();

                println!(
                    "Added {name} {version_number} to {}",
                    project_type.as_table()
                );
            }

            fs::write("podzol.toml", document.to_string())?;
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

            manifest.build_mrpack(&client, &mut writer).await?;

            writer.close().await?;
        }
        Commands::Init { version, name, .. } => {
            let name = if let Some(name) = name {
                name
            } else {
                "pack".to_string()
                // let dir = current_dir()?.file_name();

                // todo!()
            };

            let minecraft_version = if let Some(version) = version {
                version
            } else {
                let versions = client.get_game_versions().await?;

                let latest_version = versions
                    .into_iter()
                    .filter(|version| matches!(version.version_type, VersionType::Release))
                    .max_by_key(|version| version.date)
                    .ok_or(anyhow!("No valid Minecraft versions found"))?;

                latest_version.version
            };

            let manifest = Manifest {
                pack: manifest::Pack {
                    name,
                    version: "0.1.0".to_string(),
                    description: None,
                },
                enviroment: manifest::Enviroment {
                    minecraft: minecraft_version,
                    loaders: HashMap::new(),
                },
                files: HashMap::new(),
                mods: HashMap::new(),
                resource_packs: HashMap::new(),
                shaders: HashMap::new(),
            };

            fs::write("podzol.toml", &toml_edit::ser::to_string_pretty(&manifest)?)?;
        }
        Commands::Remove => todo!(),
    }

    Ok(())
}
