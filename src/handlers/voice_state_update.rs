use serenity::all::{ChannelId, ChannelType, Context, CreateChannel, GuildChannel, VoiceState};
use tracing::{debug, info, warn};

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
        debug!(
            "Processing voice state - Old channel: {:?}, New channel: {:?}, User: {}",
            self.old_channel, self.new_channel, self.new_state.user_id
        );

        if self.is_joining() && !self.is_joining_afk() {
            info!(
                "User {} joined voice channel {:?}",
                self.new_state.user_id, self.new_state.channel_id
            );

            if self.is_joining_generator() {
                self.handle_connection().await?;
            }

            self.stats.user_joined(self.new_state.user_id).await;
        } else if self.is_leaving() {
            info!(
                "User {} left voice channel {:?}",
                self.new_state.user_id, self.old_channel
            );
            self.handle_disconnection().await?;
        } else if self.is_moving() {
            info!(
                "User {} moved from channel {:?} to {:?}",
                self.new_state.user_id, self.old_channel, self.new_channel
            );
            self.handle_move().await?;
        } else {
            debug!(
                "No significant voice state change for user {}",
                self.new_state.user_id
            );
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

        debug!(
            "User {} joined generator channel {}",
            self.new_state.user_id, channel_id
        );

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

        info!(
            "Creating new voice channel '{}' in category {}",
            name, parent_id
        );

        let builder = CreateChannel::new(name)
            .category(parent_id)
            .kind(ChannelType::Voice);

        let new_channel = guild_id.create_channel(&self.ctx, builder).await?;

        info!(
            "Moving user {} to new channel {}",
            self.new_state.user_id, new_channel.id
        );

        guild_id
            .move_member(&self.ctx, self.new_state.user_id, new_channel.id)
            .await?;

        debug!(
            "Successfully created and moved user to channel {}",
            new_channel.id
        );

        Ok(())
    }

    async fn handle_disconnection(&self) -> Result<()> {
        let Some(ref old_state) = self.old_state else {
            return Ok(());
        };

        let Some(channel_id) = old_state.channel_id else {
            return Ok(());
        };

        if channel_id == self.config.generator.channel_id {
            return Ok(());
        }

        debug!("Checking if channel {} should be deleted", channel_id);

        let Some(guild_channel) = channel_id.to_channel(&self.ctx).await?.guild() else {
            return Ok(());
        };

        if guild_channel.parent_id != Some(self.config.generator.parent_id) {
            debug!("Channel {} not in managed category, ignoring", channel_id);
            return Ok(());
        }

        if channel_is_empty(self.ctx, &guild_channel).await? {
            info!("Channel {} is empty, deleting it", channel_id);
            channel_id.delete(&self.ctx).await?;

            if let Some(member) = &old_state.member {
                self.stats.user_left(&member.user).await?;
            }

            debug!("Successfully deleted channel {}", channel_id);
        } else {
            debug!("Channel {} still has members, keeping it", channel_id);
        }

        Ok(())
    }

    async fn handle_move(&self) -> Result<()> {
        if self.is_joining_generator() {
            self.handle_disconnection().await?;
            self.handle_connection().await?;

            if self.is_leaving_afk() {
                self.stats.user_joined(self.new_state.user_id).await;
            }
        } else if self.is_joining_afk() {
            if let Some(ref member) = self.new_state.member {
                debug!("User {} moved to AFK channel", member.user.id);
                self.handle_disconnection().await?;
                self.stats.user_left(&member.user).await?;
            }
        } else if self.is_leaving_afk() {
            debug!("User {} left AFK channel", self.new_state.user_id);
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

async fn channel_is_empty(ctx: &Context, channel: &GuildChannel) -> Result<bool> {
    let members = channel.members(ctx)?;
    let is_empty = members.is_empty();

    debug!("Channel {} has {} members", channel.id, members.len());

    Ok(is_empty)
}
