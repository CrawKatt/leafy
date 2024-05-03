use serenity::all::Role;
use crate::{DB, unwrap_log};
use crate::utils::misc::config::{Admin, GuildData};
use crate::utils::{CommandResult, Context};
use crate::utils::misc::autocomplete::args_set_admins;

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
    #[autocomplete = "args_set_admins"]
    role: Role,
    #[description = "El rol para establecer un administrador secundario (opcional)"]
    role_2: Option<Role>,
) -> CommandResult {
    DB.use_ns("discord-namespace").use_db("discord").await?;

    let guild_name = ctx.guild().unwrap().name.clone();
    let guild_id = unwrap_log!(ctx.guild_id(), "No se pudo obtener el guild_id");
    let role_id = role.id.to_string();
    let role_2_id = role_2.as_ref().map(|role| role.id.to_string());
    let existing_data = GuildData::verify_data(guild_id).await?;
    
    if existing_data.is_none() {
        let data = GuildData::default()
            .guild_id(guild_id)
            .admins(Admin::default()
                .role_id(&role_id)
            );
        data.save_to_db().await?;
        ctx.say(format!("Config data created for {guild_name} stablished admin to: {}", role.name)).await?;
        
        return Ok(())
    }

    let Some(role_2_id) = role_2_id else {
        let data = Admin::default()
            .role_id(&role_id);
        data.update_field_in_db("admins.role_id", &role_id, &guild_id.to_string()).await?;
        ctx.say(format!("Admin role set to: **{role_id}**")).await?;

        return Ok(())
    };

    let data = Admin::default()
        .role_id(&role_id)
        .role_2_id(&role_2_id);

    data.update_field_in_db("admins.role_id", &role_id, &guild_id.to_string()).await?;
    data.update_field_in_db("admins.role_2_id", &role_2_id, &guild_id.to_string()).await?;

    ctx.say(format!("Admin roles set to: **{role_id}** and **{role_2_id}**")).await?;

    Ok(())
}