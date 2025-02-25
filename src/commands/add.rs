use anyhow::Result;
use std::fs;
use toml_edit::{DocumentMut, InlineTable};

use crate::{ProjectType, manifest::Manifest, modrinth::Client};

pub async fn add(client: &Client, projects: Vec<String>, project_type: ProjectType) -> Result<()> {
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

    Ok(())
}
