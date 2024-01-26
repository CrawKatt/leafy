use serde::{Deserialize, Serialize};
use serenity::all::{GuildId, Role, RoleId};
use crate::DB;
use crate::utils::{CommandResult, Context};
use surrealdb::Result as SurrealResult;
use crate::commands::setters::set_admins::AdminData;
use crate::utils::autocomplete::args_set_forbidden_role;
use crate::utils::debug::UnwrapLog;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ForbiddenRoleData {
    pub role: Role,
    pub role_id: RoleId,
    pub guild_id: GuildId,
}

impl ForbiddenRoleData {
    pub const fn new(role: Role, role_id: RoleId, guild_id: GuildId) -> Self {
        Self {
            role,
            role_id,
            guild_id
        }
    }
    pub async fn save_to_db(&self) -> SurrealResult<()> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let _created: Vec<Self> = DB
            .create("forbidden_roles")
            .content(self)
            .await?;

        println!("Created forbidden role: {:?}", self.role_id);

        Ok(())
    }
    pub async fn update_in_db(&self) -> SurrealResult<()> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let sql_query = "UPDATE forbidden_roles SET role_id = $role_id";
        let _updated: Option<Self> = DB
            .query(sql_query)
            .bind(("role_id", &self.role_id))
            .await?
            .take(0)?;

        println!("Updated forbidden role: {:?}", self.role_id);

        Ok(())
    }
    pub async fn verify_data(&self) -> SurrealResult<Option<Self>> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let sql_query = "SELECT * FROM forbidden_roles WHERE role_id = $role_id";
        let existing_data: Option<Self> = DB
            .query(sql_query)
            .bind(("role_id", &self.role_id))
            .await?
            .take(0)?;

        println!("Verified forbidden role: {:?}", self.role_id);

        Ok(existing_data)
    }

    pub async fn get_role_id(guild_id: GuildId) -> SurrealResult<Option<RoleId>> {
        DB.use_ns("discord-namespace").use_db("discord").await?;

        // Obtener el rol prphíbido de mencionar desde la base de datos
        // role.id porque `guild_id` es objeto de `role`
        let sql_query = "SELECT * FROM forbidden_roles WHERE role.guild_id = $guild_id";
        let existing_data: Option<Self> = DB
            .query(sql_query)
            .bind(("guild_id", &guild_id))
            .await?
            .take(0)?;

        Ok(existing_data.map(|data| data.role_id))
    }

}

/// Establece el rol de usuario prohibido de mencionar
#[poise::command(prefix_command, slash_command)]
pub async fn set_forbidden_role(
    ctx: Context<'_>,
    #[autocomplete = "args_set_forbidden_role"]
    #[description = "The role to set as the forbidden role"] role: Role,
) -> CommandResult {
    DB.use_ns("discord-namespace").use_db("discord").await?;
    let guild_id = ctx.guild_id().unwrap_log("Could not get the guild_id", line!(), module_path!())?;
    let data = ForbiddenRoleData::new(role.clone(), role.id, GuildId::default());
    let existing_data = data.verify_data().await?;
    let author = ctx.author();
    let owner = ctx.guild().unwrap().owner_id;
    let admin_role = AdminData::get_admin_role(guild_id).await?;

    let Some(admin_role) = admin_role else {
        ctx.say("No admin role has been set").await?;
        return Ok(())
    };

    if !author.has_role(&ctx.serenity_context().http, guild_id, admin_role).await? && author.id != owner {
        ctx.say("No tienes permisos para usar este comando").await?;
        return Ok(())
    }

    match existing_data {
        Some(_) => {
            data.update_in_db().await?;
        }
        None => {
            data.save_to_db().await?;
        }
    }

    let message = format!("Set forbidden role to: **{}**", role.name);
    ctx.say(message).await?;

    Ok(())
}