use crate::utils::{CommandResult, Context};
use crate::commands::setters::set_ooc_channel::OocChannel;
use crate::DB;

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
    let sql_query = "SELECT * FROM ooc_channel WHERE guild_id = $guild_id";
    let existing_data: Option<OocChannel> = DB
        .query(sql_query)
        .bind(("guild_id", guild_id))
        .await?
        .take(0)?;

    let Some(existing_data) = existing_data else {
        ctx.say("No hay un canal de OOC establecido").await?;
        return Ok(())
    };

    let ooc_channel_id = existing_data.channel_id;
    ctx.say(format!("El canal de Fuera de Contexto establecido es: <#{ooc_channel_id}>")).await?;

    Ok(())
}