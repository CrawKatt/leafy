use std::error::Error;
use std::fmt;
use serde::ser::StdError;
use crate::log_handle;
/*
use serenity::all::{GuildId, RoleId};
use crate::utils::{CommandResult, Context};
use surrealdb::Result as SurrealResult;
use crate::commands::setters::set_admins::AdminData;
use crate::commands::setters::set_forbidden_role::ForbiddenRoleData;
*/

#[derive(Debug)]
pub struct UnwrapLogError {
    msg: String,
}

impl fmt::Display for UnwrapLogError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "UnwrapLogError: {}", self.msg)
    }
}

impl Error for UnwrapLogError {}

pub trait UnwrapLog<T> {
    fn unwrap_log(self, msg: &str) -> Result<T, UnwrapLogError>;
}

impl<T: Default> UnwrapLog<T> for Option<T> {
    fn unwrap_log(self, msg: &str) -> Result<T, UnwrapLogError> {
        self.map_or_else(|| {
            log_handle!("{msg}");
            Err(UnwrapLogError { msg: msg.to_string() })
        }, |t| Ok(t))
    }
}

impl<T: Default, E: StdError> UnwrapLog<T> for Result<T, E> {
    fn unwrap_log(self, msg: &str) -> Result<T, UnwrapLogError> {
        match self {
            Ok(t) => Ok(t),
            Err(why) => {
                log_handle!("{msg}: {why}");
                Err(UnwrapLogError { msg: msg.to_string() })
            }
        }
    }
}
/*
pub trait AdminRoleCheck {
    async fn get_admin_role(guild_id: GuildId, sql_query: &str) -> SurrealResult<RoleId>;
    async fn check_admin_role(ctx: &Context<'_>, guild_id: GuildId, sql_query: &str) -> CommandResult {
        let admin_role = Self::get_admin_role(guild_id, sql_query).await.unwrap_log("Could not get the admin role: `debug.rs` Line 32");

        if !ctx.author().has_role(&ctx.serenity_context().http, guild_id, admin_role).await.unwrap_log("Could not get the admin role") {
            ctx.say("No tienes permisos para usar este comando").await?;
            return Ok(())
        }

        Ok(())
    }
}

impl AdminRoleCheck for AdminData {
    async fn get_admin_role(guild_id: GuildId, sql_query: &str) -> SurrealResult<RoleId> {
        get_role_id_from_db(guild_id, sql_query).await
    }
}

impl AdminRoleCheck for ForbiddenRoleData {
    async fn get_admin_role(guild_id: GuildId, sql_query: &str) -> SurrealResult<RoleId> {
       get_role_id_from_db(guild_id, sql_query).await
    }
}

async fn get_role_id_from_db(guild_id: GuildId, sql_query: &str) -> SurrealResult<RoleId> {
    DB.use_ns("discord-namespace").use_db("discord").await?;
    let existing_data: Option<AdminData> = DB
        .query(sql_query)
        .bind(("guild_id", &guild_id))
        .await?
        .take(0)?;

    Ok(existing_data.unwrap_log("Could not get existing data: `debug.rs` Line 56").role_id)
}
 */