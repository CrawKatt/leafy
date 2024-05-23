use serenity::all::{Member, RoleId};
use poise::serenity_prelude as serenity;
use crate::location;
use crate::utils::misc::debug::UnwrapLog;

/// Verifica si el usuario tiene un rol de administrador
/// Si el usuario tiene un rol de administrador, no se silenciará
pub fn check_admin_exception(admin_role_id: Option<String>, member: &Member, ctx: &serenity::Context) -> bool {
    admin_role_id.map_or(false, |admin_role_id| {
        member.roles(&ctx.cache)
            .unwrap_log(location!())
            .iter()
            .flat_map(|roles| roles.iter())
            .any(|role| role.id == admin_role_id.parse::<RoleId>().unwrap_or_default())
    })
}