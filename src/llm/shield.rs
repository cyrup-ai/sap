//! Data shielding for large file system scans (planned feature)
#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};
use serde_json::Value;
use anyhow::Result;
use uuid::Uuid;

/// Maximum size for in-memory JSON (10MB)
const MAX_JSON_SIZE_BYTES: usize = 10 * 1024 * 1024;

/// Shield result indicating how the data was handled
#[derive(Debug)]
pub enum ShieldResult {
    /// Data is small enough to pass through
    PassThrough(Vec<Value>),
    /// Data was too large and written to file
    FileShielded {
        path: PathBuf,
        _original_size: usize,
        _entry_count: usize,
        summary: ShieldSummary,
    },
}

/// Summary of shielded data for the agent
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ShieldSummary {
    pub total_entries: usize,
    pub total_size_bytes: usize,
    pub file_path: String,
    pub top_level_dirs: Vec<String>,
    pub file_types_summary: std::collections::HashMap<String, usize>,
    pub largest_dirs: Vec<(String, usize)>,
    pub marker_files: Vec<String>,
}

/// Shield to protect against overly large JSON data
pub struct Shield {
    temp_dir: PathBuf,
}

impl Shield {
    pub fn new() -> Result<Self> {
        let temp_dir = PathBuf::from("/tmp/sap");
        fs::create_dir_all(&temp_dir)?;
        Ok(Self { temp_dir })
    }
    
    /// Process JSONL data and shield if necessary
    pub fn process(&self, jsonl_data: Vec<Value>, root_path: Option<&str>) -> Result<ShieldResult> {
        // Calculate size
        let json_string = serde_json::to_string(&jsonl_data)?;
        let size_bytes = json_string.len();
        
        if size_bytes <= MAX_JSON_SIZE_BYTES {
            return Ok(ShieldResult::PassThrough(jsonl_data));
        }
        
        // Data too large, write to file
        let file_name = format!("{}.jsonl", Uuid::new_v4());
        let file_path = self.temp_dir.join(&file_name);
        
        // Write each entry as a separate line (JSONL format)
        let mut file_content = String::new();
        for entry in &jsonl_data {
            file_content.push_str(&serde_json::to_string(entry)?);
            file_content.push('\n');
        }
        fs::write(&file_path, file_content)?;

        // Generate summary
        let summary = self.generate_summary(&jsonl_data, &file_path, root_path)?;
        
        Ok(ShieldResult::FileShielded {
            path: file_path,
            _original_size: size_bytes,
            _entry_count: jsonl_data.len(),
            summary,
        })
    }
    
    fn generate_summary(&self, data: &[Value], file_path: &Path, root_path: Option<&str>) -> Result<ShieldSummary> {
        let mut top_level_dirs = std::collections::HashSet::new();
        let mut file_types: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        let mut dir_sizes: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        let mut marker_files: Vec<String> = Vec::new();

        for entry in data {
            if let Some(path_str) = entry.get("path").and_then(|p| p.as_str()) {
                // Strip root_path prefix to get relative path
                let relative_path = if let Some(root) = root_path {
                    path_str.strip_prefix(root)
                        .and_then(|p| p.strip_prefix('/'))  // Remove leading slash
                        .unwrap_or(path_str)  // Fall back to original if strip fails
                } else {
                    path_str
                };

                // Extract filename for marker detection
                if let Some(filename) = relative_path.split('/').next_back()
                    && matches!(
                        filename,
                        "Cargo.toml" | "package.json" | "go.mod" | "pyproject.toml"
                        | "setup.py" | "pom.xml" | "build.gradle" | "Makefile"
                        | "CMakeLists.txt" | "README.md" | "main.rs" | "lib.rs"
                    )
                {
                    marker_files.push(relative_path.to_string());
                }

                // Extract top-level directory from RELATIVE path
                if let Some(first_component) = relative_path.split('/').next()
                    && !first_component.is_empty()
                {
                    top_level_dirs.insert(first_component.to_string());
                }

                // Count file types
                if let Some(file_type) = entry.get("type").and_then(|t| t.as_str()) {
                    *file_types.entry(file_type.to_string()).or_insert(0) += 1;
                }

                // Track directory sizes (use original absolute path)
                if let Some(parent) = PathBuf::from(path_str).parent()
                    && let Some(size) = entry.get("size").and_then(|s| s.as_u64()) {
                        *dir_sizes.entry(parent.to_string_lossy().to_string()).or_insert(0) += size as usize;
                    }
            }
        }

        // Get largest directories
        let mut largest_dirs: Vec<(String, usize)> = dir_sizes.into_iter().collect();
        largest_dirs.sort_by(|a, b| b.1.cmp(&a.1));
        largest_dirs.truncate(10);

        Ok(ShieldSummary {
            total_entries: data.len(),
            total_size_bytes: serde_json::to_string(data)?.len(),
            file_path: file_path.to_string_lossy().to_string(),
            top_level_dirs: top_level_dirs.into_iter().collect(),
            file_types_summary: file_types,
            largest_dirs,
            marker_files,
        })
    }
}

