//! LLM agent for intelligent file system analysis (planned feature)
#![allow(dead_code)]

use rig::providers::ollama;
use rig::agent::Agent;
use rig::completion::Prompt;
use rig::client::CompletionClient;
use rig_derive::rig_tool;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use anyhow::Result;
use std::fs::File;
use std::io::{BufRead, BufReader};

use super::shield::{Shield, ShieldResult};

/// Read the first N lines from a shielded JSONL file
#[rig_tool(
    description = "Read the first N lines from a shielded JSONL file",
    params(
        file_path = "Path to the shielded JSONL file",
        lines = "Number of lines to read from the beginning"
    )
)]
fn head_shielded_file(file_path: String, lines: usize) -> Result<Vec<String>, rig::tool::ToolError> {
    let file = File::open(&file_path).map_err(|e| rig::tool::ToolError::ToolCallError(Box::new(e)))?;
    let reader = BufReader::new(file);
    let result: Vec<String> = reader
        .lines()
        .take(lines)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| rig::tool::ToolError::ToolCallError(Box::new(e)))?;
    Ok(result)
}

/// Read the last N lines from a shielded JSONL file
#[rig_tool(
    description = "Read the last N lines from a shielded JSONL file",
    params(
        file_path = "Path to the shielded JSONL file",
        lines = "Number of lines to read from the end"
    )
)]
fn tail_shielded_file(file_path: String, lines: usize) -> Result<Vec<String>, rig::tool::ToolError> {
    let file = File::open(&file_path).map_err(|e| rig::tool::ToolError::ToolCallError(Box::new(e)))?;
    let reader = BufReader::new(file);
    let all_lines: Vec<String> = reader
        .lines()
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| rig::tool::ToolError::ToolCallError(Box::new(e)))?;
    
    let start = all_lines.len().saturating_sub(lines);
    Ok(all_lines[start..].to_vec())
}

/// Search for lines containing a pattern in a shielded JSONL file
#[rig_tool(
    description = "Search for lines containing a pattern in a shielded JSONL file",
    params(
        file_path = "Path to the shielded JSONL file",
        pattern = "Pattern to search for (case-insensitive)"
    )
)]
fn grep_shielded_file(file_path: String, pattern: String) -> Result<Vec<String>, rig::tool::ToolError> {
    let file = File::open(&file_path).map_err(|e| rig::tool::ToolError::ToolCallError(Box::new(e)))?;
    let reader = BufReader::new(file);
    let pattern_lower = pattern.to_lowercase();
    
    let result: Vec<String> = reader
        .lines()
        .filter_map(|line| {
            line.ok().and_then(|l| {
                if l.to_lowercase().contains(&pattern_lower) {
                    Some(l)
                } else {
                    None
                }
            })
        })
        .collect();
    Ok(result)
}

/// Sample random lines from a shielded JSONL file
#[rig_tool(
    description = "Sample random lines from a shielded JSONL file",
    params(
        file_path = "Path to the shielded JSONL file",
        count = "Number of random lines to sample"
    )
)]
fn sample_shielded_file(file_path: String, count: usize) -> Result<Vec<String>, rig::tool::ToolError> {
    use rand::prelude::IndexedRandom;
    
    let file = File::open(&file_path).map_err(|e| rig::tool::ToolError::ToolCallError(Box::new(e)))?;
    let reader = BufReader::new(file);
    let all_lines: Vec<String> = reader
        .lines()
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| rig::tool::ToolError::ToolCallError(Box::new(e)))?;
    
    let mut rng = rand::rng();
    let sampled: Vec<String> = all_lines
        .as_slice()
        .choose_multiple(&mut rng, count)
        .cloned()
        .collect();
    
    Ok(sampled)
}

/// Input structure for the file system agent
#[derive(Debug, Serialize, Deserialize)]
pub struct AgentInput {
    /// The high-level objective the LLM is trying to achieve
    pub objective: String,
    
    /// The specific current task being performed
    pub current_task: String,
    
    /// Instructions for what the agent should do with this data
    pub instructions: String,
    
    /// Metadata about the file system scan
    pub metadata: ScanMetadata,
    
