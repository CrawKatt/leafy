use serenity::all::Channel;

use crate::DB;
use crate::utils::{CommandResult, Context};
use crate::utils::config::{Channels, GuildData, DatabaseOperations};

#[poise::command(
    prefix_command,
    slash_command,
    category = "Moderator",
    required_permissions = "MODERATE_MEMBERS",
    guild_only,
    ephemeral
)]
pub async fn set_log_channel(
    ctx: Context<'_>,
    #[description = "The channel to set as the log channel"]
    #[channel_types("Text")]
    log_channel: Channel,
) -> CommandResult {
    ctx.defer().await?;
    DB.use_ns("discord-namespace").use_db("discord").await?;
    let guild_id = ctx.guild_id().unwrap();
    let channel_id = log_channel.id().to_string();

    let existing_data = GuildData::verify_data(guild_id).await?;

    if existing_data.is_none() {
        let data = GuildData::builder()
            .channels(Channels::builder()
                .logs(channel_id.clone())
                .build()
            )
            .build();
        data.save_to_db(guild_id).await?;
        ctx.say(format!("Log channel establecido: <#{channel_id}>")).await?;

        return Ok(());
    }

    let data = Channels::builder()
        .logs(channel_id.clone())
        .build();

    data.update_field_in_db("channels/logs", &channel_id, &guild_id.to_string()).await?;
    ctx.say(format!("Log channel establecido: <#{channel_id}>")).await?;

    Ok(())
}