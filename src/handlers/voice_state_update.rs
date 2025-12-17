use serenity::all::{ChannelType, Context, CreateChannel, GuildChannel, VoiceState};

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

    match (old_channel, new_channel) {
        (None, Some(_)) => {
            handle_connection(&stats, &ctx, config, &new).await?;
        }
        (Some(_), None) => {
            if let Some(old_state) = old {
                handle_disconnection(stats, &ctx, config, old_state).await?;
            }
        }
        (Some(old_ch), Some(new_ch)) if old_ch != new_ch => {
            if new_ch == config.generator.afk_channel_id
                && let Some(member) = new.member
            {
                stats.user_left(member.user).await?;
            } else if old_ch == config.generator.afk_channel_id {
                stats.user_joined(new.user_id).await;
            }

            if let Some(old_state) = old {
                handle_disconnection(stats, &ctx, config, old_state).await?;
            }
        }
        _ => {}
    }

    Ok(())
}

async fn handle_connection(
    stats: &VoiceStats,
    ctx: &Context,
    config: &Config,
    state: &VoiceState,
) -> Result<()> {
    if let Some(channel_id) = state.channel_id
        && let Some(guild_id) = state.guild_id
        && channel_id == config.generator.channel_id
        && let Some(guild_channel) = channel_id.to_channel(&ctx).await?.guild()
        && let Some(parent_id) = guild_channel.parent_id
    {
        let name = config
            .drop_rates
            .get_random_drop(&config.channels)
            .map_or(DEFAULT_CHANNEL_NAME, |s| s.as_str());

        let builder = CreateChannel::new(name)
            .category(parent_id)
            .kind(ChannelType::Voice);

        let new_channel = guild_id.create_channel(&ctx, builder).await?;

        guild_id
            .move_member(&ctx, state.user_id, new_channel.id)
            .await?;
    }

    stats.user_joined(state.user_id).await;

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
        && let Some(guild_channel) = channel_id.to_channel(&ctx).await?.guild()
        && guild_channel.parent_id == Some(config.generator.parent_id)
        && let Some(member) = state.member
    {
        if channel_is_empty(ctx, &guild_channel).await? {
            channel_id.delete(&ctx).await?;
        }

        stats.user_left(member.user).await?;
    }

    Ok(())
}

async fn channel_is_empty(ctx: &Context, channel: &GuildChannel) -> Result<bool> {
    Ok(channel.members(ctx)?.is_empty())
}
