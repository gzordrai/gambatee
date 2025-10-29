use log::{info, warn};
use serenity::{
    all::{Context, EventHandler, Ready, VoiceState},
    async_trait,
};

pub mod voice_state_update;

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        info!("Logged in as {}", ready.user.tag())
    }

    async fn voice_state_update(&self, ctx: Context, old: Option<VoiceState>, new: VoiceState) {
        match voice_state_update::voice_state_update(ctx, old, new).await {
            Ok(_) => info!(""),
            Err(e) => warn!("Encountered error: {:?}", e),
        }
    }
}