    /// The actual file data (or reference if shielded)
    #[serde(flatten)]
    pub data: AgentData,
}

/// Metadata about the file system scan
#[derive(Debug, Serialize, Deserialize)]
pub struct ScanMetadata {
    /// Root path that was scanned
    pub root_path: String,
    
    /// Scan timestamp
    pub timestamp: String,
    
    /// Flags used for the scan
    pub scan_flags: ScanFlags,
    
    /// Version of SAP tool
    pub sap_version: String,
}

/// Scan configuration flags
#[derive(Debug, Serialize, Deserialize)]
pub struct ScanFlags {
    pub recursive: bool,
    pub include_hidden: bool,
    pub follow_symlinks: bool,
    pub git_status: bool,
}

/// The actual data - either direct or shielded reference
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "data_type")]
pub enum AgentData {
    /// Direct JSONL data (when small enough)
    Direct {
        entries: Vec<Value>,
    },
    
    /// Reference to shielded file (when too large)
    Shielded {
        file_path: String,
        summary: Box<super::shield::ShieldSummary>,
        mcp_instructions: McpInstructions,
    },
}

/// MCP tool instructions for exploring shielded files
#[derive(Debug, Serialize, Deserialize)]
pub struct McpInstructions {
    pub tool_name: String,
    pub available_commands: Vec<String>,
    pub usage_examples: Vec<String>,
}

/// Structured response from the file system agent
#[derive(Debug, Serialize, Deserialize)]
pub struct AgentResponse {
    /// Human-readable summary of the file structure
    pub summary: String,
    
    /// Statistical information about the scan
    pub statistics: FileStatistics,
    
    /// List of important files found
    pub key_files: Vec<String>,
    
    /// Analysis of the project structure
    pub structure_analysis: StructureAnalysis,
    
    /// Recommendations for further exploration
    pub recommendations: Vec<String>,
}

/// File system statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct FileStatistics {
    pub total_files: usize,
    pub total_dirs: usize,
    pub total_size_bytes: usize,
    pub primary_language: Option<String>,
    pub file_type_distribution: std::collections::HashMap<String, usize>,
}

/// Analysis of project structure
#[derive(Debug, Serialize, Deserialize)]
pub struct StructureAnalysis {
    pub project_type: ProjectType,
    pub key_directories: Vec<String>,
    pub observations: Vec<String>,
    pub detected_frameworks: Vec<String>,
    pub build_systems: Vec<String>,
}

/// Types of projects we can detect
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectType {
    WebApp,
    Library,
    CliTool,
    MobileApp,
    ApiService,
    MonoRepo,
    Documentation,
    DataPipeline,
    MachineLearning,
    Unknown,
}

const SYSTEM_PROMPT: &str = r#"You act as a terminal interpreting and modifying the content of an `ls` command to make the output easier for other LLMs to understand and action upon effectively.

SAP (the enhanced `ls` command) has thrown you the raw file listing data. Your job is to:
1. Interpret the file system structure like a human would when looking at `ls` output
2. Transform it into a format that helps other LLMs quickly understand what's in this directory/project
3. FOCUS the results based on the objective and current_task provided

Key inputs you'll receive:
- objective: The high-level goal the LLM is trying to achieve
- current_task: The specific task being performed right now
- instructions: Additional guidance for processing

Your focusing strategy:
- When given an objective/task, prioritize files and directories relevant to that goal
- Add contextual notes about file contents when they're relevant to the task
- De-emphasize or summarize areas that are out of scope
- For example:
  - If objective is "fix authentication bug" and current_task is "exploring auth module"
    → Focus on auth-related files, config files, tests
    → Add notes like "contains JWT validation logic" or "defines user permissions"
    → Summarize unrelated directories as "frontend assets (not relevant to auth)"
  - If objective is "understand build system" 
    → Focus on package.json, Cargo.toml, Makefile, etc.
    → Note build scripts, dependencies, CI/CD files
    → Briefly mention source code exists but don't detail it

Input formats you will receive:
1. Direct JSON array of file entries (when data is small enough)
2. Shielded reference object with:
   - status: "shielded"
   - summary: statistics about the file list
   - file_path: location of the full JSONL data
   - instructions: MCP tools available for exploration

