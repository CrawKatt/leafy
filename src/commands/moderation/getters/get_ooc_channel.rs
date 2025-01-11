use crate::DB;
use crate::utils::config::GuildData;
use crate::utils::{CommandResult, Context};

#[poise::command(
    prefix_command,
    slash_command,
    category = "Moderator",
    required_permissions = "MODERATE_MEMBERS",
    guild_only,
    ephemeral
)]
pub async fn get_ooc_channel(ctx: Context<'_>) -> CommandResult {
    ctx.defer().await?;
    let guild_id = ctx.guild_id().unwrap();
    let existing_data: Option<GuildData> = DB
        .select(("guild_config", guild_id.to_string()))
        .await?;

    let Some(existing_data) = existing_data else {
        ctx.say("No hay un canal de OOC establecido").await?;
        return Ok(())
    };

    let ooc_channel_id = existing_data
        .channels
        .ooc
        .ok_or("No se encontró un canal de OOC o no ha sido establecido")?;
    
    ctx.say(format!("El canal de Fuera de Contexto establecido es: <#{ooc_channel_id}>")).await?;

    Ok(())
}