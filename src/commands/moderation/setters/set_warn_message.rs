use crate::DB;
use crate::utils::{CommandResult, Context};
use crate::utils::config::{GuildData, Messages};

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
    let guild_id = ctx.guild_id().unwrap();
    let existing_data = GuildData::verify_data(guild_id).await?;

    if existing_data.is_none() {
        let data = GuildData::default()
            .guild_id(guild_id)
            .messages(Messages::default()
                .warn(&warn_message)
            );

        data.save_to_db().await?;
        ctx.say(format!("El mensaje de advertencia ha sido establecido a: {warn_message}")).await?;
        
        return Ok(())
    }

    let data = Messages::default()
        .warn(&warn_message);

    data.update_field_in_db("messages.warn", &warn_message, &guild_id.to_string()).await?;
    ctx.say(format!("El mensaje de advertencia ha sido establecido a: {warn_message}")).await?;

    Ok(())
}