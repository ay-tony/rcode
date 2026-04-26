use serde::Deserialize;
use std::{env, error::Error};

#[derive(Deserialize)]
pub struct Config {
    pub llm: LlmConfig,
    pub agent: AgentConfig,
}

#[derive(Deserialize)]
pub struct LlmConfig {
    pub api_key: String,
    pub api_base: String,
}

#[derive(Deserialize)]
pub struct AgentConfig {
    pub model: String,
    pub system_prompt: String,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Config, Box<dyn Error>> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Error reading config file '{}': {}", path, e))?;
        let config: Config =
            toml::from_str(&content).map_err(|e| format!("Config file syntax error: {}", e))?;
        Ok(config)
    }
}

pub fn resolve_config_path() -> Result<String, Box<dyn Error>> {
    // 优先用当前目录的 .rcode.toml
    if std::path::Path::new(".rcode.toml").exists() {
        return Ok(".rcode.toml".to_string());
    }

    // 回退到 ~/.rcode.toml
    let home = env::var("HOME")?;
    let global = format!("{}/.rcode.toml", home);
    if std::path::Path::new(&global).exists() {
        return Ok(global);
    }

    Err("Failed to find .rcode.toml. Please create it in current directory or at ~/.rcode.toml as global.".into())
}
