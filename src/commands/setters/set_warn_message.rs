use crate::DB;
use crate::utils::{CommandResult, Context};
use crate::utils::debug::UnwrapLog;
use crate::commands::setters::WarnMessageData;

/// Establece el mensaje de advertencia si se menciona a un usuario o un usuario con un rol prohíbido
#[poise::command(
    prefix_command,
    slash_command,
    category = "Moderator",
    required_permissions = "MODERATE_MEMBERS",
    guild_only,
    ephemeral
)]
pub async fn set_warn_message(
    ctx: Context<'_>,
    #[description = "The message to set as the warn message"] warn_message: String,
) -> CommandResult {
    DB.use_ns("discord-namespace").use_db("discord").await?;
    let current_module = file!();
    let guild_id = ctx.guild_id().unwrap_log("Could not get the guild_id", current_module, line!())?;

    let data = WarnMessageData::new(guild_id, warn_message.clone());
    let existing_data = data.verify_data().await?;

    if existing_data.is_some() {
        data.update_in_db().await?;
    } else {
        data.save_to_db().await?;
    }

    poise::say_reply(ctx, format!("El mensaje de advertencia ha sido establecido a: {warn_message}")).await?;

    Ok(())
}