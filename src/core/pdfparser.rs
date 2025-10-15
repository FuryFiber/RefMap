use std::process::Command;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub title : String,
    pub keywords: Vec<String>,
    pub authors: Vec<String>,
    pub date: String,
    pub path: String,
}

impl Metadata {
    pub fn from_file(path: &str) -> Result<Metadata, anyhow::Error> {
        let output = Command::new("pdfinfo")
            .arg(path)
            .output()
            .expect("Failed to run pdfinfo");

        if !output.status.success() {
            eprintln!("Error running pdfinfo");
            return Err(anyhow::anyhow!("Error running pdfinfo"));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Initialize fields
        let mut title = String::new();
        let mut keywords = Vec::new();
        let mut authors = Vec::new();
        let mut date = String::new();

        for line in stdout.lines() {
            if let Some(rest) = line.strip_prefix("Title:") {
                title = rest.trim().to_string();
            } else if let Some(rest) = line.strip_prefix("Author:") {
                authors = rest
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect();
            } else if let Some(rest) = line.strip_prefix("Keywords:") {
                keywords = rest
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect();
            } else if let Some(rest) = line.strip_prefix("CreationDate:") {
                date = rest.trim().to_string();
            }
        }

        Ok(Metadata {
            title,
            keywords,
            authors,
            date,
            path: path.to_string(),
        })
    }
}