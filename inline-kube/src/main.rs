use std::path::PathBuf;

use rustkube::{direct::KubeConfig, kube_dir};

fn main() -> anyhow::Result<()> {
    let file_path = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| kube_dir().join("config"));

    let mut kc = KubeConfig::read_from(file_path)?;

    kc.inline()?;

    serde_yaml::to_writer(std::io::stdout(), &kc)?;

    Ok(())
}
