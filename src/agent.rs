use crate::{
    config::{AgentConfig, Config},
    render::{clear_lines_above, count_lines_in_terminal, render_markdown},
    tools::execute_command,
};
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
        FunctionCall, FunctionCallStream, FunctionObjectArgs, FunctionType,
    },
};
use crossterm::terminal;
use futures::StreamExt;
use serde_json::json;
use std::{
    collections::HashMap,
    error::Error,
    io::{self, Write},
};

pub struct Agent {
    client: Client<OpenAIConfig>,
    messages: Vec<ChatCompletionRequestMessage>,
    tools: Vec<ChatCompletionTools>,
    config: AgentConfig,
}

impl Agent {
    pub fn new(config: Config) -> Result<Self, Box<dyn Error>> {
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

    pub async fn chat(&mut self, input: &str) -> Result<String, Box<dyn Error>> {
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
                                full_content += "\n";
                                writeln!(lock)?;
                            }
                            drop(lock);

                            let count = count_lines_in_terminal(
                                &full_content,
                                terminal::size()?.0 as usize,
                            )
                            .unwrap_or(0);
                            clear_lines_above(count);
                            render_markdown(&full_content);

                            self.messages.push(
                                ChatCompletionRequestAssistantMessage::from(full_content.clone())
                                    .into(),
                            );
                            return Ok(full_content);
                        }

                        // 工具调用结束
                        Some(FinishReason::ToolCalls) => {
                            writeln!(lock)?;
                            let tool_calls = Self::build_tool_calls(tool_call_chunks)?;

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

    pub fn clear(&mut self) {
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

                        let result = execute_command(command)?;

                        // println!("[rcode] tool_call: {:?}", function_tool.function);
                        // println!("[rcode] id: {}", function_tool.id);
                        // println!("[rcode] result: {}", result.replace("\n", "\n[rcode] "));

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_tool_call_from_chunks() {
        let chunks = vec![
            ChatCompletionMessageToolCallChunk {
                index: 0,
                id: Some("call_1".to_string()),
                r#type: Some(FunctionType::Function),
                function: Some(FunctionCallStream {
                    name: Some("execute_command".to_string()),
                    arguments: None,
                }),
            },
            ChatCompletionMessageToolCallChunk {
                index: 0,
                id: None,
                r#type: None,
                function: Some(FunctionCallStream {
                    name: None,
                    arguments: Some("{\"command\": \"ls\"}".to_string()),
                }),
            },
        ];

        let tool_calls = Agent::build_tool_calls(chunks);
        assert!(tool_calls.is_ok());

        let tool_calls = tool_calls.unwrap();
        assert_eq!(tool_calls.len(), 1);

        assert_eq!(
            tool_calls[0],
            ChatCompletionMessageToolCalls::Function(ChatCompletionMessageToolCall {
                id: "call_1".to_string(),
                function: FunctionCall {
                    name: "execute_command".to_string(),
                    arguments: "{\"command\": \"ls\"}".to_string(),
                },
            },)
        );
    }

    #[test]
    fn multiple_tool_calls_from_chunks() {
        let chunks = vec![
            ChatCompletionMessageToolCallChunk {
                index: 0,
                id: Some("call_1".to_string()),
                r#type: Some(FunctionType::Function),
                function: Some(FunctionCallStream {
                    name: Some("execute_command".to_string()),
                    arguments: None,
                }),
            },
            ChatCompletionMessageToolCallChunk {
                index: 0,
                id: None,
                r#type: None,
                function: Some(FunctionCallStream {
                    name: None,
                    arguments: Some("{\"command\": \"ls\"}".to_string()),
                }),
            },
            ChatCompletionMessageToolCallChunk {
                index: 1,
                id: Some("call_2".to_string()),
                r#type: Some(FunctionType::Function),
                function: Some(FunctionCallStream {
                    name: Some("execute_command".to_string()),
                    arguments: None,
                }),
            },
            ChatCompletionMessageToolCallChunk {
                index: 1,
                id: None,
                r#type: None,
                function: Some(FunctionCallStream {
                    name: None,
                    arguments: Some("{\"command\": \"ls".to_string()),
                }),
            },
            ChatCompletionMessageToolCallChunk {
                index: 1,
                id: None,
                r#type: None,
                function: Some(FunctionCallStream {
                    name: None,
                    arguments: Some(" -al\"}".to_string()),
                }),
            },
        ];

        let tool_calls = Agent::build_tool_calls(chunks);
        assert!(tool_calls.is_ok());

        let tool_calls = tool_calls.unwrap();
        assert_eq!(tool_calls.len(), 2);

        assert_eq!(
            tool_calls[0],
            ChatCompletionMessageToolCalls::Function(ChatCompletionMessageToolCall {
                id: "call_1".to_string(),
                function: FunctionCall {
                    name: "execute_command".to_string(),
                    arguments: "{\"command\": \"ls\"}".to_string(),
                },
            },)
        );
        assert_eq!(
            tool_calls[1],
            ChatCompletionMessageToolCalls::Function(ChatCompletionMessageToolCall {
                id: "call_2".to_string(),
                function: FunctionCall {
                    name: "execute_command".to_string(),
                    arguments: "{\"command\": \"ls -al\"}".to_string(),
                },
            },)
        );
    }
}
