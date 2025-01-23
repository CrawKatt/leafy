use crate::utils::debug::IntoUnwrapResult;
use crate::utils::CommandResult;
use crate::DB;
use bon::Builder;
use poise::serenity_prelude as serenity;
use serde::{Deserialize, Serialize};
use serenity::all::Member;

#[derive(Serialize, Deserialize, Debug, Builder)]
pub struct SanctionRolesConfig {
    pub roles: Vec<String>,  // IDs de los roles de sanción configurados para el servidor
}

#[derive(Serialize, Deserialize, Debug, Builder)]
pub struct SanctionRoles {
    pub guild_id: String,
    pub user_id: String,
    pub roles: Vec<String>,
}

pub async fn handler(
    user: &serenity::User,
    guild_id: &serenity::GuildId,
    member_data_old: Option<&Member>,
) -> CommandResult {
    // Recuperar los roles de sanción configurados para el servidor
    let config: Option<SanctionRolesConfig> = DB
        .select(("sanction_roles_config", guild_id.to_string()))
        .await?;

    // Si no hay roles de sanción configurados, no hacemos nada
    let sanction_roles = config
        .into_result()?
        .roles;
    
    let member = member_data_old.clone().into_result()?;

    // Filtrar los roles del usuario para quedarse solo con los roles de sanción
    let user_sanction_roles: Vec<String> = member
        .roles
        .iter()
        .filter(|role_id| sanction_roles.contains(&role_id.to_string()))
        .map(|role_id| role_id.to_string())
        .collect();

    // Si el usuario no tiene roles de sanción, no almacenamos nada
    if user_sanction_roles.is_empty() {
        return Ok(());
    }

    // Construir el registro del usuario sancionado
    let data = SanctionRoles::builder()
        .user_id(user.id.to_string())
        .guild_id(guild_id.to_string())
        .roles(user_sanction_roles)
        .build();

    // Crear o actualizar el registro en la base de datos
    let _created: Option<SanctionRoles> = DB.create(("sanctioned_users", user.id.to_string()))
        .content(data)
        .await?;

    Ok(())
}