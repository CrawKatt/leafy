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
    pub role_id: RoleId,
    pub guild_id: GuildId,
    pub role_2: Option<Role>,
    pub role_2_id: RoleId,
}

impl AdminData {
    pub const fn new(role: Role, role_2: Option<Role>, role_id: RoleId, role_2_id: RoleId, guild_id: GuildId) -> Self {
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
        let _created: Vec<Self> = DB
            .create("admins")
            .content(self)
            .await?;

        println!("Created admin role: {:?}", self.role_id);

        Ok(())
    }
    pub async fn update_in_db(&self) -> SurrealResult<()> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let sql_query = "UPDATE admins SET role_id = $role_id";
        let _updated: Option<Self> = DB
            .query(sql_query)
            .bind(("role_id", &self.role_id))
            .await?
            .take(0)?;

        println!("Updated admin role: {:?}", self.role_id);

        Ok(())
    }
    pub async fn verify_data(&self) -> SurrealResult<Option<Self>> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let sql_query = "SELECT * FROM admins WHERE role_id = $role_id";
        let existing_data: Option<Self> = DB
            .query(sql_query)
            .bind(("role_id", &self.role_id))
            .await?
            .take(0)?;

        println!("Verified admin role: {:?}", self.role_id);

        Ok(existing_data)
    }
    
    pub async fn get_admin_role(guild_id: GuildId) -> SurrealResult<RoleId> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let sql_query = "SELECT * FROM admins WHERE guild_id = $guild_id";
        let existing_data: Option<Self> = DB
            .query(sql_query)
            .bind(("guild_id", &guild_id))
            .await?
            .take(0)?;

        println!("Verified admin role: {existing_data:?}");

        Ok(existing_data.unwrap().role_id)
    }
}

#[poise::command(prefix_command, slash_command)]
pub async fn set_admins(
    ctx: Context<'_>,
    #[description = "The role to set as admin"]
    #[autocomplete = "args_set_admins"]
    role: Role,
    role_2: Option<Role>,
) -> CommandResult {
    DB.use_ns("discord-namespace").use_db("discord").await?;
    let guild_id = ctx.guild_id().unwrap();
    let role_id = role.id;

    let admin_data = AdminData::new(role.clone(), role_2.clone(), role_id, role_2.clone().unwrap_log("Could not get the second role").id, guild_id);
    let existing_data = admin_data.verify_data().await?;

    if existing_data.is_none() {
        admin_data.save_to_db().await?;
    } else {
        admin_data.update_in_db().await?;
    }

    let role_name = role.name;

    ctx.say(format!("Admin role set to: **{role_name}**")).await?;

    if let Some(role) = role_2 {
        let role_2_name = role.name;
        ctx.say(format!("Admin role set to: **{role_2_name}**")).await?;
    }

    Ok(())
}