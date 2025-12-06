use log::{info, warn};
use serenity::{
    all::{Context, EventHandler, Ready, VoiceState},
    async_trait,
};

use crate::voice_stats::VoiceStats;

pub mod voice_state_update;

pub struct Handler {
    voice_stats: VoiceStats,
}

impl Handler {
    pub fn new(voice_stats: VoiceStats) -> Self {
        Self { voice_stats }
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        info!("Logged in as {}", ready.user.tag())
    }

    async fn voice_state_update(&self, ctx: Context, old: Option<VoiceState>, new: VoiceState) {
        match voice_state_update::voice_state_update(self.voice_stats, ctx, old, new).await {
            Ok(_) => info!(""),
            Err(e) => warn!("Encountered error: {:?}", e),
        }
    }
}
