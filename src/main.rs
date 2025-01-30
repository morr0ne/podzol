use std::{fmt::Display, str::FromStr};

use anyhow::Result;
use clap::{Parser, Subcommand};
use commands::init;
use manifest::Loader;
use modrinth::Client;

mod commands;
mod manifest;
mod modrinth;
mod mrpack;

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
            commands::add(&client, projects, project_type).await?;
        }
        Commands::Export => {
            commands::export(&client).await?;
        }
        Commands::Init { version, name, .. } => {
            init(&client, version, name).await?;
        }
        Commands::Remove => todo!(),
    }

    Ok(())
}
