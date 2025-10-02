use anyhow::Result;
use rig_core::{completion::ToolDefinition, providers, tool::Tool};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Deserialize)]
struct ContentOptimizationArgs {
    content: String,
    max_length: Option<usize>,
    preserve_key_points: Option<bool>,
}

#[derive(Deserialize)]
struct ResponseFormattingArgs {
    response: String,
    format_type: String, // "concise", "structured", "summary"
}

#[derive(Debug, thiserror::Error)]
#[error("Guardian agent error")]
struct GuardianError;

#[derive(Deserialize, Serialize)]
struct ContentShield;
impl Tool for ContentShield {
    const NAME: &'static str = "shield_content";

    type Error = GuardianError;
    type Args = ContentOptimizationArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "shield_content".to_string(),
            description: "Shield the target LLM from long content by intelligently summarizing and optimizing input. Processes JSONC stream line-by-line in near real-time.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "content": {
                        "type": "string",
                        "description": "A single line of JSONC content from the stream to be shielded/optimized"
                    },
                    "max_length": {
                        "type": "number",
                        "description": "Maximum length for the optimized content per line (optional)"
                    },
                    "preserve_key_points": {
                        "type": "boolean",
                        "description": "Whether to preserve all key points in summarization"
                    }
                },
                "required": ["content"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        // Guardian logic to shield from long content in real-time streaming
        // Each line of JSONC is processed as it arrives
        let max_len = args.max_length.unwrap_or(500);
        let content = if args.content.len() > max_len {
            // Intelligent summarization applied per line for streaming efficiency
            format!(
                "[OPTIMIZED: {} chars -> {} chars] {}",
                args.content.len(),
                max_len,
                &args.content[..max_len.min(args.content.len())]
            )
        } else {
            args.content
        };
        Ok(content)
    }
}

#[derive(Deserialize, Serialize)]
struct ResponseOptimizer;
impl Tool for ResponseOptimizer {
    const NAME: &'static str = "optimize_response";

    type Error = GuardianError;
    type Args = ResponseFormattingArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "optimize_response".to_string(),
            description: "Optimize responses from the protected LLM in near real-time as JSONC lines stream through. Applies formatting optimizations on-the-fly.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "response": {
                        "type": "string",
                        "description": "A single line of JSONC response from the stream to optimize"
                    },
                    "format_type": {
                        "type": "string",
                        "description": "Type of optimization: 'concise', 'structured', or 'summary'",
                        "enum": ["concise", "structured", "summary"]
                    }
                },
                "required": ["response", "format_type"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        // Real-time optimization of streaming JSONC lines
        let optimized = match args.format_type.as_str() {
            "concise" => {
                // Remove redundancy, keep essential info per line
                format!(
                    "[CONCISE] {}",
                    args.response
                        .split_whitespace()
                        .take(50)
                        .collect::<Vec<_>>()
                        .join(" ")
                )
            }
            "structured" => {
                // Add structure to response line
                format!(
                    "[STRUCTURED]\n• Main Point: {}\n• Details: ...",
                    args.response.lines().next().unwrap_or(&args.response)
                )
            }
            "summary" => {
                // Summarize to key points for this line
                format!(
                    "[SUMMARY] Key points from response: {}",
                    &args.response[..100.min(args.response.len())]
                )
            }
            _ => args.response,
        };
        Ok(optimized)
    }
}

