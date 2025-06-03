use crate::utils::config::{GuildData, Messages};
use crate::utils::{CommandResult, Context};
use crate::DB;
use serenity::all::Attachment;
use surrealdb::opt::PatchOp;

#[poise::command(slash_command, guild_only, required_permissions = "ADMINISTRATOR")]
pub async fn set_welcome_background(
    ctx: Context<'_>,
    #[description = "Nueva imagen de fondo"] attachment: Attachment,
) -> CommandResult {
    let guild_id = ctx.guild_id().unwrap();
    let file_path = format!("assets/{guild_id}_background.png");

    let bytes = attachment.download().await?;
    tokio::fs::write(&file_path, &bytes).await?;

    DB.use_ns("discord-namespace").use_db("discord").await?;

    let existing_data = GuildData::verify_data(guild_id).await?;
    if existing_data.is_none() {
        let data = GuildData::builder()
            .messages(Messages::builder()
                .background(&file_path)
                .build()
            )
            .build();
        data.save_to_db(guild_id).await?;
        ctx.say("✅ Imagen de fondo de bienvenida establecida correctamente").await?;

        return Ok(());
    }

    let _update: Option<GuildData> = DB
        .update(("guild_config", &guild_id.to_string()))
        .patch(PatchOp::replace("messages/background", &file_path))
        .await?;

    ctx.say("✅ Imagen de fondo de bienvenida actualizada correctamente").await?;

    Ok(())
}