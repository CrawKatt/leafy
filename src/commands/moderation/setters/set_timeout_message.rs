use crate::utils::{CommandResult, Context};
use crate::utils::config::{GuildData, Messages, DatabaseOperations};

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
    ctx.defer().await?;
    let guild_id = ctx.guild_id().unwrap();
    
    let existing_data = GuildData::verify_data(guild_id).await?;
    if existing_data.is_none() {
        let data = GuildData::builder()
            .messages(Messages::builder()
                .time_out(&time_out_message)
                .build()
            )
            .build();
        data.save_to_db(guild_id).await?;
        ctx.say(format!("Time out message establecido: {time_out_message}")).await?;

        return Ok(())
    }

    let data = Messages::builder()
        .time_out(&time_out_message)
        .build();

    data.update_field_in_db("messages.time_out", &time_out_message, &guild_id.to_string()).await?;
    ctx.say(format!("Time out message actualizado: {time_out_message}")).await?;

    Ok(())
}