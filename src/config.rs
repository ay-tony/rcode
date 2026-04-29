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
        let mut config: Config =
            toml::from_str(&content).map_err(|e| format!("Config file syntax error: {}", e))?;

        // system_prompt 字段为相对于配置目录的文件路径，读取其内容
        let config_dir = std::path::Path::new(path)
            .parent()
            .ok_or("Invalid config file path")?;
        let prompt_path = config_dir.join(&config.agent.system_prompt);
        let prompt_content = std::fs::read_to_string(&prompt_path)
            .map_err(|e| format!("Error reading system prompt file '{}': {}", prompt_path.display(), e))?;
        config.agent.system_prompt = prompt_content;

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
        let tmpdir = tempfile::tempdir().unwrap();

        let prompt_path = tmpdir.path().join("default_system_prompt.md");
        std::fs::write(&prompt_path, "You are a test assistant.").unwrap();

        let config_path = tmpdir.path().join("config.toml");
        std::fs::write(
            &config_path,
            r#"
[llm]
api_key = "test-key"
api_base = "https://test.com"

[agent]
model = "test-model"
system_prompt = "default_system_prompt.md"
"#,
        )
        .unwrap();

        let config = Config::from_file(config_path.to_str().unwrap()).unwrap();
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
