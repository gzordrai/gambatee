use dotenv::dotenv;
use serenity::all::{
    ChannelType, Context, CreateChannel, EventHandler, GatewayIntents, GuildId, Ready, VoiceState,
};
use serenity::{Client, async_trait};

mod config;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("Logged in as {}", ready.user.tag())
    }

    async fn voice_state_update(&self, ctx: Context, old: Option<VoiceState>, new: VoiceState) {
        if let (Some(channel_id), Some(guild_id)) = (new.channel_id, new.guild_id) {
            if let Ok(channel) = channel_id.to_channel(&ctx).await {
                if let Some(guild_channel) = channel.guild() {
                    if let Some(parent_id) = guild_channel.parent_id {
                        // Use rarety to take a random channel name instead of a this one
                        let builder = CreateChannel::new("name")
                            .category(parent_id)
                            .kind(ChannelType::Voice);

                        if let Err(e) = GuildId::new(guild_id.into())
                            .create_channel(&ctx, builder)
                            .await
                        {}
                    }
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let token = std::env::var("DISCORD_TOKEN").expect("Failed to find token");
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::GUILD_VOICE_STATES;
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Error creating client");

    if let Err(e) = client.start().await {
        println!("Client error: {e:?}");
    }
}
