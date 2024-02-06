use serenity::all::Channel;
use crate::DB;
use crate::utils::autocomplete::args_log_command;
use crate::utils::{CommandResult, Context};
use crate::utils::debug::UnwrapLog;
use crate::commands::setters::GuildData;

/// Establece el canal de logs del servidor
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
    #[autocomplete = "args_log_command"]
    #[description = "The channel to set as the log channel"] channel: Channel,
) -> CommandResult {
    DB.use_ns("discord-namespace").use_db("discord").await?;

    let current_module = file!();
    let guild_id = ctx.guild_id().unwrap_log("Could not get guild_id", current_module, line!())?;
    let channel_id = channel.id();
    let data = GuildData::new(guild_id, channel_id);
    let existing_data = data.verify_data().await?;

    let Some(_) = existing_data else {
        // Si el dato no existe, créalo
        data.save_to_db().await?;
        ctx.say(format!("Log channel establecido: <#{channel_id}>")).await?;
        return Ok(());
    };

    // Si el dato ya existe, actualízalo
    data.update_in_db().await?;
    ctx.say(format!("Log channel establecido: <#{channel_id}>")).await?;

    Ok(())
}