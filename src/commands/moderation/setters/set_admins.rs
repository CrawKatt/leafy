use serenity::all::Role;

use crate::DB;
use crate::utils::{CommandResult, Context};
use crate::utils::config::{Admin, DatabaseOperations, GuildData};
use crate::utils::debug::IntoUnwrapResult;

#[poise::command(
    prefix_command,
    slash_command,
    category = "Moderator",
    required_permissions = "MANAGE_ROLES",
    guild_only,
    ephemeral
)]
pub async fn set_admins(
    ctx: Context<'_>,
    #[description = "El rol para establecer como administrador"]
    role: Role,
    #[description = "El rol para establecer un administrador secundario (opcional)"]
    role_2: Option<Role>,
) -> CommandResult {
    ctx.defer().await?;
    DB.use_ns("discord-namespace").use_db("discord").await?;

    let guild_name = ctx.guild().into_result()?.name.clone();
    let guild_id = ctx.guild_id().into_result()?;
    let role_id = role.id.to_string();
    let role_2_id = role_2.as_ref().map(|role| role.id.to_string());
    let mut admin_roles = vec![role_id.clone()];
    if let Some(role_2_id) = &role_2_id {
        admin_roles.push(role_2_id.clone());
    }

    let existing_data = GuildData::verify_data(guild_id).await?;
    if existing_data.is_none() {
        let data = GuildData::builder()
            .admins(Admin::builder()
                .role(admin_roles.clone())
                .build()
            )
            .build();
        
        data.save_to_db(guild_id).await?;
        ctx.say(format!("Config data created for {guild_name}, admin roles set to: **{}**", role.name)).await?;
        return Ok(());
    }

    // Actualizar la configuración existente
    let data = Admin::builder()
        .role(admin_roles.clone())
        .build();
    data.update_admins("admins.roles", admin_roles, &guild_id.to_string()).await?;

    let role_name = &role.name;
    let role_2_name = role_2.as_ref().map_or("None", |role| &*role.name);
    ctx.say(format!("Admin roles set to: **{role_name}** and **{role_2_name}**")).await?;

    Ok(())
}