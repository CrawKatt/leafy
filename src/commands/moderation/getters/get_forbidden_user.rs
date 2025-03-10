use serenity::all::UserId;
use crate::DB;
use crate::utils::config::GuildData;
use crate::utils::{CommandResult, Context};

/// Obtiene el usuario prohíbido de mencionar si está establecido.
#[poise::command(
    prefix_command,
    slash_command,
    category = "Moderator",
    required_permissions = "MANAGE_ROLES",
    guild_only,
    ephemeral
)]
pub async fn get_forbidden_user(
    ctx: Context<'_>,
) -> CommandResult {
    ctx.defer().await?;
    DB.use_ns("discord-namespace").use_db("discord").await?;
    let guild_id = ctx.guild_id().unwrap();
    let database_info: Option<GuildData> = DB
        .select(("guild_config", guild_id.to_string()))
        .await?;
    
    let Some(forbidden_user_id) = database_info else {
        ctx.say("No se ha establecido un usuario prohíbido de mencionar").await?;
        return Ok(())
    };
    
    let forbidden_user_id = forbidden_user_id
        .forbidden
        .user
        .ok_or("No se ha establecido un usuario prohíbido de mencionar")?
        .parse::<UserId>()?;
    
    let forbidden_user = ctx
        .cache()
        .user(forbidden_user_id)
        .ok_or("No se ha establecido un usuario prohíbido de mencionar")?
        .name
        .clone();
    
    ctx.say(format!("Forbidden user is **{forbidden_user}**")).await?;

    Ok(())
}