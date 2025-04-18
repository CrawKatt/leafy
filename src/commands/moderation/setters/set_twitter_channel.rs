use serenity::all::ChannelId;
use surrealdb::opt::PatchOp;
use crate::DB;
use crate::utils::{CommandResult, Context};
use crate::utils::config::{GuildData, Twitter};

#[poise::command(
    prefix_command,
    slash_command,
    category = "Moderator",
    required_permissions = "MODERATE_MEMBERS",
    guild_only,
    ephemeral
)]
pub async fn set_twitter_channel(
    ctx: Context<'_>,
    #[description = "Canal para recibir tweets"] channel: ChannelId,
) -> CommandResult {
    ctx.defer().await?;
    DB.use_ns("discord-namespace").use_db("discord").await?;
    let guild_id = ctx.guild_id().unwrap();
    let existing_data = GuildData::verify_data(guild_id).await?;

    if existing_data.is_none() {
        let data = GuildData::builder()
            .twitter(Twitter::builder()
                .channel(channel.to_string())
                .build()
            )
            .build();
        data.save_to_db(guild_id).await?;
        ctx.say(format!("Canal de Twitter establecido: Tweets se enviarán a {channel}")).await?;

        return Ok(())
    }

    let _update: Option<GuildData> = DB
        .update(("guild_config", &guild_id.to_string()))
        .patch(PatchOp::replace("twitter/channel", channel.to_string()))
        .await?;

    ctx.say(format!("Canal de Twitter actualizado: Tweets se enviarán a {channel}")).await?;

    Ok(())
}