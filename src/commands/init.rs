use anyhow::{anyhow, Result};
use std::{collections::HashMap, env::current_dir, fs};

use crate::{
    manifest::{self, Manifest},
    modrinth::{Client, VersionType},
};

pub async fn init(client: &Client, version: Option<String>, name: Option<String>) -> Result<()> {
    let current_dir = current_dir().expect("Failed to get current directory");

    let name = if let Some(name) = name {
        name
    } else {
        // TODO: some degree of error handling I guess
        current_dir
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("pack")
            .to_string()
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

    git2::Repository::init(current_dir)?;

    Ok(())
}
