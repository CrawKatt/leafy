use serenity::all::{CreateAttachment, CreateMessage, Member, UserId};
use crate::location;
use crate::utils::{CommandResult, Context};
use crate::utils::config::GuildData;
use crate::utils::debug::{IntoUnwrapResult, UnwrapLog};

#[poise::command(
    prefix_command,
    slash_command,
    category = "Fun",
    aliases("shh"),
    guild_only,
    track_edits
)]
pub async fn cat_shh(ctx: Context<'_>, user: Option<Member>) -> CommandResult {
    let channel_id = ctx.channel_id();
    let author = &*ctx.author_member().await.into_result()?;
    let member = user.unwrap_or_else(|| author.clone());
    let user_id = member.user.id;

    // obtener el usuario prohibido
    let forbidden_user = GuildData::verify_data(ctx.guild_id().unwrap()).await?
        .into_result()?
        .forbidden
        .user
        .unwrap_log(location!())?
        .parse::<UserId>()?;

    if user_id == forbidden_user {
        poise::say_reply(ctx, "No puedes usar este comando con un usuario al que no está permitido hacer mención (@)").await?;
        return Ok(())
    }

    let attachment = CreateAttachment::path("assets/shh.gif").await?;
    channel_id.send_files(ctx.http(), vec![attachment], CreateMessage::default().content(format!("<@{user_id}>"))).await?;

    Ok(())
}