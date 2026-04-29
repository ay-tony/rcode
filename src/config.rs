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
        let prompt_content = std::fs::read_to_string(&prompt_path).map_err(|e| {
            format!(
                "Error reading system prompt file '{}': {}",
                prompt_path.display(),
                e
            )
        })?;
        config.agent.system_prompt = prompt_content;

        // 如果当前工程根目录下存在 AGENT.md，追加到 system_prompt
        let agent_md_path = std::path::Path::new("./AGENT.md");
        if agent_md_path.exists() {
            let agent_md = std::fs::read_to_string(agent_md_path)
                .map_err(|e| format!("Error reading './AGENT.md': {}", e))?;
            if !agent_md.is_empty() {
                config
                    .agent
                    .system_prompt
                    .push_str("\n\nPlease follow the project instruction:\n\n");
                config.agent.system_prompt.push_str(&agent_md);
            }
        }

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
    use std::sync::Mutex;

    static CWD_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn parse_valid_config() {
        let _guard = CWD_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let tmpdir = tempfile::tempdir().unwrap();
        let original = std::env::current_dir().unwrap();
        std::env::set_current_dir(&tmpdir).unwrap();

        let rcode_dir = tmpdir.path().join(".rcode");
        std::fs::create_dir(&rcode_dir).unwrap();

        let prompt_path = rcode_dir.join("default_system_prompt.md");
        std::fs::write(&prompt_path, "You are a test assistant.").unwrap();

        let config_path = rcode_dir.join("config.toml");
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

        let config = Config::from_file(".rcode/config.toml").unwrap();
        assert_eq!(config.llm.api_key, "test-key");
        assert_eq!(config.llm.api_base, "https://test.com");
        assert_eq!(config.agent.model, "test-model");
        assert_eq!(config.agent.system_prompt, "You are a test assistant.");

        std::env::set_current_dir(original).unwrap();
    }

    #[test]
    fn parse_config_with_agent_md() {
        let _guard = CWD_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let tmpdir = tempfile::tempdir().unwrap();
        let original = std::env::current_dir().unwrap();
        std::env::set_current_dir(&tmpdir).unwrap();

        let rcode_dir = tmpdir.path().join(".rcode");
        std::fs::create_dir(&rcode_dir).unwrap();

        let prompt_path = rcode_dir.join("default_system_prompt.md");
        std::fs::write(&prompt_path, "You are a test assistant.").unwrap();

        let agent_md_path = tmpdir.path().join("AGENT.md");
        std::fs::write(&agent_md_path, "Always write tests first.").unwrap();

        let config_path = rcode_dir.join("config.toml");
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

        let config = Config::from_file(".rcode/config.toml").unwrap();
        assert_eq!(config.llm.api_key, "test-key");
        assert_eq!(config.llm.api_base, "https://test.com");
        assert_eq!(config.agent.model, "test-model");
        assert_eq!(
            config.agent.system_prompt,
            "You are a test assistant.\n\nPlease follow the project instruction:\n\nAlways write tests first."
        );

        std::env::set_current_dir(original).unwrap();
    }

    #[test]
    fn parse_invalid_config() {
        let config = Config::from_file("./invalid_config_path");
        assert!(config.is_err());
    }
}
