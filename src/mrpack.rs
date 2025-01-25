use std::{collections::HashMap, fmt::Display, path::PathBuf, str::FromStr};

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

use crate::manifest::Side;

#[serde_as]
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    pub format_version: u32,
    #[serde_as(as = "DisplayFromStr")]
    pub game: Game,
    pub version_id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    pub files: Vec<File>,
    pub dependencies: HashMap<String, String>,
}

#[derive(Debug)]
pub enum Game {
    Minecraft,
}

impl FromStr for Game {
    type Err = String;

    fn from_str(game: &str) -> Result<Self, Self::Err> {
        match game {
            "minecraft" => Ok(Self::Minecraft),
            _ => Err(format!(
                "Unknown game '{game}'. The only supported game is minecraft",
            )),
        }
    }
}

impl Display for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Minecraft => write!(f, "minecraft"),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct File {
    pub path: PathBuf,
    pub hashes: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<Env>,
    pub downloads: Vec<String>,
    pub file_size: u64, // I doubt there's stuff with files above 4gb or if it's even allowed but it's here I guess
}

#[serde_as]
#[derive(Debug, Deserialize, Serialize)]
pub struct Env {
    #[serde_as(as = "DisplayFromStr")]
    pub client: Requirement,
    #[serde_as(as = "DisplayFromStr")]
    pub server: Requirement,
}

impl From<Side> for Env {
    fn from(side: Side) -> Self {
        match side {
            Side::Client => Env {
                client: Requirement::Required,
                server: Requirement::Unsupported,
            },
            Side::Server => Env {
                client: Requirement::Unsupported,
                server: Requirement::Required,
            },
            Side::Both => Env {
                client: Requirement::Required,
                server: Requirement::Required,
            },
        }
    }
}

#[derive(Debug)]
pub enum Requirement {
    Required,
    Optional,
    Unsupported,
}

impl Requirement {
    pub const fn is_needed(&self) -> bool {
        match self {
            Self::Required | Self::Optional => true,
            Self::Unsupported => false,
        }
    }
}

impl FromStr for Requirement {
    type Err = String;

    fn from_str(requirement: &str) -> Result<Self, Self::Err> {
        match requirement {
            "required" => Ok(Self::Required),
            "optional" => Ok(Self::Optional),
            "unsupported" => Ok(Self::Unsupported),
            _ => Err(format!(
                "Unknown requirement '{requirement}'. Supported requirements are: required, optional, unsupported",
            )),
        }
    }
}

impl Display for Requirement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Required => write!(f, "required"),
            Self::Optional => write!(f, "optional"),
            Self::Unsupported => write!(f, "unsupported"),
        }
    }
}
