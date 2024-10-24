use serenity::all::{Member, RoleId};
use poise::serenity_prelude as serenity;
use crate::location;
use crate::utils::debug::UnwrapLog;

/// Verifica si el usuario tiene un rol de administrador
/// Si el usuario tiene un rol de administrador, no se silenciará
pub fn check_admin_exception(admin_role_id: &str, member: &Member, ctx: &serenity::Context) -> bool {
    // comprobar si el usuario tiene un rol de administrador
    admin_role_id.parse::<RoleId>().ok().map_or(false, |admin_role_id| {
        member.roles(&ctx.cache)
            .unwrap_log(location!())
            .iter()
            .flat_map(|roles| roles.iter())
            .any(|role| role.id == admin_role_id)
    })
}