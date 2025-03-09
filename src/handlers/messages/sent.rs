use std::sync::Arc;

use poise::serenity_prelude as serenity;
use serenity::all::{Context, EmojiId, Message, ReactionType, RoleId, UserId};

use crate::handlers::misc::attachment_case::attachment_handler;
use crate::handlers::misc::everyone_case::handle_everyone;
use crate::handlers::misc::forbidden_mentions::{handle_forbidden_role, handle_forbidden_user};
use crate::handlers::misc::link_spam_handler::{extract_link, spam_checker};
use crate::utils::config::GuildData;
use crate::utils::debug::IntoUnwrapResult;
use crate::utils::CommandResult;
use crate::utils::MessageData;
use crate::DB;

pub async fn handler(ctx: &Context, new_message: &Message) -> CommandResult {
    let processor = MessageProcessor::new();
    processor.process(ctx, new_message).await?;

    Ok(())
}

enum MessageState {
    Attachment,
    ForbiddenMention,
    Everyone,
    Spam,
    Logger,
}

impl MessageState {
    async fn handle(&self, ctx: &Context, message: &Message) -> CommandResult {
        match self {
            Self::Attachment => attachment(message).await?,
            Self::ForbiddenMention => forbidden_mention(ctx, message).await?,
            Self::Everyone => everyone_mention(ctx, message).await?,
            Self::Spam => spam(ctx, message).await?,
            Self::Logger => logger(message).await?,
        }

        Ok(())
    }
}

struct MessageProcessor {
    states: Vec<MessageState>,
}

impl MessageProcessor {
    fn new() -> Self {
        Self {
            states: vec![
                MessageState::Attachment,
                MessageState::ForbiddenMention,
                MessageState::Everyone,
                MessageState::Spam,
                MessageState::Logger,
            ],
        }
    }

    async fn process(&self, ctx: &Context, message: &Message) -> CommandResult {
        if message.author.bot {
            return Ok(());
        }

        if message.content == "<:HojaYo:1082385549450563584>" {
            let emoji = EmojiId::new(1_082_385_549_450_563_584);
            let react = ReactionType::Custom {
                id: emoji,
                animated: false,
                name: Some("HojaYo".to_string()),
            };

            message.react(&ctx.http, react).await?;
        }

        for state in &self.states {
            state.handle(ctx, message).await?;
        }

        Ok(())
    }
}

async fn attachment(message: &Message) -> CommandResult {
    if let Err(err) = attachment_handler(message).await {
        println!("Error manejando adjunto: {err:?}");
    }

    Ok(())
}

async fn forbidden_mention(ctx: &Context, message: &Message) -> CommandResult {
    let guild_id = message.guild_id.unwrap();
    let data = GuildData::verify_data(guild_id).await?.into_result()?;

    let forbidden_user_id = data.forbidden.user.into_result()?.parse::<UserId>()?;
    if message.mentions_user_id(forbidden_user_id) {
        handle_forbidden_user(ctx, message, guild_id, forbidden_user_id).await?;
        return Ok(());
    }

    let user_id = message.mentions.first().map(|user| user.id);
    let Some(user_id) = user_id else { return Ok(()) }; // si no hay mención salir de la función

    let forbidden_role_id = data.forbidden.role.into_result()?.parse::<RoleId>()?;
    let has_role = user_id
        .to_user(&ctx.http).await?
        .has_role(&ctx.http, guild_id, forbidden_role_id)
        .await?;

    if has_role {
        handle_forbidden_role(ctx, message, guild_id).await?;
    }

    Ok(())
}

async fn everyone_mention(ctx: &Context, message: &Message) -> CommandResult {
    if message.content.contains("@everyone") || message.content.contains("@here") {
        let guild_id = message.guild_id.unwrap();
        let data = GuildData::verify_data(guild_id).await?.into_result()?;
        let admin_role_id = data.admins.role;
        let mut member = guild_id.member(&ctx.http, message.author.id).await?;

        handle_everyone(admin_role_id.as_ref(), &mut member, ctx, data.time_out.time.into_result()?.parse()?, message).await?;
    }

    Ok(())
}

async fn spam(ctx: &Context, message: &Message) -> CommandResult {
    let content = Arc::new(message.content.clone());
    if extract_link(&content).is_some() {
        let guild_id = message.guild_id.unwrap();
        let data = GuildData::verify_data(guild_id).await?.into_result()?;
        spam_checker(&content, message.channel_id, data.admins.role.as_ref(), ctx, data.time_out.time.into_result()?.parse()?, message, guild_id).await?;
    }

    Ok(())
}

async fn logger(message: &Message) -> CommandResult {
    let data = MessageData::builder()
        .message_content(message.content.clone())
        .author_id(message.author.id)
        .guild_id(message.guild_id.unwrap())
        .channel_id(message.channel_id)
        .build();

    let _: Option<MessageData> = DB
        .create(("messages", message.id.to_string()))
        .content(data)
        .await?;

    Ok(())
}