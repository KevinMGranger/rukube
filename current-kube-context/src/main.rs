use anyhow::Context;
use rustkube::read_config;

fn main() -> anyhow::Result<()> {
    let kube_config = read_config()?;

    let context = kube_config
        .contexts
        .get(&kube_config.current_context)
        .context("No matching context found")?;
    match &context.namespace {
        Some(ns) => println!("{}", ns),
        None => println!("No namespace"),
    }

    Ok(())
}
