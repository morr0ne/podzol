use anyhow::{Result, anyhow};
use inquire::{Select, Text};
use std::{collections::HashMap, env::current_dir, fs, path::Path};

use crate::{
    manifest::{self, Manifest},
    modrinth::{Client, VersionType},
};

fn name_from_path(path: &Path) -> &str {
    path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("pack")
}

pub async fn init_interactive(client: &Client) -> Result<()> {
    let pwd = current_dir().expect("Failed to fetch current dir");

    let name = Text::new("Name")
        .with_default(name_from_path(&pwd))
        .prompt()?;
    let version = Text::new("Version").with_default("0.1.0").prompt()?;

    let versions = client
        .get_game_versions()
        .await?
        .into_iter()
        .map(|v| v.version)
        .collect();

    let game_version = Select::new("Game version", versions).prompt()?;

    init(client, &pwd, version, Some(game_version), Some(name)).await
}

pub async fn init(
    client: &Client,
    path: &Path,
    version: String,
    game_version: Option<String>,
    name: Option<String>,
) -> Result<()> {
    let name = if let Some(name) = name {
        name
    } else {
        name_from_path(path).to_string()
    };

    let minecraft_version = if let Some(game_version) = game_version {
        game_version
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
            version,
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
