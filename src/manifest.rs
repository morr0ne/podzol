use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_with::{DeserializeFromStr, SerializeDisplay};
use std::{collections::HashMap, fmt::Display, path::PathBuf, str::FromStr};

use crate::{
    modrinth::Client,
    mrpack::{self, Game, Metadata},
};

#[derive(Debug, Deserialize, Serialize)]
pub struct Manifest {
    pub pack: Pack,
    pub enviroment: Enviroment,
    #[serde(default)]
    pub mods: HashMap<String, Definition>,
    #[serde(default)]
    pub resource_packs: HashMap<String, Definition>,
    #[serde(default)]
    pub data_packs: HashMap<String, Definition>,
    #[serde(default)]
    pub shaders: HashMap<String, Definition>,
}

impl Manifest {
    pub async fn into_metadata(self, client: &Client) -> Result<Metadata> {
        let mut files = Vec::with_capacity(self.mods.len());

        for (name, definition) in self.mods {
            let Definition { version, side } = definition;

            let version = client.get_version(&name, &version).await?;

            for file in version.files {
                files.push(mrpack::File {
                    path: PathBuf::from("mods").join(file.filename),
                    hashes: file.hashes,
                    env: Some(side.clone().into()),
                    downloads: vec![file.url],
                    file_size: file.size,
                });
            }
        }

        let dependencies: HashMap<String, String> = self
            .enviroment
            .loaders
            .into_iter()
            .map(|(loader, version)| (loader.as_mrpack().to_string(), version))
            .chain(std::iter::once((
                "minecraft".to_string(),
                self.enviroment.minecraft,
            )))
            .collect();

        let metadata = Metadata {
            format_version: 1,
            game: Game::Minecraft,
            version_id: self.pack.version,
            name: self.pack.name,
            summary: self.pack.description,
            files,
            dependencies,
        };

        Ok(metadata)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Pack {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Enviroment {
    pub minecraft: String,
    #[serde(default, flatten)]
    pub loaders: HashMap<Loader, String>,
}

#[derive(Debug, DeserializeFromStr, SerializeDisplay, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Loader {
    Fabric,
    Forge,
    Quilt,
    NeoForge,
}

impl FromStr for Loader {
    type Err = String;

    fn from_str(loader: &str) -> Result<Self, Self::Err> {
        match loader {
            "fabric" => Ok(Self::Fabric),
            "forge" => Ok(Self::Forge),
            "quilt" => Ok(Self::Quilt),
            "neoforge" => Ok(Self::NeoForge),
            _ => Err(format!(
                "Unknown loader '{loader}'. Supported loaders are: fabric, forge, quilt, neoforge",
            )),
        }
    }
}

impl Display for Loader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Fabric => write!(f, "fabric"),
            Self::Forge => write!(f, "forge"),
            Self::Quilt => write!(f, "quilt"),
            Self::NeoForge => write!(f, "neoforge"),
        }
    }
}

impl Loader {
    pub const fn as_mrpack(&self) -> &'static str {
        match self {
            Self::Fabric => "fabric-loader",
            Self::Forge => "forge",
            Self::Quilt => "quilt-loader",
            Self::NeoForge => "neoforge",
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Definition {
    version: String,
    side: Side,
}

#[derive(Debug, DeserializeFromStr, SerializeDisplay, Clone)]

pub enum Side {
    Client,
    Server,
    Both,
}

impl FromStr for Side {
    type Err = String;

    fn from_str(side: &str) -> Result<Self, Self::Err> {
        match side {
            "client" => Ok(Self::Client),
            "server" => Ok(Self::Server),
            "both" => Ok(Self::Both),
            _ => Err(format!(
                "Unknown side '{side}'. Supported sides are: client, server, both",
            )),
        }
    }
}

impl Display for Side {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Client => write!(f, "client"),
            Self::Server => write!(f, "server"),
            Self::Both => write!(f, "both"),
        }
    }
}
