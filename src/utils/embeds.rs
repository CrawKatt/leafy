use poise::serenity_prelude as serenity;
use serenity::builder::{CreateEmbed, CreateEmbedFooter};
use serenity::all::{ChannelId, CreateEmbedAuthor, CreateMessage, Message, User, UserId};

pub async fn send_embed(
    ctx: &serenity::Context,
    log_channel_id: ChannelId,
    delete_channel_id: &ChannelId,
    author_id: UserId,
    message_content: &String,
) -> Option<Message> {
    let author_mention = format!("<@{}>", author_id);
    let embed = CreateEmbed::default()
        .title("Mensaje eliminado")
        .description(format!("Autor del mensaje: {author_mention}\nCanal de origen: <#{delete_channel_id}>\nContenido del mensaje: {}", &message_content))
        .color(0x00ff00);

    log_channel_id.send_message(&ctx.http, create_message_embed(embed, Default::default())).await.ok()
}

pub async fn send_embed_if_mention(
    ctx: &serenity::Context,
    log_channel_id: ChannelId,
    delete_channel_id: &ChannelId,
    author_id: UserId,
    message_content: &String,
    user: User,
) -> Option<Message> {
    let author_user = author_id.to_user(&ctx.http).await.unwrap();
    let author_username = author_user.name.clone();
    let author_mention = format!("<@{}>", author_id);
    let user_mention = format!("<@{}>", user.id);
    let user_mention_bold = format!("**{}**", user.name);
    let message_content = message_content.replace(&user_mention,&user_mention_bold);

    let embed = CreateEmbed::default()
        .title("Mensaje eliminado")
        .description(format!("Autor del mensaje: {}\nCanal de origen: <#{delete_channel_id}>\nContenido del mensaje: {}", author_mention, &message_content))
        .author((|a: CreateEmbedAuthor| {
            a.name(author_username)
                .icon_url(author_user.avatar_url().unwrap())
        })(CreateEmbedAuthor::new(&author_user.name)))
        .color(0x00ff00)
        .footer((|f: CreateEmbedFooter| {
            f.text("Nota: si hay una parte del mensaje que está en \"Negrita\" significa que es una mención con \"@\" a esa persona.")
        })(CreateEmbedFooter::new("")));

    log_channel_id.send_message(&ctx.http, create_message_embed(embed, Default::default())).await.ok()
}

fn create_message_embed(embed: CreateEmbed, m: CreateMessage) -> CreateMessage {
    m.embed(embed)
}