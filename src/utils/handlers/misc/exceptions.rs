use serenity::all::{Member, RoleId};
use poise::serenity_prelude as serenity;
use crate::utils::misc::debug::UnwrapLog;

const CURRENT_MODULE: &str = file!();

/// Verifica si el usuario tiene un rol de administrador
/// Si el usuario tiene un rol de administrador, no se silenciará
pub fn check_admin_exception(admin_role_id: Option<String>, member: &Member, ctx: &serenity::Context) -> bool {
    admin_role_id.map_or(false, |admin_role_id| {
        member.roles(&ctx.cache)
            .unwrap_log("Could not get member roles", CURRENT_MODULE, line!())
            .iter()
            .flat_map(|roles| roles.iter())
            .any(|role| role.id == RoleId::new(admin_role_id.parse::<u64>().unwrap_or_default()))
    })
}