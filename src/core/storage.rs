use crate::core::map::MindMap;
use crate::core::theme::SerializeableTheme;
use std::{fs, io};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use anyhow::{Error, Result};
use toml::Value;
use zip::write::{ExtendedFileOptions, FileOptions};
use zip::ZipWriter;

pub fn save_map(map: &MindMap, path: &str) -> Result<()> {
    let project_dir = std::path::Path::new(path);
    if !project_dir.exists() {
        fs::create_dir_all(project_dir)?;
    }

    let json_path = project_dir.join("map.json");
    let data = serde_json::to_string_pretty(map)?;
    fs::write(&json_path, data)?;

    // Copy PDFs into the project directory's "pdfs" folder
    let pdfs_dir = project_dir.join("pdfs");
    if !pdfs_dir.exists() {
        fs::create_dir_all(&pdfs_dir)?;
    }

    for node in &map.nodes {
        if let Some(relative_path) = &node.path {
            let original_path = std::path::Path::new(relative_path);
            if original_path.is_relative() {
                // This is a placeholder; actual implementation depends on tracking original paths
                // For this example, assume relative paths are already correct
                continue;
            }
            let file_name = original_path.file_name().unwrap().to_str().unwrap();
            let dest_path = pdfs_dir.join(file_name);
            if !dest_path.exists() {
                std::fs::copy(original_path, &dest_path)?;
            }
        }
    }

    Ok(())
}

pub fn load_map(path: &str) -> Result<MindMap> {
    let project_dir = std::path::Path::new(path);
    let json_path = project_dir.join("map.json");
    let data = fs::read_to_string(&json_path)?;
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

    let table = config.as_table_mut().ok_or_else(|| Error::msg("Config is not a table"))?;
    table.insert("last_file".to_string(), Value::String(path.to_string()));

    let encoded = toml::to_string(&config)?;
    fs::write(&config_path, encoded)?;
    Ok(())
}

pub fn export_project(project_dir: &str, zip_path: &str) -> Result<()> {
    let project_path = Path::new(project_dir);
    if !project_path.exists() {
        return Err(anyhow::Error::msg("Project directory does not exist"));
    }

    let file = File::create(zip_path)?;
    let mut zip = ZipWriter::new(file);
    let options: FileOptions<ExtendedFileOptions> = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    // Recursively walk through the project directory
    for entry in walkdir::WalkDir::new(project_path).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        let name = path.strip_prefix(project_path)?.to_str().unwrap();

        if path.is_dir() {
            zip.add_directory(name, options.clone())?;
        } else {
            let mut file = File::open(path)?;
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer)?;
            zip.start_file(name, options.clone())?;
            zip.write_all(&buffer)?;
        }
    }

    zip.finish()?;
    Ok(())
}

pub fn get_theme() -> Result<SerializeableTheme, anyhow::Error> {
    let config_dir = get_config_dir();
    let json_path = config_dir.join("theme.json");
    let data = fs::read_to_string(&json_path)?;
    Ok(serde_json::from_str(&data)?)
}
