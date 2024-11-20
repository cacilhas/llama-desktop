extern crate toml;

use std::{
    error::Error,
    fmt::Display,
    fs::File,
    io::Write,
    path::PathBuf,
};

use toml::Table;

fn main() -> Result<(), Box<dyn Error>> {
    let out_path = ::std::env::var("OUT_DIR")?;
    let out_path = PathBuf::from(out_path);
    let cargo = include_str!("Cargo.toml").parse::<Table>()?;
    let Some(package) = cargo["package"].as_table() else {
        return Err(Missing("package table").into());
    };
    let Some(version) = package["version"].as_str() else {
        return Err(Missing("package.version").into());
    };
    let mut f = File::create(out_path.join("version.rs"))?;
    writeln!(f, "pub const VERSION: &str = \"{}\";", version)?;
    Ok(())
}

#[derive(Debug)]
struct Missing(&'static str);

impl Error for Missing {}

impl Display for Missing {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} not found", self.0)
    }
}