Your output MUST be valid JSON matching the AgentResponse structure:
{
  "summary": "Human-readable summary FOCUSED on the objective/task",
  "statistics": {
    "total_files": number,
    "total_dirs": number,
    "total_size_bytes": number,
    "primary_language": "detected programming language or null",
    "file_type_distribution": {
      ".rs": count,
      ".js": count,
      // etc
    }
  },
  "key_files": ["files most relevant to the objective/task with brief notes"],
  "structure_analysis": {
    "project_type": "web_app|library|cli_tool|mobile_app|api_service|mono_repo|documentation|data_pipeline|machine_learning|unknown",
    "key_directories": ["src", "tests", "docs", etc],
    "observations": ["observations RELEVANT to the objective/task"],
    "detected_frameworks": ["React", "Express", "Tokio", etc],
    "build_systems": ["npm", "cargo", "make", etc]
  },
  "recommendations": [
    "Next steps SPECIFIC to achieving the current task",
    "What files/dirs to explore next for the objective",
    // focused on the goal, not generic advice
  ]
}

When you receive a shielded reference (file too large):
- You'll get a summary and the file path
- Use the MCP tools to explore it like you would with `head`, `tail`, `grep` in a terminal
- Focus on understanding the overall structure first
- Sample intelligently to build your analysis

Remember: You're the intelligent intermediary between raw `ls` output and an LLM that needs to understand this codebase. Make it actionable."#;

/// Detect project type from marker files
fn detect_project_type_from_markers(marker_files: &[String]) -> ProjectType {
    for file in marker_files {
        match file.as_str() {
            "Cargo.toml" => return ProjectType::Library,
            "package.json" => return ProjectType::WebApp,
            "pyproject.toml" | "setup.py" => return ProjectType::Library,
            "go.mod" => return ProjectType::Library,
            "pom.xml" | "build.gradle" => return ProjectType::Library,
            _ => {}
        }
    }
    ProjectType::Unknown
}

/// Detect project type from directory structure
fn detect_project_type_from_dirs(top_dirs: &[String]) -> ProjectType {
    if top_dirs.iter().any(|d| d.contains("src") || d.contains("lib")) {
        ProjectType::Library
    } else if top_dirs.iter().any(|d| d.contains("docs")) {
        ProjectType::Documentation
    } else {
        ProjectType::Unknown
    }
}

/// Detect project type using marker files first, falling back to directory structure
fn detect_project_type(marker_files: &[String], top_dirs: &[String]) -> ProjectType {
    // Try marker-based detection first (most reliable)
    let marker_result = detect_project_type_from_markers(marker_files);

    // Fall back to directory structure if markers didn't give us an answer
    if matches!(marker_result, ProjectType::Unknown) {
        detect_project_type_from_dirs(top_dirs)
    } else {
        marker_result
    }
}

/// Detect build systems from file paths
fn detect_build_systems<'a, I>(files: I) -> Vec<String>
where
    I: IntoIterator<Item = &'a String>,
{
    files
        .into_iter()
        .filter_map(|f| {
            if f.ends_with("Cargo.toml") {
                Some("Cargo".to_string())
            } else if f.ends_with("package.json") {
                Some("npm/yarn/pnpm".to_string())
            } else if f.ends_with("Makefile") {
                Some("Make".to_string())
            } else if f.ends_with("CMakeLists.txt") {
                Some("CMake".to_string())
            } else if f.ends_with("pom.xml") {
                Some("Maven".to_string())
            } else if f.ends_with("build.gradle") {
                Some("Gradle".to_string())
            } else {
                None
            }
        })
        .collect()
}

/// Agent for post-processing file system output for LLM consumption
pub struct FileSystemAgent {
    agent: Agent<ollama::CompletionModel>,
    shield: Shield,
}

