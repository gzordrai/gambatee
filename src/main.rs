use std::{str::FromStr, sync::Arc};

use serenity::Client;
use serenity::all::GatewayIntents;

use crate::{config::Config, error::Result, handlers::Handler, voice_stats::VoiceStats};

mod config;
mod error;
mod handlers;
mod voice_stats;

const DEFAULT_CONFIG_PATH: &str = "/etc/gambatee/config.toml";

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let token = std::env::var("DISCORD_TOKEN")?;
    let url = std::env::var("DATABASE_URL")?;
    let config = Config::from_str(DEFAULT_CONFIG_PATH)?;

    let intents = GatewayIntents::GUILDS | GatewayIntents::GUILD_VOICE_STATES;
    let voice_stats = Arc::new(VoiceStats::new(&url).await?);

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler::new(voice_stats))
        .await?;

    init_voice_session(&client, config).await;
    client.start().await?;

    Ok(())
}

async fn init_voice_session(client: &Client, config: Config) {
    let mut data = client.data.write().await;

    data.insert::<Config>(config);
}
