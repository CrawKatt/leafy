use std::sync::Arc;
use std::collections::HashMap;
use serenity::all::{GuildId, Http, Member, Message, Role, RoleId, UserId};
use poise::serenity_prelude as serenity;
use serde::{Deserialize, Serialize};
use crate::commands::set_forbidden_role::ForbiddenRoleData;
use crate::DB;
use crate::utils::CommandResult;
use crate::utils::MessageData;
use crate::commands::set_timeout_role::RoleData;
use crate::commands::set_forbidden_user::ForbiddenUserData;
use surrealdb::Result as SurrealResult;
pub async fn message_handler(ctx: &serenity::Context, new_message: &Message) -> CommandResult {
    if new_message.author.bot {
        return Ok(());
    }

    // variable que busca la mención en el menssage_content si existe
    let message_content = &new_message.content;

    // variable que obtiene el id del servidor
    let guild_id = new_message.guild_id.unwrap_or_default();

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
        handle_forbidden_user(ctx, new_message, data).await?;
        return Ok(());
    }

    // El primer RoleId(1) es un Default es por la creación del objeto, es innecesario y da igual
    let forbiden_role_data = ForbiddenRoleData::new(Role::default(), RoleId::default(), guild_id);
    let result = forbiden_role_data.get_role_id().await?;
    let forbidden_role_id = result.unwrap_or_default();
    let mentioned_user = guild_id.member(&ctx.http, user_id).await?;
    let mentioned_user_roles = mentioned_user.roles(&ctx.cache).unwrap_or_default();

    // Si el usuario mencionado tiene el rol de prohibido de mencionar, silenciar al autor del mensaje
    if mentioned_user_roles.iter().any(|role| role.id == forbidden_role_id) {
        handle_forbidden_role(ctx, new_message, guild_id, data).await?;
        return Ok(());
    }

    let _created: Vec<MessageData> = DB.create("messages").content(data).await?;

    Ok(())
}

async fn handle_forbidden_role(
    ctx: &serenity::Context,
    new_message: &Message,
    guild_id: GuildId,
    data: MessageData
) -> CommandResult {
    let author_user_id = new_message.author.id;
    let sql_query = "SELECT * FROM time_out_roles WHERE guild_id = $guild_id";
    let time_out_role: Option<RoleData> = DB
        .query(sql_query)
        .bind(("guild_id", guild_id)) // pasar el valor
        .await?
        .take(0)?;

    let time_out_role_id = time_out_role.unwrap_or_default().role_id;

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
        member.add_role(&ctx.http, time_out_role_id).await?;
        println!("Silenciado");
        warns.reset_warns().await?;
    }

    let _created: Vec<MessageData> = DB.create("messages").content(data).await?;
    let message = Some("Mensaje eliminado por mencionar a un usuario cuyo rol está prohíbido de mencionar");
    http.delete_message(new_message.channel_id, new_message.id, message).await?;
    handle_time(member, http, time_out_role_id);

    Ok(())
}

#[derive(Serialize, Deserialize)]
struct Warns {
    user_id: UserId,
    warns: u8,
}

impl Warns {
    pub const fn new(user_id: UserId) -> Self {
        Self { user_id, warns: 0 }
    }

    pub async fn get_warns(&self) -> SurrealResult<Option<Self>> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let sql_query = "SELECT * FROM warns WHERE user_id = $user_id";
        let existing_data: Option<Self> = DB
            .query(sql_query)
            .bind(("user_id", &self.user_id))
            .await?
            .take(0)?;

        Ok(existing_data)
    }

    pub async fn save_to_db(&self) -> SurrealResult<()> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let _created: Vec<Self> = DB
            .create("warns")
            .content(self)
            .await?;

        println!("Created warns: {:?}", self.warns);

        Ok(())
    }

    pub async fn add_warn(&mut self) -> SurrealResult<()> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let sql_query = "UPDATE warns SET warns = $warns WHERE user_id = $user_id";
        let _updated: Vec<Self> = DB
            .query(sql_query)
            .bind(("warns", &self.warns))
            .bind(("user_id", &self.user_id))
            .await?
            .take(0)?;

        println!("Updated warns: {:?}", self.warns);

        Ok(())
    }

    pub async fn reset_warns(&mut self) -> SurrealResult<()> {
        self.warns = 0;
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let sql_query = "UPDATE warns SET warns = $warns WHERE user_id = $user_id";
        let _updated: Vec<Self> = DB
            .query(sql_query)
            .bind(("warns", &self.warns))
            .bind(("user_id", &self.user_id))
            .await?
            .take(0)?;

        println!("Updated warns: {:?}", self.warns);

        Ok(())
    }
}

async fn handle_forbidden_user(
    ctx: &serenity::Context,
    new_message: &Message,
    data: MessageData
) -> CommandResult {
    let author_user_id = new_message.author.id;
    let Some(guild_id) = new_message.guild_id else {
        println!("Failed to get guild id");
        return Ok(());
    };

    let member = guild_id.member(&ctx.http, author_user_id).await?;
    let sql_query = "SELECT * FROM time_out_roles WHERE guild_id = $guild_id";
    let time_out_role: Option<RoleData> = DB
        .query(sql_query)
        .bind(("guild_id", guild_id)) // pasar el valor
        .await?
        .take(0)?;

    let time_out_role_id = time_out_role.unwrap_or_default().role_id;

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

    if warns.warns >= 3 {
        member.add_role(&ctx.http, time_out_role_id).await?;
        println!("Silenciado");
        warns.reset_warns().await?;
    }

    let _created: Vec<MessageData> = DB.create("messages").content(data).await?;
    http.delete_message(new_message.channel_id, new_message.id, None).await?;

    handle_time(member, http, time_out_role_id);

    Ok(())
}

fn handle_time(member: Member, http: Arc<Http>, time_out_role_id: RoleId) {
    tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
        member.remove_role(&http, time_out_role_id).await.unwrap_or_else(|why| {
            println!("Failed to remove role: {why:?}");
        });
        println!("Desilenciado");
    });
}