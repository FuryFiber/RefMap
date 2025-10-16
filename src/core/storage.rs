use crate::core::map::MindMap;
use std::{fs, io};
use std::io::Read;
use anyhow::{Error, Result};
use toml::Value;

pub fn save_map(map: &MindMap, path: &str) -> Result<()> {
    let data = serde_json::to_string_pretty(map)?;
    fs::write(path, data)?;
    Ok(())
}

pub fn load_map(path: &str) -> Result<MindMap> {
    let data = fs::read_to_string(path)?;
    let map: MindMap = serde_json::from_str(&data)?;
    Ok(map)
}

fn get_config_dir() -> std::path::PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("refmap")
}

pub fn load_last_file() -> Result<String, Error> {
    let config_dir = get_config_dir();
    let config_path = config_dir.join("config.toml");

    if !config_dir.exists() {
        return Err(Error::from(io::Error::new(io::ErrorKind::NotFound, "Config directory not found")));
    }

    if !config_path.exists() {
        return Err(Error::from(io::Error::new(io::ErrorKind::NotFound, "Config file not found")));
    }

    let mut data = String::new();
    fs::File::open(&config_path)?.read_to_string(&mut data)?;
    let config: Value = toml::from_str(&data)?;
    if let Some(Value::String(path)) = config.get("last_file") {
        Ok(path.clone())
    } else {
        Err(Error::from(io::Error::new(io::ErrorKind::InvalidData, "last_file not found")))
    }
}

pub fn save_last_file(path: &str) -> Result<(), Error> {
    let config_dir = get_config_dir();
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir)?;
    }

    let config_path = config_dir.join("config.toml");

    let mut config: Value = if config_path.exists() {
        let data = fs::read_to_string(&config_path)?;
        toml::from_str(&data)?
    } else {
        Value::Table(Default::default())
    };

    // Safely get a mutable reference to the table
    let table = config.as_table_mut().ok_or_else(|| {
        Error::msg("Config is not a table")
    })?;

    // Insert the last_file key
    table.insert("last_file".to_string(), Value::String(path.to_string()));

    // Write the updated config back to the file
    let encoded = toml::to_string(&config)?;
    fs::write(&config_path, encoded)?;
    Ok(())
}
