use toml::Table;

#[dynamic]
pub static VERSION: String = {
    let cargo = include_str!("../../Cargo.toml").parse::<Table>().unwrap();
    let package = cargo["package"].as_table().unwrap();
    let version = package["version"].as_str().unwrap();
    return version.to_string();
};
