use serenity::all::{GuildId, Role, RoleId};
use serde::{Deserialize, Serialize};
use surrealdb::Result as SurrealResult;

use crate::DB;
use crate::utils::{CommandResult, Context};
use crate::utils::autocomplete::args_set_admins;
use crate::utils::debug::UnwrapLog;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct AdminData {
    pub role: Role,
    pub role_2: Option<Role>,
    pub role_id: Option<RoleId>,
    pub role_2_id: Option<RoleId>,
    pub guild_id: GuildId,
}

impl AdminData {
    pub const fn new(role: Role, role_2: Option<Role>, role_id: Option<RoleId>, role_2_id: Option<RoleId>, guild_id: GuildId) -> Self {
        Self {
            role,
            role_2,
            role_id,
            role_2_id,
            guild_id
        }
    }
    pub async fn save_to_db(&self) -> SurrealResult<()> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let sql_query = "SELECT * FROM admins WHERE guild_id = $guild_id";
        let existing_data: Option<Self> = DB
            .query(sql_query)
            .bind(("guild_id", &self.guild_id))
            .await?
            .take(0)?;

        if existing_data.is_none() {
            let _created: Vec<Self> = DB
                .create("admins")
                .content(self)
                .await?;

            println!("Created admin role: {:?}", self.role_id);
        } else {
            let sql_query = "UPDATE admins SET role_id = $role_id, role_2_id = $role_2_id WHERE guild_id = $guild_id";
            let _updated: Option<Self> = DB
                .query(sql_query)
                .bind(("role_id", &self.role_id))
                .bind(("role_2_id", &self.role_2_id))
                .bind(("guild_id", &self.guild_id))
                .await?
                .take(0)?;

            println!("Updated admin role: {:?}", self.role_id);
        }

        Ok(())
    }
    pub async fn get_admin_role(guild_id: GuildId) -> SurrealResult<Option<RoleId>> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let sql_query = "SELECT * FROM admins WHERE guild_id = $guild_id";
        let query_result = DB
            .query(sql_query)
            .bind(("guild_id", &guild_id))
            .await;

        let existing_data: Option<Self> = match query_result {
            Ok(mut result) => result.take(0)?,
            Err(why) => return Err(why)
        };

        let Some(data) = existing_data else {
            println!("Admin role: None");
            return Ok(None)
        };

        println!("Admin role: {:?} : `set_admins.rs` Line 75", data.role_id);
        Ok(data.role_id)
    }
}

#[poise::command(prefix_command, slash_command)]
pub async fn set_admins(
    ctx: Context<'_>,
    #[description = "The role to set as admin"]
    #[autocomplete = "args_set_admins"]
    role: Option<Role>,
    role_2: Option<Role>,
) -> CommandResult {
    DB.use_ns("discord-namespace").use_db("discord").await?;
    let guild_id = ctx.guild_id().unwrap_log("Could not get the guild_id: `set_admins.rs` Line 91")?;

    // si el autor no tiene el rol de administrador, no puede usar el comando
    let author = ctx.author();
    let owner = ctx.guild().unwrap().owner_id;
    let admin_role = AdminData::get_admin_role(guild_id).await?;

    let Some(admin_role) = admin_role else {
        if author.id != owner {
            ctx.say("No tienes permisos para usar este comando").await?;
            return Ok(())
        }
        
        let role_id = role.as_ref().map(|role| role.id);
        let role_2_id = role_2.as_ref().map(|role| role.id);
        let role = role.unwrap_log("Could not get the role: `set_admins.rs` Line 107")?;
        let admin_data = AdminData::new(role.clone(), role_2, role_id, role_2_id, guild_id);
        admin_data.save_to_db().await?;
        let role_name = role.name;
        ctx.say(format!("No se ha establecido un rol de administrador anteriormente.\nSe ha establecido como administrador a: **{role_name}**")).await?;

        return Ok(())
    };

    if !author.has_role(&ctx.serenity_context().http, guild_id, admin_role).await? && author.id != owner {
        ctx.say("No tienes permisos para usar este comando").await?;
        return Ok(())
    }

    let role_id = role.as_ref().map(|role| role.id);
    let role_2_id = role_2.as_ref().map(|role| role.id);
    let admin_data = AdminData::new(role.clone().unwrap_log("Could not get the role: `set_admins.rs` Line 99")?, role_2.clone(), role_id, role_2_id, guild_id);
    admin_data.save_to_db().await?;

    let role_name = role.unwrap_log("Could not get the role_name: `set_admins.rs` Line 102")?.name;
    ctx.say(format!("Admin role set to: **{role_name}**")).await?;
    
    let role_2 = role_2.unwrap_log("Could not get the role: `set_admins.rs` Line 105")?;
    let role_2_name = role_2.name;
    ctx.say(format!("Admin role set to: **{role_2_name}**")).await?;

    Ok(())
}