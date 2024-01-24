use serenity::all::{GuildId, User, UserId};
use serde::{Deserialize, Serialize};
use surrealdb::Result as SurrealResult;
use crate::commands::setters::set_admins::AdminData;

use crate::DB;
use crate::utils::autocomplete::args_set_forbidden_user;
use crate::utils::{CommandResult, Context};
use crate::utils::debug::UnwrapLog;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ForbiddenUserData {
    pub user: User,
    pub user_id: UserId,
    pub guild_id: GuildId,
}

impl ForbiddenUserData {
    pub const fn new(user: User, user_id: UserId, guild_id: GuildId) -> Self {
        Self { user, user_id, guild_id }
    }
    pub async fn save_to_db(&self) -> SurrealResult<()> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let _created: Vec<Self> = DB
            .create("forbidden_users")
            .content(self)
            .await?;

        println!("Created forbidden user: {:?}", self.user_id);

        Ok(())
    }
    pub async fn update_in_db(&self) -> SurrealResult<()> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let sql_query = "UPDATE forbidden_users SET user_id = $user_id";
        let _updated: Vec<Self> = DB
            .query(sql_query)
            .bind(("user_id", &self.user_id))
            .await?
            .take(0)?;

        println!("Updated forbidden user: {:?}", self.user_id);

        Ok(())
    }
    pub async fn verify_data(&self) -> SurrealResult<Option<Self>> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let sql_query = "SELECT * FROM forbidden_users WHERE user_id = $user_id";
        let existing_data: Option<Self> = DB
            .query(sql_query)
            .bind(("user_id", &self.user_id))
            .await?
            .take(0)?;

        println!("Verified forbidden user: {:?}", self.user_id);

        Ok(existing_data)
    }
}

/// Establece el usuario prohibido de mencionar
#[poise::command(prefix_command, slash_command)]
pub async fn set_forbidden_user(
    ctx: Context<'_>,
    #[autocomplete = "args_set_forbidden_user"]
    #[description = "The user to set as the forbidden user"] user: User,
) -> CommandResult {
    DB.use_ns("discord-namespace").use_db("discord").await?;
    
    let guild_id = ctx.guild_id().unwrap_log("Could not get the guild id: `set_forbidden_user.rs` Line 70")?;
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

    let data = ForbiddenUserData::new(user.clone(), user.id, ctx.guild_id().unwrap());
    let existing_data = data.verify_data().await?;

    let Some(_) = existing_data else {
        data.save_to_db().await?;
        ctx.say(format!("Se ha prohibido mencionar a: **{}**", user.name)).await?;
        return Ok(())
    };

    data.update_in_db().await?;

    ctx.say(format!("Se ha prohibido mencionar a: **{}**", user.name)).await?;

    Ok(())
}