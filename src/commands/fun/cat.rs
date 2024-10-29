use serenity::all::{CreateAttachment, CreateMessage, UserId};
use crate::location;
use crate::utils::{CommandResult, Context};
use crate::utils::config::GuildData;
use crate::utils::debug::{IntoUnwrapResult, UnwrapLog};

#[poise::command(
    prefix_command,
    category = "Fun",
    aliases("shh"),
    guild_only,
    track_edits
)]
pub async fn cat_shh(ctx: Context<'_>, user: Option<UserId>) -> CommandResult {
    let channel_id = ctx.channel_id();
    let user = user.unwrap_or_else(|| ctx.author().id);

    let user = user.to_user(&ctx).await?;

    // obtener el usuario prohibido
    let forbidden_user = GuildData::verify_data(ctx.guild_id().unwrap()).await?
        .into_result()?
        .forbidden
        .user
        .unwrap_log(location!())?
        .parse::<UserId>()?;

    if user.id == forbidden_user {
        poise::say_reply(ctx, "No puedes usar este comando con un usuario al que no está permitido hacer mención (@)").await?;
        return Ok(())
    }

    let attachment = CreateAttachment::path("assets/shh.gif").await?;
    channel_id.send_files(ctx.http(), vec![attachment], CreateMessage::default()).await?;

    Ok(())
}