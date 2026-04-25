use async_openai::{
    Client,
    config::OpenAIConfig,
    types::chat::{
        ChatCompletionMessageToolCalls, ChatCompletionRequestAssistantMessage,
        ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestMessage,
        ChatCompletionRequestSystemMessage, ChatCompletionRequestToolMessageArgs,
        ChatCompletionRequestToolMessageContent, ChatCompletionRequestUserMessage,
        ChatCompletionTool, ChatCompletionTools, CreateChatCompletionRequestArgs,
        FunctionObjectArgs,
    },
};
use serde::Deserialize;
use serde_json::json;
use std::{
    env,
    error::Error,
    io::{self, Write},
};

#[derive(Deserialize)]
struct Config {
    llm: LlmConfig,
    agent: AgentConfig,
}

#[derive(Deserialize)]
struct LlmConfig {
    api_key: String,
    api_base: String,
}

#[derive(Deserialize)]
struct AgentConfig {
    model: String,
    system_prompt: String,
}

impl Config {
    fn from_file(path: &str) -> Result<Config, Box<dyn Error>> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Error reading config file '{}': {}", path, e))?;
        let config: Config =
            toml::from_str(&content).map_err(|e| format!("Config file syntax error: {}", e))?;
        Ok(config)
    }
}

struct Agent {
    client: Client<OpenAIConfig>,
    messages: Vec<ChatCompletionRequestMessage>,
    tools: Vec<ChatCompletionTools>,
    config: AgentConfig,
}

impl Agent {
    fn new(config: Config) -> Result<Self, Box<dyn Error>> {
        let openai_config = OpenAIConfig::new()
            .with_api_key(config.llm.api_key)
            .with_api_base(config.llm.api_base);
        let client = Client::with_config(openai_config);

        Ok(Self {
            client,
            messages: vec![
                ChatCompletionRequestSystemMessage::from(config.agent.system_prompt.clone()).into(),
            ],
            tools: vec![ChatCompletionTools::Function(ChatCompletionTool {
                function: FunctionObjectArgs::default()
                    .name("execute_command")
                    .description("Execute command in shell")
                    .parameters(json!({
                        "type": "object",
                        "properties": {
                            "command": {
                                "type": "string",
                                "description": "The shell command to execute",
                            }
                        },
                        "required": ["command"],
                        "additionalProperties": false
                    }))
                    .strict(true)
                    .build()?,
            })],
            config: config.agent,
        })
    }

    async fn chat(&mut self, input: &str) -> Result<String, Box<dyn Error>> {
        self.messages
            .push(ChatCompletionRequestUserMessage::from(input).into());

        loop {
            let request = CreateChatCompletionRequestArgs::default()
                .model(self.config.model.clone())
                .messages(self.messages.clone())
                .tools(self.tools.clone())
                .build()?;

            let response_message = self
                .client
                .chat()
                .create(request)
                .await?
                .choices
                .first()
                .ok_or("API return no choices")?
                .message
                .clone();

            if let Some(tool_calls) = response_message.tool_calls {
                self.messages.push(
                    ChatCompletionRequestAssistantMessageArgs::default()
                        .tool_calls(tool_calls.clone())
                        .build()?
                        .into(),
                );

                for tool_call in tool_calls {
                    let (result, id) = self.execute_tool(&tool_call)?;

                    self.messages.push(
                        ChatCompletionRequestToolMessageArgs::default()
                            .content(ChatCompletionRequestToolMessageContent::Text(result))
                            .tool_call_id(id)
                            .build()?
                            .into(),
                    );
                }

                continue;
            }

            let content = response_message
                .content
                .clone()
                .ok_or("response content is empty")?;

            self.messages
                .push(ChatCompletionRequestAssistantMessage::from(content.clone()).into());

            return Ok(content);
        }
    }

    fn clear(&mut self) {
        self.messages.truncate(1);
    }

    fn execute_tool(
        &self,
        tool_call: &ChatCompletionMessageToolCalls,
    ) -> Result<(String, String), Box<dyn Error>> {
        match tool_call {
            ChatCompletionMessageToolCalls::Function(function_tool) => {
                let args: serde_json::Value =
                    serde_json::from_str(&function_tool.function.arguments)?;

                match function_tool.function.name.as_str() {
                    "execute_command" => {
                        let command = args["command"].as_str().ok_or("Missing command")?;

                        let output = std::process::Command::new("sh")
                            .arg("-c")
                            .arg(command)
                            .output()?;

                        let result = if output.status.success() {
                            String::from_utf8_lossy(&output.stdout).to_string()
                        } else {
                            format!(
                                "Command failed with exit code {:?}:\n{}",
                                output.status.code(),
                                String::from_utf8_lossy(&output.stderr)
                            )
                        };

                        println!("[rcode] tool_call: {:?}", function_tool.function);
                        println!("[rcode] id: {}", function_tool.id);
                        println!("[rcode] result: {}", result.replace("\n", "\n[rcode] "));

                        Ok((result, function_tool.id.clone()))
                    }
                    _ => Err(format!("Unknown tool: {}", function_tool.function.name).into()),
                }
            }
            ChatCompletionMessageToolCalls::Custom(_) => {
                Err("Unsupported tool type: custom tool call".into())
            }
        }
    }
}

fn resolve_config_path() -> Result<String, Box<dyn Error>> {
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut agent = Agent::new(Config::from_file(&resolve_config_path()?)?)?;

    loop {
        print!("> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        if input == "/exit" {
            break;
        }

        if input == "/clear" {
            agent.clear();
            println!("[rcode] All messages cleared!");
            continue;
        }

        println!("{}", agent.chat(&input).await?);
    }

    Ok(())
}
