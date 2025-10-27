use dotenv::dotenv;
use log::info;
use serenity::all::{
    ChannelType, Context, CreateChannel, EventHandler, GatewayIntents, GuildId, Ready, UserId,
    VoiceState,
};
use serenity::prelude::TypeMapKey;
use serenity::{Client, async_trait};
use std::collections::HashMap;

use crate::config::{Config, load_config};

mod config;

const DEFAULT_CHANNEL_NAME: &str = "channel";

struct Handler;
struct VoiceSessionTime;
struct UserVoiceSession {
    total_time: u64,
    last_joined_timestamp: u64,
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        info!("Logged in as {}", ready.user.tag())
    }

    async fn voice_state_update(&self, ctx: Context, old: Option<VoiceState>, new: VoiceState) {
        let mut data = ctx.data.write().await;
        let config = data.get_mut::<Config>().unwrap();

        if let (Some(channel_id), Some(guild_id)) = (new.channel_id, new.guild_id) {
            if let Ok(channel) = channel_id.to_channel(&ctx).await {
                if let Some(guild_channel) = channel.guild() {
                    if let Some(parent_id) = guild_channel.parent_id {
                        let name = config
                            .drop_rates
                            .get_random_drop(&config.channels)
                            .map_or(DEFAULT_CHANNEL_NAME, |s| s.as_str());

                        let builder = CreateChannel::new(name)
                            .category(parent_id)
                            .kind(ChannelType::Voice);

                        let channel = GuildId::new(guild_id.into())
                            .create_channel(&ctx, builder)
                            .await
                            .unwrap();

                        let _ = guild_id.move_member(&ctx, new.user_id, channel.id);
                    }
                }
            }
        }
    }
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
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::GUILD_VOICE_STATES;
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
