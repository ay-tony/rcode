use async_openai::{
    Client,
    config::OpenAIConfig,
    types::chat::{
        ChatCompletionRequestAssistantMessage, ChatCompletionRequestMessage,
        ChatCompletionRequestSystemMessage, ChatCompletionRequestUserMessage,
        CreateChatCompletionRequestArgs,
    },
};
use std::{
    env,
    error::Error,
    io::{self, Write},
};

struct Agent {
    client: Client<OpenAIConfig>,
    messages: Vec<ChatCompletionRequestMessage>,
}

impl Agent {
    fn new(api_key: &str) -> Result<Self, Box<dyn Error>> {
        let config = OpenAIConfig::new()
            .with_api_key(api_key)
            .with_api_base("https://api.kimi.com/coding/v1")
            .with_header("User-Agent", "claude-code/2.1.116")?;
        let client = Client::with_config(config);
        Ok(Self {
            client,
            messages: vec![
                ChatCompletionRequestSystemMessage::from("You are a helpful assistant.").into(),
            ],
        })
    }

    async fn chat(&mut self, input: &str) -> Result<String, Box<dyn Error>> {
        self.messages
            .push(ChatCompletionRequestUserMessage::from(input).into());

        let request = CreateChatCompletionRequestArgs::default()
            .model("kimi-for-coding")
            .messages(self.messages.clone())
            .build()?;

        let response = self.client.chat().create(request).await?;

        let content = response
            .choices
            .first()
            .ok_or("API return empty choices")?
            .message
            .content
            .clone()
            .ok_or("response content is empty")?;

        self.messages
            .push(ChatCompletionRequestAssistantMessage::from(content.clone()).into());

        Ok(content)
    }

    fn clear(&mut self) {
        self.messages.truncate(1);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let api_key = env::var("KIMI_API_KEY")?;
    let mut kimi = Agent::new(&api_key)?;

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
            kimi.clear();
            println!("all messages cleared!");
            continue;
        }

        println!("{}", kimi.chat(&input).await?);
    }

    Ok(())
}
