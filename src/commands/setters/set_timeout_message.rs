use crate::utils::{CommandResult, Context};
use crate::utils::misc::debug::UnwrapLog;
use crate::commands::setters::TimeOutMessageData;

/// Establece el mensaje de time out
#[poise::command(
    prefix_command,
    slash_command,
    category = "Moderator",
    required_permissions = "MODERATE_MEMBERS",
    guild_only,
    ephemeral
)]
pub async fn set_time_out_message(
    ctx: Context<'_>,
    #[description = "The message to set as the time out message"] time_out_message: String,
) -> CommandResult {
    let guild_id = ctx.guild_id().unwrap_log("Failed to get guild id", module_path!(), line!())?;
    let time_out_message_data = TimeOutMessageData::new(guild_id, time_out_message.clone());
    let existing_data = time_out_message_data.verify_data().await?;

    let Some(existing_data) = existing_data else {
        time_out_message_data.save_to_db().await?;
        ctx.say(format!("Time out message establecido: {time_out_message}")).await?;
        return Ok(())
    };

    existing_data.update_in_db().await?;
    ctx.say(format!("Time out message actualizado: {time_out_message}")).await?;

    Ok(())
}