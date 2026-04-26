use async_openai::{
    Client,
    config::OpenAIConfig,
    types::chat::{
        ChatCompletionMessageToolCall, ChatCompletionMessageToolCallChunk,
        ChatCompletionMessageToolCalls, ChatCompletionRequestAssistantMessage,
        ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestMessage,
        ChatCompletionRequestSystemMessage, ChatCompletionRequestToolMessageArgs,
        ChatCompletionRequestToolMessageContent, ChatCompletionRequestUserMessage,
        ChatCompletionTool, ChatCompletionTools, CreateChatCompletionRequestArgs, FinishReason,
        FunctionCall, FunctionObjectArgs,
    },
};
use futures::StreamExt;
use serde::Deserialize;
use serde_json::json;
use std::{
    collections::HashMap,
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
                .stream(true)
                .build()?;

            let mut stream = self.client.chat().create_stream(request).await?;
            let mut lock = io::stdout().lock();

            let mut full_content = String::new();
            let mut tool_call_chunks: Vec<ChatCompletionMessageToolCallChunk> = Vec::new();

            while let Some(chunk) = stream.next().await {
                let chunk = chunk?;

                if let Some(choice) = chunk.choices.first() {
                    // 如果是对话
                    if let Some(text) = &choice.delta.content {
                        write!(lock, "{}", text)?;
                        full_content.push_str(text);
                    }

                    // 如果是工具调用
                    if let Some(calls) = &choice.delta.tool_calls {
                        tool_call_chunks.extend(calls.clone());
                    }

                    // 检查结束原因
                    match &choice.finish_reason {
                        // 对话结束
                        Some(FinishReason::Stop) => {
                            if !full_content.ends_with('\n') {
                                write!(lock, "\n")?;
                            }
                            self.messages.push(
                                ChatCompletionRequestAssistantMessage::from(full_content.clone())
                                    .into(),
                            );
                            return Ok(full_content);
                        }

                        // 工具调用结束
                        Some(FinishReason::ToolCalls) => {
                            write!(lock, "\n")?;
                            let tool_calls = self.build_tool_calls(tool_call_chunks)?;

                            self.messages.push(
                                ChatCompletionRequestAssistantMessageArgs::default()
                                    .tool_calls(tool_calls.clone())
                                    .build()?
                                    .into(),
                            );

                            // 执行每个工具
                            for tool_call in tool_calls {
                                let (result, id) = self.execute_tool(&tool_call)?;

                                self.messages.push(
                                    ChatCompletionRequestToolMessageArgs::default()
                                        .content(ChatCompletionRequestToolMessageContent::Text(
                                            result,
                                        ))
                                        .tool_call_id(id)
                                        .build()?
                                        .into(),
                                );
                            }
                            break;
                        }

                        _ => {}
                    }

                    lock.flush()?;
                }
            }
        }
    }

    fn build_tool_calls(
        &self,
        chunks: Vec<ChatCompletionMessageToolCallChunk>,
    ) -> Result<Vec<ChatCompletionMessageToolCalls>, Box<dyn Error>> {
        // index -> (id, name, arguments)
        let mut groups: HashMap<u32, (Option<String>, Option<String>, String)> = HashMap::new();

        for chunk in chunks {
            let entry = groups
                .entry(chunk.index)
                .or_insert((None, None, String::new()));
            if let Some(id) = chunk.id {
                entry.0 = Some(id);
            }
            if let Some(func) = chunk.function {
                if let Some(name) = func.name {
                    entry.1 = Some(name);
                }
                if let Some(args) = func.arguments {
                    entry.2.push_str(&args);
                }
            }
        }

        let mut result = Vec::new();
        let mut indices: Vec<_> = groups.keys().cloned().collect();
        indices.sort();

        for idx in indices {
            let (id, name, args) = groups
                .remove(&idx)
                .ok_or("Failed to remove item from hashmap")?;
            let id = id.ok_or("Missing tool call id")?;
            let name = name.ok_or("Missing tool call name")?;

            result.push(ChatCompletionMessageToolCalls::Function(
                ChatCompletionMessageToolCall {
                    id,
                    function: FunctionCall {
                        name,
                        arguments: args,
                    },
                },
            ));
        }

        Ok(result)
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

        agent.chat(&input).await?;
    }

    Ok(())
}
