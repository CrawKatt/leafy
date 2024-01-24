use std::sync::Arc;
use std::collections::HashMap;
use serenity::all::{CreateAttachment, GuildId, Http, Member, Mentionable, Message, Role, RoleId, UserId};
use poise::serenity_prelude as serenity;

use crate::{DB, log_handle};
use crate::utils::{CommandResult, Warns};
use crate::utils::MessageData;
use crate::commands::setters::set_admins::AdminData;
use crate::commands::setters::set_timeout_role::RoleData;
use crate::commands::setters::set_forbidden_user::ForbiddenUserData;
use crate::commands::setters::set_forbidden_role::ForbiddenRoleData;
use crate::commands::setters::set_timeout_message::TimeOutMessageData;
use crate::commands::setters::set_timeout_timer::SetTimeoutTimer;
use crate::commands::setters::set_warn_message::WarnMessageData;
use crate::utils::debug::UnwrapLog;

pub async fn message_handler(ctx: &serenity::Context, new_message: &Message) -> CommandResult {
    if new_message.author.bot {
        return Ok(());
    }

    // variable que busca la mención en el menssage_content si existe
    let message_content = &new_message.content;

    // variable que obtiene el id del servidor
    let guild_id = new_message.guild_id.unwrap_log("Could not get guild id: `sent_message.rs` line 25")?;

    // Obtener el canal de logs de la base de datos
    let data = MessageData::new(
        new_message.id,
        message_content.to_owned(),
        new_message.author.id,
        new_message.channel_id,
        new_message.guild_id,
    );

    // Si el mensaje no contiene una mención, guardar el mensaje en la base de datos
    // (NECESARIO PARA EVITAR EL PANIC)
    if !message_content.contains("<@") {
        let _created: Vec<MessageData> = DB.create("messages").content(data).await?;
        return Ok(());
    }

    let user_id = message_content
        .split("<@")
        .collect::<Vec<&str>>()[1]
        .split('>')
        .collect::<Vec<&str>>()[0]
        .parse::<u64>()?;

    let user = UserId::new(user_id).to_user(&ctx.http).await?;
    let forbidden_user_data = ForbiddenUserData::new(user, UserId::new(user_id), guild_id);
    let forbidden_user_id = forbidden_user_data.user_id;

    // Si el usuario prohibido de mencionar es mencionado, silenciar al autor del mensaje
    if new_message.mentions_user_id(forbidden_user_id) {
        handle_forbidden_user(ctx, new_message, guild_id, data, forbidden_user_id).await?;
        return Ok(());
    }

    // El primer RoleId(1) es un Default es por la creación del objeto, es innecesario y da igual
    let forbiden_role_data = ForbiddenRoleData::new(Role::default(), RoleId::default(), guild_id);
    let result = forbiden_role_data.get_role_id().await?;

    let Some(result) = result else {
        println!("No hay un rol prohibido de mencionar: `sent_message.rs` line 66");
        return Ok(())
    };

    let forbidden_role_id = result;
    let mentioned_user = guild_id.member(&ctx.http, user_id).await?;
    let mentioned_user_roles = mentioned_user.roles(&ctx.cache).unwrap_log("Could not get mentioned user roles: `sent_message.rs` line 65")?;

    // Si el usuario mencionado tiene el rol de prohibido de mencionar, silenciar al autor del mensaje
    if mentioned_user_roles.iter().any(|role| role.id == forbidden_role_id) {
        handle_forbidden_role(ctx, new_message, guild_id, data).await?;
        return Ok(());
    }

    let _created: Vec<MessageData> = DB.create("messages").content(data).await?;

    Ok(())
}

