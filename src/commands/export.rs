use std::fs;

use anyhow::Result;
use async_zip::base::write::ZipFileWriter;
use tokio::fs::File;

use crate::{manifest::Manifest, modrinth::Client};

pub async fn export(client: &Client) -> Result<()> {
    let manifest: Manifest = toml_edit::de::from_slice(&fs::read("podzol.toml")?)?;

    let mut writer = ZipFileWriter::with_tokio(
        File::create(format!(
            "{}-{}.mrpack",
            manifest.pack.name, manifest.pack.version
        ))
        .await?,
    );

    manifest.build_mrpack(client, &mut writer).await?;

    writer.close().await?;

    Ok(())
}
