use dotenv::dotenv;
use serenity::Client;
use serenity::all::GatewayIntents;
use serenity::prelude::TypeMapKey;
use std::collections::HashMap;

use crate::{
    config::{Config, load_config},
    handlers::Handler,
};

mod config;
mod handlers;

struct VoiceSessionTime;
struct UserVoiceSession {
    total_time: u64,
    last_joined_timestamp: u64,
}

impl TypeMapKey for VoiceSessionTime {
    type Value = HashMap<u64, UserVoiceSession>;
}

#[tokio::main]
async fn main() {
    env_logger::init();
    dotenv().ok();

    let token = std::env::var("DISCORD_TOKEN").expect("Failed to find discord token");
    let config = load_config().expect("Failed to load config");
    let intents = GatewayIntents::GUILDS | GatewayIntents::GUILD_VOICE_STATES;
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Failed to create client");

    init_voice_session(&client, config).await;
    client.start().await.expect("Failed to start client");
}

async fn init_voice_session(client: &Client, config: Config) {
    let mut data = client.data.write().await;

    data.insert::<VoiceSessionTime>(HashMap::default());
    data.insert::<Config>(config);
}