pub async fn handle_forbidden_role(
    ctx: &serenity::Context,
    new_message: &Message,
    guild_id: GuildId,
    data: MessageData
) -> CommandResult {
    let author_user_id = new_message.author.id;
    let member = guild_id.member(&ctx.http, author_user_id).await?;
    let sql_query = "SELECT * FROM time_out_roles WHERE guild_id = $guild_id";
    let time_out_role: Option<RoleData> = DB
        .query(sql_query)
        .bind(("guild_id", guild_id)) // pasar el valor
        .await?
        .take(0)?;

    let sql_query = "SELECT * FROM admins WHERE guild_id = $guild_id";
    let admin_role: Option<AdminData> = DB
        .query(sql_query)
        .bind(("guild_id", guild_id)) // pasar el valor
        .await?
        .take(0)?;

    let sql_query = "SELECT * FROM time_out_timer WHERE guild_id = $guild_id";
    let time_out_timer: Option<SetTimeoutTimer> = DB
        .query(sql_query)
        .bind(("guild_id", guild_id)) // pasar el valor
        .await?
        .take(0)?;

    let sql_query = "SELECT * FROM time_out_message WHERE guild_id = $guild_id";
    let time_out_message: Option<WarnMessageData> = DB
        .query(sql_query)
        .bind(("guild_id", guild_id)) // pasar el valor
        .await?
        .take(0)?;

    let Some(time_out_message) = time_out_message else {
        println!("No hay un mensaje de silencio: `sent_message.rs` line 122");
        return Ok(())
    };

    let time_out_message = time_out_message.warn_message;

    let Some(time_out_timer) = time_out_timer else {
        println!("No hay un tiempo de silencio establecido: `sent_message.rs` line 129");
        return Ok(())
    };

    let time_out_timer = time_out_timer.time;

    let Some(admin_role) = admin_role else {
        println!("No hay un admin establecido: `sent_message.rs` line 136");
        return Ok(())
    };

    let admin_role_id = admin_role.role_id;
    let admin_exception = admin_role_id.map_or(false, |admin_role_id| {
        member.roles(&ctx.cache)
            .unwrap_log("Could not get member roles: `sent_message.rs` line 112")
            .iter()
            .flat_map(|roles| roles.iter())
            .any(|role| role.id == admin_role_id)
    });

    if admin_exception {
        println!("Admin exception");
        return Ok(())
    }

    let Some(time_out_role) = time_out_role else {
        println!("No hay un rol de silencio establecido: `sent_message.rs` line 152");
        return Ok(())
    };

    let time_out_role_id = time_out_role.role_id;
    let mut warns = Warns::new(author_user_id);
    let existing_warns = warns.get_warns().await?;

    if let Some(mut existing_warns) = existing_warns {
        existing_warns.warns += 1;
        existing_warns.add_warn().await?;
        warns = existing_warns;
    } else {
        warns.warns += 1;
        warns.save_to_db().await?;
    }

    let mut message_map = HashMap::new();
    message_map.insert("content", format!("Mensaje eliminado por mencionar a un usuario prohibido de mencionar\nAdvertencia {}/3", warns.warns));
    let http = ctx.http.clone();
    http.send_message(new_message.channel_id, vec![], &message_map).await?;
    let member = guild_id.member(&ctx.http, author_user_id).await?;

    if warns.warns >= 3 {
        handle_warns(&member, new_message, time_out_role_id, message_map, &http, warns, time_out_timer, time_out_message).await?;
    }

    let _created: Vec<MessageData> = DB.create("messages").content(data).await?;
    let message = Some("Mensaje eliminado por mencionar a un usuario cuyo rol está prohíbido de mencionar");
    http.delete_message(new_message.channel_id, new_message.id, message).await?;

    Ok(())
}

