use crate::config_file::Config;
use crate::flags::Configurable;
use crate::app::Cli;

/// Flag to enable LLM-friendly JSON Lines output
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct LlmOutput {
    pub enabled: bool,
    pub objective: Option<String>,
    pub current_task: Option<String>,
}

impl LlmOutput {
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

impl Configurable<Self> for LlmOutput {
    /// Get config from CLI arguments
    fn from_cli(cli: &Cli) -> Option<Self> {
        if cli.llm {
            Some(Self {
                enabled: true,
                objective: cli.objective.clone(),
                current_task: cli.current_task.clone(),
            })
        } else {
            None
        }
    }

    /// Get config from config file  
    fn from_config(config: &Config) -> Option<Self> {
        config.llm.map(|enabled| Self {
            enabled,
            objective: None,
            current_task: None,
        })
    }
}