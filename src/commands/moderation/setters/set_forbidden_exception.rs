use crate::utils::debug::{IntoUnwrapResult, UnwrapResult};
use crate::utils::{CommandResult, Context};
use crate::DB;

use bon::Builder;
use serde::{Deserialize, Serialize};
use serenity::all::{GuildId, Permissions, UserId};
use std::panic::Location;
use surrealdb::opt::PatchOp;
use surrealdb::{RecordId, Result as SurrealResult};

#[derive(Serialize, Deserialize, Clone, Default, Debug, Builder)]
pub struct ForbiddenException {
    pub is_active: Option<bool>,
    pub id: Option<RecordId>
}

impl ForbiddenException {
    pub async fn save_to_db(self, guild_id: GuildId, user_id: UserId) -> SurrealResult<()> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let record_id = format!("{guild_id}_{user_id}");
        let _created: Option<Self> = DB
            .create(("forbidden_exception", record_id))
            .content(self)
            .await?;

        Ok(())
    }

    pub async fn verify_data(&self, guild_id: GuildId, user_id: UserId) -> UnwrapResult<Option<Self>> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let record_id = format!("{guild_id}_{user_id}");
        let existing_data = DB
            .select(("forbidden_exception", record_id))
            .await?;

        Ok(existing_data)
    }

    pub async fn switch(&mut self, guild_id: GuildId, user_id: UserId) -> UnwrapResult<()> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let Some(is_active) = self.is_active else {
            println!("No is_active value found {}", Location::caller());
            return Ok(())
        };

        let new_state = !is_active;
        let record_id = format!("{guild_id}_{user_id}");
        
        let _updated: Option<Self> = DB
            .update(("forbidden_exception", record_id))
            .patch(PatchOp::replace("/is_active", is_active))
            .await?;

        self.is_active = Some(new_state);

        Ok(())
    }

    pub async fn manual_switch(user_id: UserId, guild_id: GuildId, state: bool) -> SurrealResult<()> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let record_id = format!("{guild_id}_{user_id}");
        let _updated: Option<Self> = DB
            .update(("forbidden_exception", record_id))
            .patch(PatchOp::replace("/is_active", state))
            .await?;

        Ok(())
    }

    pub async fn have_exception(guild_id: GuildId, user_id: UserId) -> SurrealResult<Option<bool>> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let record_id = format!("{guild_id}_{user_id}");
        let existing_data: Option<Self> = DB
            .select(("forbidden_exception", record_id))
            .await?;

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
    ctx.defer().await?;
    let guild_id = ctx.guild_id().unwrap(); // SAFETY: Al estar el parámetro guild_only, la función solo se ejecutará en un servidor
    let guild = guild_id.to_guild_cached(&ctx).into_result()?.clone();
    let user_id = user.unwrap_or_else(|| ctx.author().id);
    let channel = ctx.guild_channel().await.into_result()?;

    if user_id != ctx.author().id {
        let member = guild_id.member(ctx.serenity_context(), ctx.author().id).await?;
        let member_permissions = guild.user_permissions_in(&channel, &member);
        if !member_permissions.contains(Permissions::ADMINISTRATOR) {
            poise::say_reply(ctx, "Debes ser administrador para cambiar la excepción de otros usuarios").await?;
            return Ok(());
        }
    }

    let mut data = ForbiddenException::builder()
        .is_active(state)
        .build();

    let existing_data = data.verify_data(guild_id, user_id).await?;
    let user = user_id.to_user(ctx.http()).await?;
    let username = user.name;

    if existing_data.is_none() {
        data.save_to_db(guild_id, user_id).await?;
        poise::say_reply(ctx, format!("User {username} has been set as a forbidden exception")).await?;
    } else {
        data.switch(guild_id, user_id).await?;
        poise::say_reply(ctx, format!("El usuario {username} ya ha solicitado una excepción. Actualizando")).await?;
    }

    Ok(())
}