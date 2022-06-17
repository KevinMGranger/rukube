use std::{collections::BTreeMap, path::Path};

use crate::direct;
pub use crate::direct::{Cluster, ClusterSpec, Context, ContextSpec, Kind, User, UserSpec};

#[derive(Debug, Clone)]
pub struct KubeConfig {
    pub contexts: BTreeMap<String, ContextSpec>,
    pub current_context: String,
    pub clusters: BTreeMap<String, ClusterSpec>,
    pub preferences: serde_yaml::Value,
    pub users: BTreeMap<String, UserSpec>,
}

impl From<direct::KubeConfig> for KubeConfig {
    fn from(kc: direct::KubeConfig) -> Self {
        Self {
            current_context: kc.current_context,
            preferences: kc.preferences,
            contexts: kc
                .contexts
                .into_iter()
                .map(|ctx| (ctx.name, ctx.context))
                .collect(),
            clusters: kc
                .clusters
                .into_iter()
                .map(|cls| (cls.name, cls.cluster))
                .collect(),
            users: kc
                .users
                .into_iter()
                .map(|usr| (usr.name, usr.user))
                .collect(),
        }
    }
}

impl Into<direct::KubeConfig> for KubeConfig {
    fn into(self) -> direct::KubeConfig {
        direct::KubeConfig {
            kind: Kind::Config,
            api_version: direct::ApiVersion::V1,
            preferences: self.preferences,
            current_context: self.current_context,

            clusters: self
                .clusters
                .into_iter()
                .map(|(name, cluster)| Cluster { name, cluster })
                .collect(),
            contexts: self
                .contexts
                .into_iter()
                .map(|(name, context)| Context { name, context })
                .collect(),
            users: self
                .users
                .into_iter()
                .map(|(name, user)| User { name, user })
                .collect(),
        }
    }
}

pub fn read_config() -> anyhow::Result<KubeConfig> {
    direct::read_config().map(KubeConfig::from)
}

pub fn write_config(kc: KubeConfig, path: &Path) -> anyhow::Result<()> {
    direct::write_config(&kc.into(), path)
}
// endregion
