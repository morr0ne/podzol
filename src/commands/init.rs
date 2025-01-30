use anyhow::{anyhow, Result};
use std::{collections::HashMap, fs, path::PathBuf};

use crate::{
    manifest::{self, Manifest},
    modrinth::{Client, VersionType},
};

pub async fn init(
    client: &Client,
    path: PathBuf,
    version: Option<String>,
    name: Option<String>,
) -> Result<()> {
    let name = if let Some(name) = name {
        name
    } else {
        // TODO: some degree of error handling I guess
        path.file_name()
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

    fs::write(
        path.join("podzol.toml"),
        &toml_edit::ser::to_string_pretty(&manifest)?,
    )?;

    fs::write(
        path.join(".gitignore"),
        r#"# The exported modpack
*.mrpack
"#,
    )?;

    if !fs::exists(path.join(".git"))? {
        git2::Repository::init(path)?;
    }

    Ok(())
}
