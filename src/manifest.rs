use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use std::{collections::HashMap, fmt::Display, str::FromStr};

#[derive(Debug, Deserialize, Serialize)]
pub struct Manifest {
    pub pack: Pack,
    #[serde(default)]
    pub mods: HashMap<String, Mod>,
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
pub struct Pack {
    pub name: String,
    pub version: String,
    pub minecraft: String,
    #[serde_as(as = "DisplayFromStr")]
    pub loader: Loader,
}

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Mod {
    Version(String),
}
