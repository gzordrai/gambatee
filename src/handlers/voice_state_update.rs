use serenity::all::{ChannelType, Context, CreateChannel, GuildChannel, VoiceState};
use tracing::{debug, info, warn};

use crate::{config::Config, error::Result, voice_stats::VoiceStats};

const DEFAULT_CHANNEL_NAME: &str = "channel";

pub async fn voice_state_update(
    stats: &VoiceStats,
    ctx: Context,
    old: Option<VoiceState>,
    new: VoiceState,
) -> Result<()> {
    let data = ctx.data.read().await;
    let config = data.get::<Config>().unwrap();
    let old_channel = old.as_ref().and_then(|s| s.channel_id);
    let new_channel = new.channel_id;

    debug!(
        "Processing voice state - Old channel: {:?}, New channel: {:?}, User: {}",
        old_channel, new_channel, new.user_id
    );

    match (old_channel, new_channel) {
        (None, Some(_)) => {
            info!(
                "User {} joined voice channel {:?}",
                new.user_id, new.channel_id
            );
            handle_connection(stats, &ctx, config, &new).await?;
        }
        (Some(_), None) => {
            info!(
                "User {} left voice channel {:?}",
                new.user_id, new.channel_id
            );

            if let Some(old_state) = old {
                handle_disconnection(stats, &ctx, config, old_state).await?;
            }
        }
        (Some(old_ch), Some(new_ch)) if old_ch != new_ch => {
            info!(
                "User {} moved from channel {} to {}",
                new.user_id, old_ch, new_ch
            );

            if new_ch == config.generator.afk_channel_id
                && let Some(member) = new.member
            {
                debug!("User {} moved to AFK channel", member.user.id);
                stats.user_left(&member.user).await?;
            } else if old_ch == config.generator.afk_channel_id {
                debug!("User {} left AFK channel", new.user_id);

                handle_connection(stats, &ctx, config, &new).await?;
                stats.user_joined(new.user_id).await;
            }

            if let Some(old_state) = old {
                handle_disconnection(stats, &ctx, config, old_state).await?;
            }
        }
        _ => {
            debug!("No significant voice state change for user {}", new.user_id);
        }
    }

    Ok(())
}

async fn handle_connection(
    stats: &VoiceStats,
    ctx: &Context,
    config: &Config,
    state: &VoiceState,
) -> Result<()> {
    debug!("Handling connection for user {}", state.user_id);

    if let Some(channel_id) = state.channel_id
        && let Some(guild_id) = state.guild_id
        && channel_id == config.generator.channel_id
    {
        debug!(
            "User {} joined generator channel {}",
            state.user_id, channel_id
        );

        if let Some(guild_channel) = channel_id.to_channel(&ctx).await?.guild()
            && let Some(parent_id) = guild_channel.parent_id
        {
            let name = config
                .drop_rates
                .get_random_drop(&config.channels)
                .map_or(DEFAULT_CHANNEL_NAME, |s| s.as_str());

            info!(
                "Creating new voice channel '{}' in category {}",
                name, parent_id
            );

            let builder = CreateChannel::new(name)
                .category(parent_id)
                .kind(ChannelType::Voice);

            let new_channel = guild_id.create_channel(&ctx, builder).await?;

            info!(
                "Moving user {} to new channel {}",
                state.user_id, new_channel.id
            );

            guild_id
                .move_member(&ctx, state.user_id, new_channel.id)
                .await?;

            debug!(
                "Successfully created and moved user to channel {}",
                new_channel.id
            );
        } else {
            warn!(
                "Could not get guild channel or parent_id for channel {}",
                channel_id
            );
        }

        stats.user_joined(state.user_id).await;
    }

    Ok(())
}

async fn handle_disconnection(
    stats: &VoiceStats,
    ctx: &Context,
    config: &Config,
    state: VoiceState,
) -> Result<()> {
    if let Some(channel_id) = state.channel_id
        && channel_id != config.generator.channel_id
    {
        debug!("Checking if channel {} should be deleted", channel_id);

        if let Some(guild_channel) = channel_id.to_channel(&ctx).await?.guild()
            && guild_channel.parent_id == Some(config.generator.parent_id)
        {
            debug!("Channel {} is in managed category", channel_id);

            if let Some(member) = state.member {
                if channel_is_empty(ctx, &guild_channel).await? {
                    info!("Channel {} is empty, deleting it", channel_id);
                    channel_id.delete(&ctx).await?;
                    debug!("Successfully deleted channel {}", channel_id);
                } else {
                    debug!("Channel {} still has members, keeping it", channel_id);
                }

                stats.user_left(&member.user).await?;
            } else {
                warn!("No member data available for disconnection");
            }
        } else {
            debug!("Channel {} not in managed category, ignoring", channel_id);
        }
    }

    Ok(())
}

async fn channel_is_empty(ctx: &Context, channel: &GuildChannel) -> Result<bool> {
    let members = channel.members(ctx)?;
    let is_empty = members.is_empty();

    debug!("Channel {} has {} members", channel.id, members.len());

    Ok(is_empty)
}
