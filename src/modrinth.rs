use std::{collections::HashMap, sync::Arc};

use anyhow::Result;
use itertools::Itertools;
use reqwest::Client as HttpClient;
use rustls::crypto::aws_lc_rs;
use rustls_platform_verifier::BuilderVerifierExt;
use serde::{Deserialize, Serialize};

use crate::{
    manifest::{Loader, Side},
    mrpack::Requirement,
};

#[derive(Debug, Deserialize, Serialize)]
pub struct Version {
    pub version_number: String,
    pub files: Vec<File>,
}

#[derive(Debug, Deserialize, Serialize)]

pub struct File {
    pub hashes: HashMap<String, String>,
    pub url: String,
    pub filename: String,
    pub size: u64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Project {
    pub client_side: Requirement,
    pub server_side: Requirement,
}

impl Project {
    pub fn get_side(&self) -> Side {
        if self.client_side.is_needed() && !self.server_side.is_needed() {
            return Side::Client;
        }

        if !self.client_side.is_needed() && self.server_side.is_needed() {
            return Side::Server;
        }

        Side::Both
    }
}

#[derive(Clone)]
pub struct Client {
    http_client: HttpClient,
}

impl Client {
    pub fn new() -> Result<Self> {
        let http_client = HttpClient::builder()
            .use_preconfigured_tls(
                rustls::ClientConfig::builder_with_provider(
                    Arc::new(aws_lc_rs::default_provider()),
                )
                .with_safe_default_protocol_versions()?
                .with_platform_verifier()
                .with_no_client_auth(),
            )
            .build()?;

        Ok(Self { http_client })
    }

    pub async fn get_project_versions(
        &self,
        project: &str,
        minecraft: &str,
        loaders: &HashMap<Loader, String>,
    ) -> Result<Vec<Version>> {
        let loaders = loaders
            .iter()
            .format_with(",", |(loader, _), f| f(&format_args!("\"{loader}\"")));

        let res = self
            .http_client
            .get(format!(
                "https://api.modrinth.com/v2/project/{project}/version"
            ))
            .query(&[
                ("loaders", format!("[\"minecraft\", {loaders}]")),
                ("game_versions", format!(r#"["{minecraft}"]"#)),
            ])
            .send()
            .await?
            .json()
            .await?;

        Ok(res)
    }

    pub async fn get_version(&self, project: &str, version: &str) -> Result<Version> {
        let res = self
            .http_client
            .get(format!(
                "https://api.modrinth.com/v2/project/{project}/version/{version}"
            ))
            .send()
            .await?
            .json()
            .await?;

        Ok(res)
    }

    pub async fn get_project(&self, project: &str) -> Result<Project> {
        let res = self
            .http_client
            .get(format!("https://api.modrinth.com/v2/project/{project}"))
            .send()
            .await?
            .json()
            .await?;

        Ok(res)
    }
}
