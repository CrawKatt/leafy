use serde::{Deserialize, Serialize};
use serenity::all::{GuildId, Role, RoleId};
use crate::DB;
use crate::utils::{CommandResult, Context};
use surrealdb::Result as SurrealResult;
use crate::utils::autocomplete::args_set_forbidden_role;

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

    pub async fn get_role_id(&self) -> SurrealResult<Option<RoleId>> {
        DB.use_ns("discord-namespace").use_db("discord").await?;

        // Obtener el rol prphíbido de mencionar desde la base de datos
        // role.id porque `guild_id` es objeto de `role`
        let sql_query = "SELECT * FROM forbidden_roles WHERE role.guild_id = $guild_id";
        let existing_data: Option<Self> = DB
            .query(sql_query)
            .bind(("guild_id", &self.guild_id))
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
    let data = ForbiddenRoleData::new(role.clone(), role.id, GuildId::default());
    let existing_data = data.verify_data().await?;

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