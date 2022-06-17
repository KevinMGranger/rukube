pub mod clean;
pub mod direct;

use std::path::{PathBuf, Path};

pub use clean::*;

pub fn kube_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap();
    Path::new(&home).join(".kube")
}
