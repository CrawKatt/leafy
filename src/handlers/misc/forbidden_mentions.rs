use std::collections::HashMap;
use std::panic::Location;

use poise::serenity_prelude as serenity;
use serenity::all::{GuildId, Message, UserId};

use crate::{DB, log_handle};
use crate::commands::moderation::setters::set_forbidden_exception::ForbiddenException;
use crate::utils::{CommandResult, MessageData, Warns};
use crate::handlers::misc::exceptions::check_admin_exception;
use crate::handlers::misc::warns::handle_warn_system;
use crate::utils::config::GuildData;
use crate::utils::debug::IntoUnwrapResult;
use crate::utils::embeds::send_warn_embed;

pub async fn handle_forbidden_user(
    ctx: &serenity::Context,
    new_message: &Message,
    guild_id: GuildId,
    data: &MessageData,
    forbidden_user_id: UserId
) -> CommandResult {
    let author_user_id = new_message.author.id;
    if author_user_id == forbidden_user_id {
        return Ok(())
    }

    let forbidden_user_exception = ForbiddenException::have_exception(forbidden_user_id).await?;
    if let Some(forbidden_user_exception) = forbidden_user_exception {
        if forbidden_user_exception {
            println!("El usuario ha solicitado una excepción : {}", Location::caller());
            return Ok(())
        }
    }

    if !new_message.mentions_user_id(forbidden_user_id) {
        println!("No se ha mencionado al usuario prohibido : {}", Location::caller());
        return Ok(())
    }

    let mut member = guild_id.member(&ctx.http, author_user_id).await?;
    let time_out_timer = GuildData::verify_data(guild_id).await?
        .into_result()?
        .time_out_config
        .time
        .into_result()?
        .parse::<i64>()?;
    
    let warn_message = GuildData::verify_data(guild_id).await?
        .into_result()?
        .messages_config
        .warn
        .unwrap_or_else(|| {
            log_handle!("No se ha establecido un mensaje de advertencia: `sent_message.rs` {}", Location::caller());
            "Por favor no hagas @ a este usuario. Si estás respondiendo un mensaje, considera responder al mensaje sin usar @".to_string()
        });
    
    let warn_message = format!("{} {warn_message}", member.distinct());
    let time_out_message = GuildData::verify_data(guild_id).await?
        .into_result()?
        .messages_config
        .time_out
        .unwrap_or_else(|| {
            log_handle!("No se ha establecido un mensaje de silencio: {}", Location::caller());
            "Has sido silenciado por mencionar a un usuario cuyo rol está prohibido de mencionar".to_string()
        });
    
    let admin_role_id = GuildData::verify_data(guild_id).await?
        .into_result()?
        .admins
        .role_id;
    
    let admin_role_id_2 = GuildData::verify_data(guild_id).await?
        .into_result()?
        .admins
        .role_2_id;

    // Salir de la función si no hay un admin establecido
    if admin_role_id.is_none() {
        log_handle!("No hay un admin establecido: {}", Location::caller());
        return Ok(())
    }

    let admin_exception = check_admin_exception(admin_role_id, &member, ctx);
    let admin_exception_2 = check_admin_exception(admin_role_id_2, &member, ctx);

    if admin_exception || admin_exception_2 {
        println!("Admin exception : {}", Location::caller());
        return Ok(())
    }

    let mut warns = Warns::new(author_user_id);
    let existing_warns = warns.get_warns().await?;
    warns_counter(&mut warns, existing_warns).await?;
    let channel_id = new_message.channel_id;
    let warnings = warns.warns;
    send_warn_embed(ctx, warnings, "./assets/sugerencia.png", channel_id, &warn_message).await?;
    let message_map = HashMap::new();
    let http = ctx.http.clone();
    if warns.warns >= 3 {
        handle_warn_system(&mut member, new_message, message_map, &http, warns, time_out_timer, time_out_message).await?;
    }
    
    let _created: Vec<MessageData> = DB.create("messages").content(data).await?;
    http.delete_message(new_message.channel_id, new_message.id, None).await?;

    Ok(())
}

pub async fn handle_forbidden_role(
    ctx: &serenity::Context,
    new_message: &Message,
    guild_id: GuildId,
) -> CommandResult {
    let author_user_id = new_message.author.id;
    let member = guild_id.member(&ctx.http, author_user_id).await?;
    let admin_role_id = GuildData::verify_data(guild_id).await?
        .into_result()?
        .admins
        .role_id;
    
    let time_out_timer = GuildData::verify_data(guild_id).await?
        .into_result()?
        .time_out_config
        .time
        .into_result()?
        .parse::<i64>()?;

    let warn_message = GuildData::verify_data(guild_id).await?
        .into_result()?
        .messages_config
        .warn
        .unwrap_or_else(|| {
            log_handle!("No se ha establecido un mensaje de advertencia: `sent_message.rs` {}", Location::caller());
            format!("{} Por favor no hagas @ a este usuario. Si estás respondiendo un mensaje, considera responder al mensaje sin usar @", member.distinct())
        });

    let warn_message = format!("{} {warn_message}", member.distinct());
    let time_out_message = GuildData::verify_data(guild_id).await?
        .into_result()?
        .messages_config
        .time_out
        .unwrap_or_else(|| {
            log_handle!("No se ha establecido un mensaje de silencio: {}", Location::caller());
            format!("{} Has sido silenciado por mencionar a un usuario cuyo rol está prohibido de mencionar", member.distinct())
        });

    let admin_exception = check_admin_exception(admin_role_id, &member, ctx);
    if admin_exception {
        println!("Admin exception : {}", Location::caller());
        return Ok(())
    }

    let mut warns = Warns::new(author_user_id);
    let existing_warns = warns.get_warns().await?;
    warns_counter(&mut warns, existing_warns).await?;
    let channel_id = new_message.channel_id;
    send_warn_embed(ctx, warns.warns, "./assets/sugerencia.png", channel_id, &warn_message).await?;
    let message_map = HashMap::new();
    let http = ctx.http.clone();
    let mut member = guild_id.member(&ctx.http, author_user_id).await?;

    if warns.warns >= 3 {
        handle_warn_system(&mut member, new_message, message_map, &http, warns, time_out_timer, time_out_message).await?;
    }
    
    http.delete_message(new_message.channel_id, new_message.id, None).await?;

    Ok(())
}

async fn warns_counter(warns: &mut Warns, existing_warns: Option<Warns>) -> CommandResult {
    if let Some(mut existing_warns) = existing_warns {
        existing_warns.warns += 1;
        existing_warns.add_warn().await?;
        *warns = existing_warns;
    } else {
        warns.warns += 1;
        warns.save_to_db().await?;
    }

    Ok(())
}