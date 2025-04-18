use crate::utils::config::{GuildData, Twitter};
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
pub async fn set_twitter_user(
    ctx: Context<'_>,
    #[description = "Usuario de Twitter a seguir"] user: String,
) -> CommandResult {
    ctx.defer().await?;
    DB.use_ns("discord-namespace").use_db("discord").await?;
    let guild_id = ctx.guild_id().unwrap();
    let existing_data = GuildData::verify_data(guild_id).await?;

    if existing_data.is_none() {
        let data = GuildData::builder()
            .twitter(Twitter::builder()
                .user(&user)
                .build()
            )
            .build();
        data.save_to_db(guild_id).await?;
        ctx.say(format!("Usuario de Twitter establecido: Se seguirán los tweets de {user}")).await?;

        return Ok(())
    }

    let _update: Option<GuildData> = DB
        .update(("guild_config", &guild_id.to_string()))
        .patch(PatchOp::replace("twitter/user", &user))
        .await?;

    ctx.say(format!("Usuario de Twitter actualizado: Se seguirán los tweets de {user}")).await?;
    Ok(())
}