pub async fn handle_forbidden_user(
    ctx: &serenity::Context,
    new_message: &Message,
    guild_id: GuildId,
    data: MessageData,
    forbidden_user_id: UserId
) -> CommandResult {
    let author_user_id = new_message.author.id;
    if author_user_id == forbidden_user_id {
        return Ok(())
    }

    let member = guild_id.member(&ctx.http, author_user_id).await?;
    let sql_query = "SELECT * FROM time_out_roles WHERE guild_id = $guild_id";
    let time_out_role: Option<RoleData> = DB
        .query(sql_query)
        .bind(("guild_id", guild_id)) // pasar el valor
        .await?
        .take(0)?;

    let sql_query = "SELECT * FROM admins WHERE guild_id = $guild_id";
    let admin_role: Option<AdminData> = DB
        .query(sql_query)
        .bind(("guild_id", guild_id)) // pasar el valor
        .await?
        .take(0)?;

    let sql_query = "SELECT * FROM time_out_timer WHERE guild_id = $guild_id";
    let time_out_timer: Option<SetTimeoutTimer> = DB
        .query(sql_query)
        .bind(("guild_id", guild_id)) // pasar el valor
        .await?
        .take(0)?;

    let sql_query = "SELECT * FROM warn_message WHERE guild_id = $guild_id";
    let warn_message: Option<WarnMessageData> = DB
        .query(sql_query)
        .bind(("guild_id", guild_id)) // pasar el valor
        .await?
        .take(0)?;

    let sql_query = "SELECT * FROM time_out_message WHERE guild_id = $guild_id";
    let time_out_message: Option<TimeOutMessageData> = DB
        .query(sql_query)
        .bind(("guild_id", guild_id)) // pasar el valor
        .await?
        .take(0)?;

    let time_out_message = time_out_message.unwrap_log("No se ha establecido un mensaje de silencio")?.time_out_message;
    let time_out_timer = time_out_timer.unwrap_log("No se ha establecido un tiempo de silencio")?.time;
    let admin_role_id = admin_role.clone().unwrap_log("No se ha establecido un rol de administrador")?.role_id;
    let admin_role_id_2 = admin_role.unwrap_log("No se ha establecido un rol de administrador")?.role_2_id;

    // Salir de la función si no hay un admin establecido
    if admin_role_id.is_none() {
        log_handle!("No hay un admin establecido: `sent_message.rs` line 196");
        return Ok(())
    }

    let admin_exception = admin_role_id.map_or(false, |admin_role_id| {
        member.roles(&ctx.cache)
            .unwrap_log("Could not get member roles: `sent_message.rs` line 112")
            .iter()
            .flat_map(|roles| roles.iter())
            .any(|role| role.id == admin_role_id)
    });

    let admin_exception_2 = admin_role_id_2.map_or(false, |admin_role_id_2| {
        member.roles(&ctx.cache)
            .unwrap_log("Could not get member roles: `sent_message.rs` line 206")
            .iter()
            .flat_map(|roles| roles.iter())
            .any(|role| role.id == admin_role_id_2)
    });

    if admin_exception || admin_exception_2 {
        return Ok(())
    }

    let time_out_role_id = time_out_role.unwrap_log("No se ha establecido un rol de timeout")?.role_id;
    let mut warns = Warns::new(author_user_id);
    let existing_warns = warns.get_warns().await?;

    if let Some(mut existing_warns) = existing_warns {
        existing_warns.warns += 1;
        existing_warns.add_warn().await?;
        warns = existing_warns;
    } else {
        warns.warns += 1;
        warns.save_to_db().await?;
    }

    let warn_message = warn_message.unwrap_log("No se ha establecido un mensaje de advertencia")?.warn_message;
    let mut message_map = HashMap::new();
    message_map.insert("content", format!("{warn_message}\nAdvertencia {}/3", warns.warns));
    let http = ctx.http.clone();
    let attachment = CreateAttachment::path("./assets/sugerencia.png").await?;
    http.send_message(new_message.channel_id, vec![attachment], &message_map).await?;

    message_map.insert("content", String::new());
    let attachment_mobile = CreateAttachment::path("./assets/sugerencia_mobile.png").await?;
    http.send_message(new_message.channel_id, vec![attachment_mobile], &message_map).await?;

    if warns.warns >= 3 {
        handle_warns(&member, new_message, time_out_role_id, message_map, &http, warns, time_out_timer, time_out_message).await?;
    }

    let _created: Vec<MessageData> = DB.create("messages").content(data).await?;
    http.delete_message(new_message.channel_id, new_message.id, None).await?;

    Ok(())
}

fn handle_time(member: Member, http: Arc<Http>, time_out_role_id: RoleId, time_out_timer: u64) {
    tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_secs(time_out_timer)).await;
        member.remove_role(&http, time_out_role_id).await.unwrap_or_default();
        println!("Desilenciado");
    });
}

async fn handle_warns(
    member: &Member,
    new_message: &Message,
    time_out_role_id: RoleId,
    mut message_map: HashMap<&str, String>,
    http: &Arc<Http>,
    mut warns: Warns,
    time_out_timer: u64,
    time_out_message: String,
) -> CommandResult {
    member.add_role(http, time_out_role_id).await?;
    message_map.insert("content", format!("{} {}", member.mention(), time_out_message));
    http.send_message(new_message.channel_id, vec![], &message_map).await?;
    warns.reset_warns().await?;
    handle_time(member.to_owned(), http.to_owned(), time_out_role_id, time_out_timer);

    Ok(())
}