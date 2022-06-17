use std::io::{self, BufRead};
use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::Context as _;
use serde::*;
use serde_yaml::Value as YamlValue;

use crate::kube_dir;

// region: Context
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ContextSpec {
    pub user: String,
    pub namespace: Option<String>,
    pub cluster: String,
    pub extensions: Option<YamlValue>,
}
#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Context {
    pub name: String,
    pub context: ContextSpec,
}
// endregion

// region: Cluster
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct ClusterSpec {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub certificate_authority_data: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub certificate_authority: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub insecure_skip_tls_verify: Option<YamlValue>,
    pub server: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<YamlValue>,
}

fn inline_data(path: &mut Option<PathBuf>, data: &mut Option<String>) -> io::Result<()> {
    if data.is_some() {
        return Ok(());
    }

    match path {
        Some(path) => {
            let read_data = fs::read_to_string(path)?;
            let inline_data = read_data
                .lines()
                .filter(|s| !(s.starts_with("-----") || s.is_empty()))
                .collect();
            *data = Some(inline_data);
        }
        None => return Ok(()),
    }

    *path = None;

    Ok(())
}

impl ClusterSpec {
    pub fn inline(&mut self) -> io::Result<()> {
        inline_data(
            &mut self.certificate_authority,
            &mut self.certificate_authority_data,
        )
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct Cluster {
    pub name: String,
    pub cluster: ClusterSpec,
}

// endregion

// region: User
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case", deny_unknown_fields, untagged)]
pub enum UserSpec {
    Token {
        token: String,
    },
    #[serde(rename_all = "kebab-case")]
    Cert {
        #[serde(skip_serializing_if = "Option::is_none")]
        client_certificate: Option<PathBuf>,
        #[serde(skip_serializing_if = "Option::is_none")]
        client_certificate_data: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        client_key: Option<PathBuf>,
        #[serde(skip_serializing_if = "Option::is_none")]
        client_key_data: Option<String>,
    },
}

impl UserSpec {
    pub fn inline(&mut self) -> io::Result<()> {
        match self {
            UserSpec::Token { .. } => Ok(()),
            UserSpec::Cert {
                client_certificate,
                client_certificate_data,
                client_key,
                client_key_data,
            } => {
                inline_data(client_certificate, client_certificate_data)?;
                inline_data(client_key, client_key_data)?;

                Ok(())
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct User {
    pub name: String,
    pub user: UserSpec,
}
// endregion

// region: Common
#[derive(Serialize, Deserialize, Debug)]
pub enum ApiVersion {
    #[serde(rename = "v1")]
    V1,
}
#[derive(Serialize, Deserialize, Debug)]
pub enum Kind {
    Config,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct KubeConfig {
    pub kind: Kind,
    #[serde(rename = "apiVersion")]
    pub api_version: ApiVersion,
    pub contexts: Vec<Context>,
    pub current_context: String,
    pub clusters: Vec<Cluster>,
    pub preferences: serde_yaml::Value,
    pub users: Vec<User>,
}

impl KubeConfig {
    pub fn inline(&mut self) -> io::Result<()> {
        for cluster in &mut self.clusters {
            cluster.cluster.inline()?;
        }

        for user in &mut self.users {
            user.user.inline()?;
        }

        Ok(())
    }

    pub fn read_from(path: impl AsRef<Path>) -> anyhow::Result<KubeConfig> {
        Ok(serde_yaml::from_reader(
            fs::OpenOptions::new()
                .read(true)
                .open(path)
                .context("Opening kube config")?,
        )
        .context("Parsing kube config")?)
    }
}

pub fn read_config() -> anyhow::Result<KubeConfig> {
    let kube_config_file = fs::OpenOptions::new()
        .read(true)
        .open(kube_dir().join("config"))
        .context("Opening kube config")?;
    let kube_config: KubeConfig =
        serde_yaml::from_reader(kube_config_file).context("Parsing kube config")?;
    Ok(kube_config)
}

pub fn write_config(kc: &KubeConfig, path: &Path) -> anyhow::Result<()> {
    let file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .context("Opening kube config")?;

    Ok(serde_yaml::to_writer(file, kc)?)
}
// endregion
