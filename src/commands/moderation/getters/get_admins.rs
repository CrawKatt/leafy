use serenity::all::RoleId;

use crate::DB;
use crate::utils::config::GuildData;
use crate::utils::{CommandResult, Context};
use crate::utils::debug::IntoUnwrapResult;

#[poise::command(
    prefix_command,
    slash_command,
    category = "Moderator",
    required_permissions = "MANAGE_ROLES",
    guild_only,
    ephemeral
)]
pub async fn get_admins(
    ctx: Context<'_>,
) -> CommandResult {
    ctx.defer().await?;
    let guild_id = ctx.guild_id().unwrap();

    let database_info: Option<GuildData> = DB
        .select(("guild_config", guild_id.to_string()))
        .await?;

    let Some(database_info) = database_info else {
        poise::say_reply(ctx, "No se han establecido roles de administrador").await?;
        return Ok(())
    };

    let admin_roles = database_info.admins.role.into_result()?;
    if admin_roles.is_empty() {
        poise::say_reply(ctx, "No hay roles de administrador establecidos").await?;
        return Ok(());
    }

    let guild = ctx.cache().guild(guild_id).into_result()?.clone();
    let mut role_names = Vec::new();

    for role_id_str in admin_roles {
        if let Ok(role_id) = role_id_str.parse::<RoleId>() {
            if let Some(role) = guild.roles.get(&role_id) {
                role_names.push(role.name.clone());
            }
        }
    }

    if role_names.is_empty() {
        poise::say_reply(ctx, "No se encontraron roles válidos en la configuración de administrador").await?;
        return Ok(());
    }

    let role_names = role_names.join(", ");
    poise::say_reply(ctx, format!("Los roles de administrador actuales son: **{role_names}**")).await?;

    Ok(())
}