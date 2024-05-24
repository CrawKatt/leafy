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
    let guild_id = ctx.guild_id().unwrap();
    let sql_query = "SELECT * FROM guild_config WHERE guild_id = $guild_id";
    let existing_data: Option<GuildData> = DB
        .query(sql_query)
        .bind(("guild_id", guild_id))
        .await?
        .take(0)?;

    let Some(existing_data) = existing_data else {
        ctx.say("No hay un canal de OOC establecido").await?;
        return Ok(())
    };

    let ooc_channel_id = existing_data
        .channel_config
        .ooc_channel_id
        .ok_or("No se encontró un canal de OOC o no ha sido establecido")?;
    
    ctx.say(format!("El canal de Fuera de Contexto establecido es: <#{ooc_channel_id}>")).await?;

    Ok(())
}