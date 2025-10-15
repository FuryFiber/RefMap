use crate::core::map::MindMap;
use std::fs;
use anyhow::Result;

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