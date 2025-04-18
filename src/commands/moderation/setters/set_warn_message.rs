use crate::utils::config::{GuildData, Messages};
use crate::utils::{CommandResult, Context};
use crate::DB;
use surrealdb::opt::PatchOp;

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
    ctx.defer().await?;
    DB.use_ns("discord-namespace").use_db("discord").await?;
    let guild_id = ctx.guild_id().unwrap();
    let existing_data = GuildData::verify_data(guild_id).await?;

    if existing_data.is_none() {
        let data = GuildData::builder()
            .messages(Messages::builder()
                .warn(&warn_message)
                .build()
            )
            .build();
        data.save_to_db(guild_id).await?;
        ctx.say(format!("El mensaje de advertencia ha sido establecido a: {warn_message}")).await?;
        
        return Ok(())
    }

    let _update: Option<GuildData> = DB
        .update(("guild_config", &guild_id.to_string()))
        .patch(PatchOp::replace("messages/warn", &warn_message))
        .await?;

    ctx.say(format!("El mensaje de advertencia ha sido establecido a: {warn_message}")).await?;

    Ok(())
}