impl FileSystemAgent {
    /// Create a new agent using Ollama
    pub fn new() -> Result<Self> {
        let client = ollama::Client::new();
        
        let agent = client
            .agent("devstral:latest")
            .preamble(SYSTEM_PROMPT)
            .tool(HeadShieldedFile)
            .tool(TailShieldedFile)
            .tool(GrepShieldedFile)
            .tool(SampleShieldedFile)
            .build();
            
        let shield = Shield::new()?;
            
        Ok(Self { agent, shield })
    }
    
    /// Process file system data with automatic shielding
    pub async fn process(&self,
        objective: String,
        current_task: String,
        instructions: String,
        metadata: ScanMetadata,
        jsonl_data: Vec<Value>
    ) -> Result<AgentResponse> {
        // Apply shield BEFORE sending to agent
        let shield_result = self.shield.process(jsonl_data, Some(&metadata.root_path))?;

        // Calculate statistics BEFORE consuming shield_result
        let (stats, key_files, structure_analysis) = match &shield_result {
            ShieldResult::PassThrough(data) => {
                // Calculate full statistics from in-memory data
                let mut stats = FileStatistics {
                    total_files: 0,
                    total_dirs: 0,
                    total_size_bytes: 0,
                    primary_language: None,
                    file_type_distribution: std::collections::HashMap::new(),
                };

                let mut key_files = Vec::new();
                let mut marker_files = Vec::new();
                let mut top_dirs = std::collections::HashSet::new();
                let root = std::path::Path::new(&metadata.root_path);

                for entry in data {
                    // Count files vs directories
                    if let Some(type_str) = entry.get("type").and_then(|v| v.as_str()) {
                        if type_str.contains("Directory") {
                            stats.total_dirs += 1;

                            // Collect top-level directories
                            if let Some(path) = entry.get("path").and_then(|v| v.as_str())
                                && let Some(name) = entry.get("name").and_then(|v| v.as_str())
                            {
                                let entry_path = std::path::Path::new(path);
                                if let Some(parent) = entry_path.parent()
                                    && parent == root
                                {
                                    top_dirs.insert(name.to_string());
                                }
                            }
                        } else {
                            stats.total_files += 1;
                        }
                    }

                    // Sum sizes
                    if let Some(size) = entry.get("size").and_then(|v| v.as_u64()) {
                        stats.total_size_bytes += size as usize;
                    }

                    // Extract file extension and count
                    if let Some(path) = entry.get("path").and_then(|v| v.as_str())
                        && let Some(ext) = std::path::Path::new(path).extension()
                    {
                        let ext_str = ext.to_string_lossy().to_string();
                        *stats.file_type_distribution.entry(ext_str).or_insert(0) += 1;
                    }

                    // Identify key files and marker files
                    if let Some(name) = entry.get("name").and_then(|v| v.as_str())
                        && matches!(
                            name,
                            "README.md" | "Cargo.toml" | "package.json" | "main.rs" | "lib.rs"
                                | "setup.py" | "pyproject.toml" | "go.mod" | "Makefile"
                                | "CMakeLists.txt" | "pom.xml" | "build.gradle"
                        )
                    {
                        marker_files.push(name.to_string());
                        if let Some(path) = entry.get("path").and_then(|v| v.as_str()) {
                            key_files.push(path.to_string());
                        }
                    }
                }

                // Convert HashSet to Vec for compatibility with detection function
                let top_dirs: Vec<String> = top_dirs.into_iter().collect();

                // Determine primary language from most common extension
                stats.primary_language = stats.file_type_distribution
                    .iter()
                    .max_by_key(|(_, count)| *count)
                    .map(|(ext, _)| match ext.as_str() {
                        "rs" => "Rust",
                        "py" => "Python",
                        "js" | "jsx" => "JavaScript",
                        "ts" | "tsx" => "TypeScript",
                        "go" => "Go",
                        "c" | "h" => "C",
                        "cpp" | "cc" | "cxx" | "hpp" => "C++",
                        "java" => "Java",
                        "rb" => "Ruby",
                        "php" => "PHP",
                        "swift" => "Swift",
                        "kt" | "kts" => "Kotlin",
                        _ => ext.as_str(),
                    }.to_string());

                // Detect project type from marker files and directory structure
                let project_type = detect_project_type(&marker_files, &top_dirs);

                // Build structure analysis for PassThrough
                let build_systems = detect_build_systems(&key_files);

                let structure_analysis = StructureAnalysis {
                    project_type,
                    key_directories: top_dirs,
                    observations: vec![],
                    detected_frameworks: vec![],
                    build_systems,
                };

                (stats, key_files, structure_analysis)
            }

            ShieldResult::FileShielded { summary, .. } => {
                // For large datasets, use Shield's pre-calculated statistics

                // Parse file_types_summary to count files vs directories
                let mut total_files = 0;
                let mut total_dirs = 0;

                for (type_str, count) in &summary.file_types_summary {
                    if type_str.contains("Directory") {
                        total_dirs += count;
                    } else {
                        total_files += count;
                    }
                }

                let stats = FileStatistics {
                    total_files,
                    total_dirs,
                    total_size_bytes: summary.total_size_bytes,
                    primary_language: None,
                    file_type_distribution: summary.file_types_summary.clone(),
                };

                // Use marker files from summary (calculated during shield processing)
                let key_files = summary.marker_files.clone();

                // Detect project type from marker files and directory structure
                let project_type = detect_project_type(&summary.marker_files, &summary.top_level_dirs);

                // Build structure analysis for FileShielded
                let build_systems = detect_build_systems(&summary.marker_files);

                let structure_analysis = StructureAnalysis {
                    project_type,
                    key_directories: summary.top_level_dirs.clone(),
                    observations: vec![],
                    detected_frameworks: vec![],
                    build_systems,
                };

                (stats, key_files, structure_analysis)
            }
        };

        // Build the structured input (consumes shield_result)
        let agent_input = match shield_result {
            ShieldResult::PassThrough(data) => {
                AgentInput {
                    objective,
                    current_task,
                    instructions,
                    metadata,
                    data: AgentData::Direct { entries: data },
                }
            },
            ShieldResult::FileShielded { path, summary, .. } => {
                AgentInput {
                    objective,
                    current_task,
                    instructions,
                    metadata,
                    data: AgentData::Shielded {
                        file_path: path.to_string_lossy().to_string(),
                        summary: Box::new(summary),
                        mcp_instructions: McpInstructions {
                            tool_name: "mcp_file_operations".to_string(),
                            available_commands: vec![
                                "head <path> <lines>".to_string(),
                                "tail <path> <lines>".to_string(),
                                "grep <path> <pattern>".to_string(),
                                "sample <path> <count>".to_string(),
                            ],
                            usage_examples: vec![
                                "head /tmp/sap/abc123.jsonl 100".to_string(),
                                "grep /tmp/sap/abc123.jsonl \"src/\"".to_string(),
                            ],
                        },
                    },
                }
            }
        };
        
        // Send structured input to agent
        let prompt = format!(
            "Analyze the following file system data and provide a structured response:\n\n{}",
            serde_json::to_string_pretty(&agent_input)?
        );
        
        let response = self.agent.prompt(&prompt).await?;

        // Try to parse agent response as structured JSON
        // The agent is instructed to return JSON matching AgentResponse structure
        let (agent_summary, agent_observations, agent_frameworks, agent_recommendations) = 
            match serde_json::from_str::<AgentResponse>(&response) {
                Ok(parsed) => {
                    // Successfully parsed JSON response from agent
                    (
                        parsed.summary,
                        parsed.structure_analysis.observations,
                        parsed.structure_analysis.detected_frameworks,
                        parsed.recommendations,
                    )
                }
                Err(_) => {
                    // Failed to parse JSON - use raw response as summary
                    // This maintains backward compatibility if agent doesn't return valid JSON
                    (response, vec![], vec![], vec![])
                }
            };

        // Merge agent's insights with our pre-calculated data
        // We trust our statistics (calculated from actual data) but use agent's analysis
        let mut final_structure = structure_analysis;
        final_structure.observations = agent_observations;
        final_structure.detected_frameworks = agent_frameworks;

        Ok(AgentResponse {
            summary: agent_summary,
            statistics: stats,
            key_files,
            structure_analysis: final_structure,
            recommendations: agent_recommendations,
        })
    }
}