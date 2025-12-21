use serenity::all::{ChannelId, ChannelType, Context, CreateChannel, GuildChannel, VoiceState};
use tracing::{debug, warn};

use crate::{config::Config, error::Result, voice_stats::VoiceStats};

const DEFAULT_CHANNEL_NAME: &str = "channel";

#[derive(Debug)]
struct VoiceStateTransition<'a> {
    stats: &'a VoiceStats,
    ctx: &'a Context,
    config: &'a Config,
    old_channel: Option<ChannelId>,
    new_channel: Option<ChannelId>,
    old_state: Option<VoiceState>,
    new_state: VoiceState,
}

impl<'a> VoiceStateTransition<'a> {
    pub fn new(
        stats: &'a VoiceStats,
        ctx: &'a Context,
        config: &'a Config,
        old: Option<VoiceState>,
        new: VoiceState,
    ) -> Self {
        let old_channel = old.as_ref().and_then(|s| s.channel_id);
        let new_channel = new.channel_id;

        Self {
            stats,
            ctx,
            config,
            old_channel,
            new_channel,
            old_state: old,
            new_state: new,
        }
    }

    pub async fn handle(&self) -> Result<()> {
        if self.is_joining() && !self.is_joining_afk() {
            if self.is_joining_generator() {
                self.handle_connection().await?;
            }

            self.stats.user_joined(self.new_state.user_id).await;
        } else if self.is_leaving() && !self.is_leaving_afk() {
            self.handle_disconnection().await?;

            if let Some(old_state) = &self.old_state
                && let Some(member) = &old_state.member
            {
                self.stats.user_left(&member.user).await?;
            };
        } else if self.is_moving() {
            self.handle_move().await?;
        }

        Ok(())
    }

    async fn handle_connection(&self) -> Result<()> {
        debug!("Handling connection for user {}", self.new_state.user_id);

        let Some(channel_id) = self.new_state.channel_id else {
            return Ok(());
        };

        let Some(guild_id) = self.new_state.guild_id else {
            return Ok(());
        };

        if channel_id != self.config.generator.channel_id {
            return Ok(());
        }

        let Some(guild_channel) = channel_id.to_channel(&self.ctx).await?.guild() else {
            warn!("Could not get guild channel for {}", channel_id);
            return Ok(());
        };

        let Some(parent_id) = guild_channel.parent_id else {
            warn!("Could not get parent_id for channel {}", channel_id);
            return Ok(());
        };

        let name = self
            .config
            .drop_rates
            .get_random_drop(&self.config.channels)
            .map_or(DEFAULT_CHANNEL_NAME, |s| s.as_str());

        let builder = CreateChannel::new(name)
            .category(parent_id)
            .kind(ChannelType::Voice);

        let new_channel = guild_id.create_channel(&self.ctx, builder).await?;

        guild_id
            .move_member(&self.ctx, self.new_state.user_id, new_channel.id)
            .await?;

        Ok(())
    }

    async fn handle_disconnection(&self) -> Result<()> {
        debug!("Handling disconnection for user {}", self.new_state.user_id);

        let Some(ref old_state) = self.old_state else {
            return Ok(());
        };

        let Some(channel_id) = old_state.channel_id else {
            return Ok(());
        };

        if channel_id == self.config.generator.channel_id {
            return Ok(());
        }

        let Some(guild_channel) = channel_id.to_channel(&self.ctx).await?.guild() else {
            return Ok(());
        };

        if guild_channel.parent_id != Some(self.config.generator.parent_id) {
            return Ok(());
        }

        if channel_is_empty(self.ctx, &guild_channel)? {
            channel_id.delete(&self.ctx).await?;
        }

        Ok(())
    }

    async fn handle_move(&self) -> Result<()> {
        debug!("Handling a move for user {}", self.new_state.user_id);

        if self.is_joining_generator() {
            self.handle_disconnection().await?;
            self.handle_connection().await?;

            if self.is_leaving_afk() {
                self.stats.user_joined(self.new_state.user_id).await;
            }
        } else if self.is_joining_afk() {
            if let Some(ref member) = self.new_state.member {
                self.handle_disconnection().await?;
                self.stats.user_left(&member.user).await?;
            }
        } else if self.is_leaving_afk() {
            self.stats.user_joined(self.new_state.user_id).await;
        }

        Ok(())
    }

    fn is_joining(&self) -> bool {
        self.old_channel.is_none() && self.new_channel.is_some()
    }

    fn is_leaving(&self) -> bool {
        self.old_channel.is_some() && self.new_channel.is_none()
    }

    fn is_moving(&self) -> bool {
        matches!((self.old_channel, self.new_channel), (Some(old), Some(new)) if old != new)
    }

    fn is_joining_generator(&self) -> bool {
        self.new_channel == Some(self.config.generator.channel_id)
    }

    fn is_joining_afk(&self) -> bool {
        self.new_channel == Some(self.config.generator.afk_channel_id)
    }

    fn is_leaving_afk(&self) -> bool {
        self.old_channel == Some(self.config.generator.afk_channel_id)
    }
}

pub async fn voice_state_update(
    stats: &VoiceStats,
    ctx: Context,
    old: Option<VoiceState>,
    new: VoiceState,
) -> Result<()> {
    let data = ctx.data.read().await;
    let config = data.get::<Config>().unwrap();
    let transition = VoiceStateTransition::new(stats, &ctx, config, old, new);

    transition.handle().await
}

fn channel_is_empty(ctx: &Context, channel: &GuildChannel) -> Result<bool> {
    Ok(channel.members(ctx)?.is_empty())
}
