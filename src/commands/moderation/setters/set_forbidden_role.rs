use serenity::all::Role;

use crate::DB;
use crate::utils::{CommandResult, Context};
use crate::utils::config::{Forbidden, GuildData, DatabaseOperations};

#[poise::command(
    prefix_command,
    slash_command,
    category = "Moderator",
    required_permissions = "MODERATE_MEMBERS",
    guild_only,
    ephemeral
)]
pub async fn set_forbidden_role(
    ctx: Context<'_>,
    #[description = "The role to set as the forbidden role"]
    forbidden_role: Role,
) -> CommandResult {
    ctx.defer().await?;
    DB.use_ns("discord-namespace").use_db("discord").await?;
    let guild_id = ctx.guild_id().unwrap();
    let role_id = forbidden_role.id.to_string();
    let existing_data = GuildData::verify_data(guild_id).await?;
    if existing_data.is_none() {
        let data = GuildData::builder()
            .forbidden(Forbidden::builder()
                .role(role_id.clone())
                .build()
            )
            .build();
        data.save_to_db(guild_id).await?;
        ctx.say(format!("Set forbidden role to: **{}**", forbidden_role.name)).await?;

        return Ok(())
    };
    let data = Forbidden::builder().role(role_id.clone()).build();
    
    // NOTA: Se debe utilizar el nombre del objeto junto con el campo a actualizar
    // Ejemplo: `forbidden.role_id`
    // Actualizar usando `role_id` creará un nuevo campo en la base de datos fuera del objeto
    data.update_field_in_db("forbidden.role", &role_id, &guild_id.to_string()).await?;
    ctx.say(format!("Set forbidden role to: **{}**", forbidden_role.name)).await?;

    Ok(())
}