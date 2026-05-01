use crate::{
    agent::Agent,
    app::{App, Message},
};

/// 处理用户发送消息：先显示 User 消息，再调用 Agent，最后显示回复或错误。
pub async fn handle_user_message(app: &mut App, agent: &mut Agent, content: String) {
    app.push_message(Message::User(content.clone()));

    match agent.chat(&content).await {
        Ok(result) => app.push_message(Message::Assistant(result)),
        Err(e) => app.push_message(Message::Error(format!("Error: {}", e))),
    }
}
