use serenity::all::{Member, RoleId};
use poise::serenity_prelude as serenity;
use crate::location;
use crate::utils::debug::UnwrapLog;

/// Verifica si el usuario tiene un rol de administrador
/// Si el usuario tiene un rol de administrador, no se silenciará
pub fn check_admin_exception(admin_role_ids: Option<&Vec<String>>, member: &Member, ctx: &serenity::Context) -> bool {
    admin_role_ids.is_some_and(|admin_role_ids| {
        let admin_role_ids: Vec<RoleId> = admin_role_ids
            .iter()
            .filter_map(|id| id.parse::<RoleId>().ok())
            .collect();

        member.roles(&ctx.cache)
            .unwrap_log(location!())
            .iter()
            .flat_map(|roles| roles.iter())
            .any(|role| admin_role_ids.contains(&role.id))
    })
}