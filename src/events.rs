use serenity::all::{ChannelId, CreateMessage, GuildId, Message, MessageId, User, UserId};
use serenity::builder::CreateEmbed;
use poise::serenity_prelude as serenity;
use serde::{Deserialize, Serialize};
use crate::DB;
use crate::utils::error::{Data, Error};

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct MessageData {
    pub message_id: MessageId,
    pub message_content: String,
    pub author_id: UserId,
    pub channel_id: ChannelId,
    pub guild_id: Option<GuildId>,
}

pub async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
) -> Result<(), Error> {
    DB.use_ns("discord-namespace").use_db("discord").await.unwrap_or_else(|why| {
        panic!("Could not use database: {why}");
    });

    match event {
        serenity::FullEvent::Ready { data_about_bot, .. } => {
            println!("Logged in as {}", data_about_bot.user.name);
        }

        serenity::FullEvent::Message { new_message } => {
            let data = MessageData {
                message_id: new_message.id,
                message_content: new_message.content.clone(),
                author_id: new_message.author.id,
                channel_id: new_message.channel_id,
                guild_id: new_message.guild_id,
            };

            if new_message.author.bot {
                return Ok(());
            }

            let _created: Vec<MessageData> = DB.create("messages").content(data).await.unwrap_or_else(|why| {
                panic!("Could not create message: {why}");
            });

            println!("Saved: {:#?}", _created);
        }

        serenity::FullEvent::MessageDelete { channel_id, deleted_message_id, .. } => {
            let sql_query = "SELECT * FROM messages WHERE message_id = $message_id";
            let database_info: Option<MessageData> = DB
                .query(sql_query)
                .bind(("message_id", deleted_message_id)) // pasar el valor
                .await.unwrap_or_else(|why| {
                    panic!("Could not query database: {why}");
                })
                .take(0).unwrap_or_else(|_| {
                    panic!("Could not get message data");
                });
            //println!("Message data: {:?}", database_info);

            let Some(database_message) = database_info else {
                return Ok(())
            };

            let message_content = database_message.message_content;
            let message_channel_id = database_message.channel_id;
            let author_id = database_message.author_id;

            let log_channel = ChannelId::new(1193595925503942688);
            if channel_id == &log_channel {
                return Ok(());
            }

            // variable que busca la mención en el menssage_content si existe
            let mention = message_content.find("<@");

            // convertir el mention en un objeto User
            let user_mentioned = match mention {
                Some(_) => {
                    let user_id = message_content
                        .split("<@")
                        .collect::<Vec<&str>>()[1]
                        .split(">")
                        .collect::<Vec<&str>>()[0]
                        .parse::<u64>()
                        .unwrap_or_else(|why| {
                            panic!("Could not parse user id {why}");
                        });

                    let user = UserId::new(user_id);
                    user.to_user(&ctx.http).await.unwrap()
                }
                None => User::default()
            };

            send_embed(
                ctx,
                log_channel,
                &message_channel_id,
                author_id,
                &message_content,
                user_mentioned,
            ).await;
        }

        _ => println!("Unhandled event: {:?}", event.snake_case_name())
    }

    Ok(())
}

async fn send_embed(
    ctx: &serenity::Context,
    log_channel_id: ChannelId,
    delete_channel_id: &ChannelId,
    author_id: UserId,
    message_content: &String,
    user: User,
) -> Option<Message> {

    let author_mention = format!("<@{}>", author_id);
    let user_mention = format!("<@{}>", user.id);
    let user_mention_bold = format!("**{}**", user.name);
    let message_content = message_content.replace(&user_mention,&user_mention_bold);
    let embed = CreateEmbed::default()
        .title("Mensaje eliminado")
        .description(format!("Autor del mensaje: {}\nCanal de origen: <#{delete_channel_id}>\nContenido del mensaje: {}", author_mention, &message_content))
        .color(0x00ff00);

    log_channel_id.send_message(&ctx.http, create_message_embed(embed, Default::default())).await.ok()
}

fn create_message_embed(embed: CreateEmbed, m: CreateMessage) -> CreateMessage {
    m.embed(embed)
}