mod agent;
mod config;
mod tools;

use crate::{
    agent::Agent,
    config::{Config, resolve_config_path},
};
use std::{
    error::Error,
    io::{self, Write},
};

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

        agent.chat(input).await?;
    }

    Ok(())
}
