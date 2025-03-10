use crate::{DB, location};
use crate::utils::config::GuildData;
use crate::utils::debug::{IntoUnwrapResult, UnwrapLog};
use crate::utils::{CommandResult, Context};

#[poise::command(
    prefix_command,
    slash_command,
    category = "Moderator",
    track_edits,
    required_permissions = "MANAGE_ROLES",
    guild_only,
    ephemeral
)]
pub async fn get_welcome_channel(
    ctx: Context<'_>,
) -> CommandResult {
    ctx.defer().await?;
    DB.use_ns("discord-namespace").use_db("discord").await?;

    let guild_id = ctx.guild_id().into_result()?;
    let existing_data: Option<GuildData> = DB
        .select(("guild_config", guild_id.to_string()))
        .await?;

    if existing_data.is_none() {
        poise::say_reply(ctx, "No se ha establecido un canal de bienvenida").await?;
        return Ok(())
    }

    let result = existing_data
        .unwrap_log(location!())?
        .channels
        .welcome
        .ok_or("No se encontró un canal de bienvenida o no ha sido establecido")?;
    
    poise::say_reply(ctx, format!("El canal de bienvenida está establecido en <#{result}>")).await?;

    Ok(())
}