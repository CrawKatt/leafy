use bon::Builder;
use poise::Command;
use serde::{Deserialize, Serialize};
use serenity::all::{ChannelId, GuildId, MessageId, UserId};
use std::collections::HashMap;
use surrealdb::{RecordId, Result as SurrealResult};

use crate::commands::ai::ask;
use crate::commands::audio::join::join;
use crate::commands::audio::leave::leave;
use crate::commands::audio::pause::pause;
use crate::commands::audio::play::play;
use crate::commands::audio::queue::queue;
use crate::commands::audio::resume::resume;
use crate::commands::audio::skip::skip;
use crate::commands::audio::stop::stop;
use crate::commands::fun::cat::cat_shh;
use crate::commands::fun::generate_dumb::dumb;
use crate::commands::fun::generate_furry::furry;
use crate::commands::fun::generate_pride::pride;
use crate::commands::fun::screenshot_this::screenshot_this;
use crate::commands::info::help::help;
use crate::commands::info::ping::ping;
use crate::commands::lessons::rust::rust;
use crate::commands::moderation::getters::get_admins::get_admins;
use crate::commands::moderation::getters::get_exception_channel::get_exception_channel;
use crate::commands::moderation::getters::get_forbidden_exception::get_forbidden_exception;
use crate::commands::moderation::getters::get_forbidden_role::get_forbidden_role;
use crate::commands::moderation::getters::get_forbidden_user::get_forbidden_user;
use crate::commands::moderation::getters::get_log_channel::get_log_channel;
use crate::commands::moderation::getters::get_ooc_channel::get_ooc_channel;
use crate::commands::moderation::getters::get_timeout_timer::get_timeout_timer;
use crate::commands::moderation::getters::get_welcome_channel::get_welcome_channel;
use crate::commands::moderation::setters::set_admins::set_admins;
use crate::commands::moderation::setters::set_autorole_id::set_autorole;
use crate::commands::moderation::setters::set_autorole_message::set_autorole_message;
use crate::commands::moderation::setters::set_exception_channel::set_exception_channel;
use crate::commands::moderation::setters::set_forbidden_exception::set_forbidden_exception;
use crate::commands::moderation::setters::set_forbidden_role::set_forbidden_role;
use crate::commands::moderation::setters::set_forbidden_user::set_forbidden_user;
use crate::commands::moderation::setters::set_log_channel::set_log_channel;
use crate::commands::moderation::setters::set_ooc_channel::set_ooc_channel;
use crate::commands::moderation::setters::set_timeout_message::set_time_out_message;
use crate::commands::moderation::setters::set_timeout_timer::set_timeout_timer;
use crate::commands::moderation::setters::set_warn_message::set_warn_message;
use crate::commands::moderation::setters::set_welcome_channel::set_welcome_channel;
use crate::commands::moderation::setters::set_welcome_message::set_welcome_message;
use crate::commands::translate::translate;
use crate::DB;

pub mod autocomplete;
pub mod config;
pub mod debug;
pub mod embeds;

#[allow(dead_code)]
pub struct Data {
    pub command_descriptions: HashMap<&'static str, String>
}

pub type CommandResult = Result<(), Error>;
pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

#[derive(Serialize, Deserialize, Debug, Clone, Builder)]
pub struct MessageData {
    pub message_content: String,
    pub author_id: UserId,
    pub id: Option<RecordId>,
    pub channel_id: ChannelId,
    pub guild_id: Option<GuildId>,
}

impl MessageData {
    pub async fn get_message_data(message_id: &MessageId) -> SurrealResult<Option<Self>> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let existing_data = DB
            .select(("messages", message_id.to_string()))
            .await?;

        Ok(existing_data)
    }

    pub async fn get_audio_data(message_id: &MessageId) -> SurrealResult<Option<Self>> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let existing_data = DB
            .select(("audio", message_id.to_string()))
            .await?;

        Ok(existing_data)
    }
}

#[derive(Serialize, Deserialize, Copy, Clone)]
pub struct Warns {
    pub user_id: UserId,
    pub warns: u8,
}

impl Warns {
    pub const fn new(user_id: UserId) -> Self {
        Self { user_id, warns: 0 }
    }

    /// todo: cambiar `WHERE user_id = $user_id` por `WHERE user_id = $user_id AND guild_id = $guild_id`?
    pub async fn get_warns(&self) -> SurrealResult<Option<Self>> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let sql_query = "SELECT * FROM warns WHERE user_id = $user_id";
        let user_id = self.user_id;
        
        let existing_data: Option<Self> = DB
            .query(sql_query)
            .bind(("user_id", user_id))
            .await?
            .take(0)?;

        Ok(existing_data)
    }

    pub async fn save_to_db(&self) -> SurrealResult<()> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let _created: Option<Self> = DB
            .create("warns")
            .content(*self)
            .await?;

        println!("Created warns: {:?}", self.warns);

        Ok(())
    }

    pub async fn add_warn(&self) -> SurrealResult<()> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let sql_query = "UPDATE warns SET warns = $warns WHERE user_id = $user_id";
        let warns = self.warns;
        let user_id = self.user_id;

        let _updated: Option<Self> = DB
            .query(sql_query)
            .bind(("warns", warns))
            .bind(("user_id", user_id))
            .await?
            .take(0)?;

        println!("Updated warns: {warns:?}");

        Ok(())
    }

    pub async fn reset_warns(&mut self) -> SurrealResult<()> {
        self.warns = 0;
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let sql_query = "UPDATE warns SET warns = $warns WHERE user_id = $user_id";
        
        let warns = self.warns;
        let user_id = self.user_id;
        
        let _updated: Option<Self> = DB
            .query(sql_query)
            .bind(("warns", warns))
            .bind(("user_id", user_id))
            .await?
            .take(0)?;

        println!("Updated warns: {warns:?}");

        Ok(())
    }
}

pub fn load_commands() -> Vec<Command<Data, Error>> {
    vec![
        ping(),
        set_admins(),
        set_log_channel(),
        set_ooc_channel(),
        set_warn_message(),
        set_timeout_timer(),
        set_forbidden_user(),
        set_forbidden_role(),
        set_welcome_message(),
        set_welcome_channel(),
        set_time_out_message(),
        set_forbidden_exception(),
        set_exception_channel(),
        set_autorole(),
        set_autorole_message(),
        get_admins(),
        get_log_channel(),
        get_ooc_channel(),
        get_timeout_timer(),
        get_forbidden_user(),
        get_forbidden_role(),
        get_welcome_channel(),
        get_exception_channel(),
        get_forbidden_exception(),
        screenshot_this(),
        pride(),
        furry(),
        join(),
        leave(),
        play(),
        skip(),
        resume(),
        pause(),
        stop(),
        queue(),
        help(),
        rust(),
        ask(),
        dumb(),
        cat_shh(),
        translate(),
    ]
}
