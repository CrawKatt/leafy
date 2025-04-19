use crate::utils::debug::IntoUnwrapResult;
use crate::utils::CommandResult;
use crate::{debug, location, DB};
use bon::Builder;
use poise::serenity_prelude as serenity;
use serde::{Deserialize, Serialize};
use serenity::all::{GuildId, Member, User};
use surrealdb::opt::PatchOp;

#[derive(Serialize, Deserialize, Debug, Builder)]
pub struct SanctionRoles {
    pub roles: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Builder)]
pub struct SanctionedUsers {
    pub guild_id: String,
    pub user_id: String,
    pub roles: Vec<String>,
}

pub async fn handler(
    user: &User,
    guild_id: &GuildId,
    member_data_old: Option<&Member>,
) -> CommandResult {
    command_result(user, guild_id, member_data_old).await?;

    Ok(())
}

async fn command_result(user: &User, guild_id: &GuildId, member_data_old: Option<&Member>) -> CommandResult {
    // Recuperar los roles de sanción configurados para el servidor
    let config: Option<SanctionRoles> = DB
        .select(("sanction_roles", guild_id.to_string()))
        .await?;

    // Si no hay roles de sanción configurados, no hacemos nada
    let Some(data) = config else {
        debug!("No se han encontrado los roles de sanción o no están configurados {}", location!());
        return Ok(())
    };

    // Obtener los roles de sanción
    let sanction_roles = data.roles;

    // Obtener la data en caché del miembro que salió del servidor
    let member = member_data_old.into_result()?;

    // Filtrar los roles del usuario para quedarse solo con los roles de sanción
    let user_sanction_roles: Vec<String> = member
        .roles
        .iter()
        .filter(|role_id| sanction_roles.contains(&role_id.to_string()))
        .map(std::string::ToString::to_string)
        .collect();

    // Si el usuario no tiene roles de sanción, no almacenamos nada
    if user_sanction_roles.is_empty() {
        debug!("No tiene roles de sanción");
        return Ok(());
    }

    // Construir el registro del usuario sancionado
    let data = SanctionedUsers::builder()
        .user_id(user.id.to_string())
        .guild_id(guild_id.to_string())
        .roles(user_sanction_roles)
        .build();

    // Si ya existe un registro con el UserId obtenido, actualizar sus roles
    let existing_data: Option<SanctionedUsers> = DB
        .select(("sanctioned_users", user.id.to_string()))
        .await?;

    if existing_data.is_some() {
        let _updated: Option<SanctionedUsers> = DB
            .update(("sanctioned_users", user.id.to_string()))
            .patch(PatchOp::replace("roles", &data.roles))
            .await?;

        return Ok(())
    }

    // Crear o actualizar el registro en la base de datos
    let _created: Option<SanctionedUsers> = DB
        .create(("sanctioned_users", user.id.to_string()))
        .content(data)
        .await?;

    Ok(())
}