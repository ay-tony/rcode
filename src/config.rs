use serde::Deserialize;
use std::{env, error::Error, io::Write};

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
    // 优先用当前目录的 .rcode/config.toml
    if std::path::Path::new(".rcode/config.toml").exists() {
        return Ok(".rcode/config.toml".to_string());
    }

    // 回退到 ~/.rcode/config.toml
    let home = env::var("HOME")?;
    let global = format!("{}/.rcode/config.toml", home);
    if std::path::Path::new(&global).exists() {
        return Ok(global);
    }

    Err("Failed to find .rcode/config.toml. Please create it in current directory or at ~/.rcode/config.toml as global".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_config() {
        let mut tmpfile = tempfile::NamedTempFile::new().unwrap();
        writeln!(
            tmpfile,
            r#"
[llm]
api_key = "test-key"
api_base = "https://test.com"

[agent]
model = "test-model"
system_prompt = "You are a test assistant."
"#
        )
        .unwrap();

        let config = Config::from_file(tmpfile.path().to_str().unwrap()).unwrap();
        assert_eq!(config.llm.api_key, "test-key");
        assert_eq!(config.llm.api_base, "https://test.com");
        assert_eq!(config.agent.model, "test-model");
        assert_eq!(config.agent.system_prompt, "You are a test assistant.");
    }

    #[test]
    fn parse_invalid_config() {
        let config = Config::from_file("./invalid_config_path");
        assert!(config.is_err());
    }
}
