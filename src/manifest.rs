use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
struct Manifest {
    pack: Pack,
    #[serde(default)]
    mods: HashMap<String, Mod>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Pack {
    name: String,
    version: String,
    minecraft: String,
    loader: Loader,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Loader {
    Fabric,
    Forge,
    Quilt,
    NeoForge,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
enum Mod {
    Version(String),
}
