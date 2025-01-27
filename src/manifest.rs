use anyhow::Result;
use futures_util::future::try_join_all;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};
use serde_with::{DeserializeFromStr, SerializeDisplay};
use std::{collections::HashMap, fmt::Display, path::Path, str::FromStr};
use tokio::task;

use crate::{
    modrinth::Client,
    mrpack::{self, Game, Metadata},
};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct Manifest {
    pub pack: Pack,
    pub enviroment: Enviroment,
    pub files: HashMap<String, File>,
    #[serde(default)]
    pub mods: HashMap<String, Definition>,
    #[serde(default)]
    pub resource_packs: HashMap<String, Definition>,
    #[serde(default)]
    pub shaders: HashMap<String, Definition>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct File {
    pub path: String,
    pub side: Side,
}

impl Manifest {
    pub async fn into_metadata(self, client: &Client) -> Result<Metadata> {
        async fn process_items<P: AsRef<Path> + Send + 'static>(
            client: Client,
            items: HashMap<String, Definition>,
            path: P,
            mp: MultiProgress,
            total_pb: ProgressBar,
        ) -> Result<Vec<mrpack::File>> {
            let tasks: Vec<_> = items
                .into_iter()
                .map(|(name, definition)| {
                    let client = client.clone();
                    let path = path.as_ref().to_owned();
                    let pb = mp.add(ProgressBar::new(1));
                    pb.set_style(
                        ProgressStyle::default_bar()
                            .template(
                                "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
                            )
                            .unwrap(),
                    );
                    pb.set_message(format!("Processing {}", name));

                    let total_pb = total_pb.clone();

                    task::spawn(async move {
                        let Definition { version, side } = definition;
                        let version = client.get_version(&name, &version).await?;
                        let mut files = Vec::new();

                        for file in version.files {
                            files.push(mrpack::File {
                                path: path.join(&file.filename),
                                hashes: file.hashes,
                                env: Some(side.clone().into()),
                                downloads: vec![file.url],
                                file_size: file.size,
                            });
                        }

                        pb.inc(1);
                        pb.finish_and_clear();
                        total_pb.inc(1);
                        Ok::<_, anyhow::Error>(files)
                    })
                })
                .collect();

            let results = try_join_all(tasks).await?;
            Ok(results.into_iter().flatten().flatten().collect())
        }

        let total_items = self.mods.len() + self.resource_packs.len() + self.shaders.len();
        let mp = MultiProgress::new();
        let total_pb = mp.add(ProgressBar::new(total_items as u64));
        total_pb.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40.green/blue} {pos:>7}/{len:7} {msg}")
                .unwrap(),
        );
        total_pb.set_message("Total progress");

        // Process all types concurrently
        let (mods, resource_packs, shaders) = tokio::try_join!(
            process_items(
                client.clone(),
                self.mods,
                "mods",
                mp.clone(),
                total_pb.clone()
            ),
            process_items(
                client.clone(),
                self.resource_packs,
                "resourcepacks",
                mp.clone(),
                total_pb.clone()
            ),
            process_items(
                client.clone(),
                self.shaders,
                "shaderpacks",
                mp.clone(),
                total_pb.clone()
            ),
        )?;

        let mut files = Vec::with_capacity(total_items);
        files.extend(mods);
        files.extend(resource_packs);
        files.extend(shaders);

        total_pb.finish_and_clear();
        mp.clear().unwrap();

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

// Rest of the code remains unchanged...

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
