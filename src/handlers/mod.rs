use std::sync::Arc;

use crate::voice_stats::VoiceStats;
use serenity::{
    all::{Context, EventHandler, Ready, VoiceState},
    async_trait,
};
use tracing::{debug, info, warn};

pub mod voice_state_update;

pub struct Handler {
    voice_stats: Arc<VoiceStats>,
}

impl Handler {
    pub fn new(voice_stats: Arc<VoiceStats>) -> Self {
        Self { voice_stats }
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        info!("Bot successfully logged in as {}", ready.user.tag());
        info!("Connected to {} guilds", ready.guilds.len());
        debug!("Bot ID: {}, Shard: {:?}", ready.user.id, ready.shard);
    }

    async fn voice_state_update(&self, ctx: Context, old: Option<VoiceState>, new: VoiceState) {
        debug!(
            "Voice state update - User: {} ({}), Guild: {:?}",
            new.user_id,
            new.member
                .as_ref()
                .map(|m| m.user.tag())
                .unwrap_or_else(|| "Unknown".to_string()),
            new.guild_id
        );

        if let Some(ref old_state) = old {
            debug!(
                "Old channel: {:?}, New channel: {:?}",
                old_state.channel_id, new.channel_id
            );
        } else {
            debug!("New channel: {:?} (no previous state)", new.channel_id);
        }

        match voice_state_update::voice_state_update(&self.voice_stats, ctx, old, new).await {
            Ok(_) => debug!("Voice state update processed successfully"),
            Err(e) => warn!("Encountered error: {:?}", e),
        }
    }
}
