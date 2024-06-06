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
    let guild_id = ctx.guild_id().unwrap();
    
    let sql_query = "SELECT * FROM guild_config WHERE guild_id = $guild_id";
    let database_info: Option<GuildData> = DB
        .query(sql_query)
        .bind(("guild_id", guild_id)) // pasar el valor
        .await?
        .take(0)?;
    
    let Some(database_info) = database_info else {
        poise::say_reply(ctx, "No se han establecido roles de administrador").await?;
        return Ok(())
    };
    
    let role_id_1 = database_info
        .admins
        .role;
    
    let role_id_2 = database_info
        .admins
        .role_2;

    let mut role_names = String::new();

    if let Some(role_id) = role_id_1 {
        let role_id = role_id.parse::<RoleId>()?;
        
        let guild = ctx.cache().guild(guild_id).into_result()?;
        let role = guild.roles.get(&role_id).into_result()?;
        let role_name = &*role.name;
        
        role_names.push_str(role_name);
        drop(guild);
    }

    if let Some(role_id) = role_id_2 {
        let role_id = role_id.parse::<RoleId>()?;
        
        let guild = ctx.cache().guild(guild_id).into_result()?;
        let role = guild.roles.get(&role_id).into_result()?;
        let role_name = &*role.name;
        
        if !role_names.is_empty() {
            role_names.push_str(", ");
        }
        role_names.push_str(role_name);
        drop(guild);
    }

    if role_names.is_empty() {
        poise::say_reply(ctx, "No hay roles de administrador establecidos").await?;
        return Ok(())
    };

    poise::say_reply(ctx, format!("Los roles de administrador actuales son: **{role_names}**")).await?;

    Ok(())
}