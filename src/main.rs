use std::{env, io};
use std::{error::Error, io::Write};

use async_openai::{
    Client,
    config::OpenAIConfig,
    types::chat::{
        ChatCompletionRequestAssistantMessage, ChatCompletionRequestMessage,
        ChatCompletionRequestSystemMessage, ChatCompletionRequestUserMessage,
        CreateChatCompletionRequestArgs,
    },
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let api_key = env::var("KIMI_API_KEY").expect("环境变量 KIMI_API_KEY 未设置");

    let config = OpenAIConfig::new()
        .with_api_key(api_key)
        .with_api_base("https://api.kimi.com/coding/v1")
        .with_header("User-Agent", "claude-code/2.1.116")?;

    let client = Client::with_config(config);

    let mut messages: Vec<ChatCompletionRequestMessage> =
        vec![ChatCompletionRequestSystemMessage::from("You are a helpful assistant.").into()];

    loop {
        print!("> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input == "exit" {
            break;
        }

        if input.is_empty() {
            continue;
        }

        messages.push(ChatCompletionRequestUserMessage::from(input).into());

        let request = CreateChatCompletionRequestArgs::default()
            .model("kimi-for-coding")
            .messages(messages.clone())
            .build()?;

        let response = client.chat().create(request).await?;

        let content = response
            .choices
            .first()
            .ok_or("API 返回空 choices")?
            .message
            .content
            .clone()
            .ok_or("回复内容为空")?;

        println!("{}", content);

        messages.push(ChatCompletionRequestAssistantMessage::from(content).into());
    }

    Ok(())
}
