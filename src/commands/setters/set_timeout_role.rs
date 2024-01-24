use serde::{Deserialize, Serialize};
use serenity::all::{GuildId, RoleId};
use crate::DB;
use crate::utils::autocomplete::args_set_role;
use surrealdb::Result as SurrealResult;
use crate::commands::setters::set_admins::AdminData;
use crate::utils::{CommandResult, Context};
use crate::utils::debug::UnwrapLog;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct RoleData {
    pub role_id: RoleId,
    pub guild_id: GuildId,
}

// CORREGIR ESTO, NO DEBERÍA SER USER SINO ROLE
impl RoleData {
    pub const fn new(role_id: RoleId, guild_id: GuildId) -> Self {
        Self { role_id, guild_id }
    }
    async fn save_to_db(&self) -> SurrealResult<()> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let created: Vec<Self> = DB
            .create("time_out_roles")
            .content(self)
            .await?;

        println!("Time Out Role Set Created: {created:?}");

        Ok(())
    }
    async fn update_in_db(&self) -> SurrealResult<()> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let sql_query = "UPDATE time_out_roles SET guild_id = $guild_id WHERE role_id = $role_id";
        let _updated: Vec<Self> = DB
            .query(sql_query)
            .bind(("guild_id", self.guild_id))
            .bind(("role_id", self.role_id))
            .await?
            .take(0)?;

        Ok(())
    }
    async fn verify_data(&self) -> SurrealResult<Option<Self>> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let sql_query = "SELECT * FROM time_out_roles WHERE role_id = $role_id";
        let existing_data: Option<Self> = DB
            .query(sql_query)
            .bind(("role_id", self.role_id))
            .await?
            .take(0)?;

        Ok(existing_data)
    }
}

/// Establece el rol de time out
#[poise::command(prefix_command, slash_command)]
pub async fn set_time_out_role(
    ctx: Context<'_>,
    #[autocomplete = "args_set_role"]
    #[description = "The user to set as the time out role"] role_id: RoleId,
) -> CommandResult {

    DB.use_ns("discord-namespace").use_db("discord").await?;
    let guild_id = ctx.guild_id().unwrap_log("Could not get the guild id: `set_time_out_role.rs` Line 66")?;
    let author = ctx.author();
    let owner = ctx.guild().unwrap().owner_id;
    let admin_role = AdminData::get_admin_role(guild_id).await?;

    let Some(role_id_result) = admin_role else {
        ctx.say("No admin role has been set").await?;
        return Ok(())
    };

    if !author.has_role(&ctx.serenity_context().http, guild_id, role_id_result).await? && author.id != owner {
        ctx.say("No tienes permisos para usar este comando").await?;
        return Ok(())
    }

    let data = RoleData::new(role_id, guild_id);

    // Consulta para verificar si el dato ya existe
    let existing_data = data.verify_data().await?;

    // Verificar si el dato existe
    let Some(_) = existing_data else {
        // Si el dato no existe, créalo
        data.save_to_db().await?;
        ctx.say(format!("Time out role establecido: <@&{role_id}>")).await?;
        return Ok(());
    };

    // Si el dato ya existe, actualízalo
    data.update_in_db().await?;

    ctx.say(format!("Time out role actualizado: <@&{role_id}>")).await?;

    Ok(())
}