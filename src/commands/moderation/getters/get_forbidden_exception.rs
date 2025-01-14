use serenity::all::UserId;
use crate::commands::moderation::setters::set_forbidden_exception::ForbiddenException;
use crate::utils::{CommandResult, Context};
use crate::utils::debug::{IntoUnwrapResult, UnwrapLog};
use crate::{DB, location};
use crate::utils::config::Getter;

/// Obtiene el estado de excepción de un usuario si es que tiene uno.
#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    category = "Moderator",
    ephemeral
)]
pub async fn get_forbidden_exception(
    ctx: Context<'_>,
    #[description = "The user id to get the forbidden exception"] user: Option<UserId>,
) -> CommandResult {
    ctx.defer().await?; // Necesario para que el Bot no devuelva un error de interacción si tarda mucho en responder
    let guild_id = ctx.guild_id().unwrap();
    let guild = guild_id.to_guild_cached(&ctx).into_result()?.clone();
    let channel = ctx.guild_channel().await.into_result()?;

    let author = guild_id.member(&ctx.serenity_context().http, ctx.author().id).await?;
    let author_permissions = guild.user_permissions_in(&channel, &author);
    // si el autor del comando no tiene permisos de moderador, obtener el estado de la excepción del usuario que ejecutó el comando
    if !author_permissions.manage_guild() {
        ctx.say("No tienes permisos para comprobar el estado de excepción de otros usuarios").await?;
        return Ok(())
    }

    let user_id = user.map_or_else(|| ctx.author().id, |user| user);
    let record_id = format!("{guild_id}_{user_id}");
    let existing_data : Option<ForbiddenException> = DB
        .select(("forbidden_exception", record_id))
        .await?;
    /*
    let sql_query = "SELECT * FROM forbidden_exception WHERE guild_id = $guild_id AND user_id = $user_id";
    let existing_data : Option<ForbiddenException> = DB
        .query(sql_query)
        .bind(("guild_id", guild_id))
        .bind(("user_id", user))
        .await?
        .take(0)?;
    */

    let existing_data = existing_data.unwrap_log(location!())?;
    let forbidden_user = existing_data.id.unwrap().to_id();
    let parts: Vec<&str> = forbidden_user.split('_').collect();
    let is_active = existing_data.is_active.unwrap_log(location!())?;
    let forbidden_user_id = parts[1].parse::<UserId>()?;

    let status = if is_active { "Activa" } else { "Inactiva" };
    let user = forbidden_user_id.to_user(ctx.http()).await?;

    if !is_active {
        ctx.say(format!("La excepción para el usuario **{}** está: **{}** ", user.name, status)).await?;
        return Ok(())
    }

    ctx.say(format!("El usuario **{}** ha solicitado una excepción. \nEstado de la excepción: **{}**", user.name, status)).await?;

    Ok(())
}