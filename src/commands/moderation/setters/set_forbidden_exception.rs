use std::panic::Location;
use serde::{Deserialize, Serialize};
use serenity::all::{GuildId, Permissions, UserId};
use surrealdb::Result as SurrealResult;
use crate::DB;
use crate::utils::{CommandResult, Context};

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct ForbiddenException {
    pub user_id: UserId,
    pub guild_id: GuildId,
    pub is_active: Option<bool>,
}

impl ForbiddenException {
    pub const fn new(user_id: UserId, guild_id: GuildId, is_active: bool) -> Self {
        Self {
            user_id,
            guild_id,
            is_active: Some(is_active),
        }
    }

    pub async fn save_to_db(&self) -> SurrealResult<()> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let _created: Vec<Self> = DB
            .create("forbidden_exception")
            .content(self)
            .await?;

        Ok(())
    }

    pub async fn verify_data(&self) -> SurrealResult<Option<Self>> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let sql_query = "SELECT * FROM forbidden_exception WHERE guild_id = $guild_id AND user_id = $user_id";
        let existing_data: Option<Self> = DB
            .query(sql_query)
            .bind(self)
            .await?
            .take(0)?;

        Ok(existing_data)
    }

    pub async fn switch(&mut self) -> SurrealResult<()> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let Some(is_active) = self.is_active else {
            println!("No is_active value found {}", Location::caller());
            return Ok(())
        };

        let sql_query = "UPDATE forbidden_exception SET is_active = $is_active WHERE guild_id = $guild_id AND user_id = $user_id";
        DB.query(sql_query).bind(&*self).await?;
        self.is_active = Some(!is_active);

        Ok(())
    }
    
    pub async fn manual_switch(user_id: UserId, guild_id: GuildId, state: bool) -> SurrealResult<()> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let sql_query = "UPDATE forbidden_exception SET is_active = $state WHERE guild_id = $guild_id AND user_id = $user_id";
        DB.query(sql_query)
            .bind(("state", state))
            .bind(("guild_id", guild_id))
            .bind(("user_id", user_id)).await?;

        Ok(())
    }

    pub async fn have_exception(user_id: UserId) -> SurrealResult<Option<bool>> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let sql_query = "SELECT * FROM forbidden_exception WHERE user_id = $user_id";
        let existing_data: Option<Self> = DB
            .query(sql_query)
            .bind(("user_id", user_id))
            .await?
            .take(0)?;

        let is_active = existing_data.map(|data| data.is_active);

        if let Some(is_active) = is_active {
            return Ok(is_active)
        }

        Ok(None)
    }
}

/// Establece una excepción para un usuario si este quiere ser mencionado con @
#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    category = "Moderator",
    ephemeral
)]
pub async fn set_forbidden_exception(
    ctx: Context<'_>,
    #[description = "The user id to set as a forbidden exception"] user: Option<UserId>,
    #[description = "The state to set for the forbidden exception"] state: bool
) -> CommandResult {
    let guild_id = ctx.guild_id().unwrap(); // SAFETY: Al estar el parámetro guild_only, la función solo se ejecutará en un servidor
    let user_id = user.unwrap_or(ctx.author().id);

    if user_id != ctx.author().id {
        let member = guild_id.member(ctx.serenity_context(), ctx.author().id).await?;
        let member_permissions = member.permissions(ctx.serenity_context())?;
        if !member_permissions.contains(Permissions::ADMINISTRATOR) {
            poise::say_reply(ctx, "Debes ser administrador para cambiar la excepción de otros usuarios").await?;
            return Ok(());
        }
    }

    let mut data = ForbiddenException::new(user_id, guild_id, state);
    let existing_data = data.verify_data().await?;
    let user = user_id.to_user(ctx.http()).await?;
    let username = user.name;

    if existing_data.is_none() {
        data.save_to_db().await?;
        poise::say_reply(ctx, format!("User {username} has been set as a forbidden exception")).await?;
    } else {
        data.switch().await?;
        poise::say_reply(ctx, format!("El usuario {username} ya ha solicitado una excepción. Actualizando")).await?;
    }

    Ok(())
}