use anyhow::Result;
use rustkube::read_config;

fn main() -> Result<()> {
    let config = read_config()?;

    println!("{config:#?}");

    Ok(())
}
