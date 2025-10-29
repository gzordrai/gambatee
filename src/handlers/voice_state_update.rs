use serenity::all::{
    ChannelType, Context, CreateChannel, GuildChannel, GuildId, Result, VoiceState,
};

use crate::config::Config;

const DEFAULT_CHANNEL_NAME: &str = "channel";

pub async fn voice_state_update(
    ctx: Context,
    old: Option<VoiceState>,
    new: VoiceState,
) -> Result<()> {
    let mut data = ctx.data.write().await;
    let config = data.get_mut::<Config>().unwrap();

    if let (Some(channel_id), Some(guild_id)) = (new.channel_id, new.guild_id) {
        if channel_id == config.generator.channel_id {
            let channel = channel_id.to_channel(&ctx).await?;

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
                        .await?;

                    guild_id.move_member(&ctx, new.user_id, channel.id).await?;
                }
            }
        }
    }

    if let Some(state) = old {
        handle_deconnection(&ctx, config, state).await?;
    }

    Ok(())
}

async fn handle_deconnection(ctx: &Context, config: &mut Config, state: VoiceState) -> Result<()> {
    if let Some(channel_id) = state.channel_id {
        if channel_id != config.generator.channel_id {
            let channel = channel_id.to_channel(&ctx).await?;

            if let Some(guild_channel) = channel.guild() {
                if guild_channel.parent_id == Some(config.generator.parent_id)
                    && channel_is_empty(&ctx, &guild_channel).await?
                {
                    channel_id.delete(&ctx).await?;
                }
            }
        }
    }

    Ok(())
}

async fn channel_is_empty(ctx: &Context, channel: &GuildChannel) -> Result<bool> {
    Ok(channel.members(ctx)?.is_empty())
}
