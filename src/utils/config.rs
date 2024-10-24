use std::fs;
use bon::bon;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub channels: Channel,
    pub admins: Admin,
    pub forbidden: Forbidden,
    pub messages: Message,
    pub time_out: Timeout,
}

#[bon]
impl Config {
    #[builder]
    const fn new(
        channels: Channel,
        admins: Admin,
        forbidden: Forbidden,
        messages: Message,
        time_out: Timeout,
    ) -> Self {
        Self { channels, admins, forbidden, messages, time_out }
    }
}

#[derive(Debug, Deserialize)]
pub struct Channel {
    #[serde(rename = "meica_chat")]
    pub exceptions: String,
    pub logs: String,
    pub ooc: String,
    pub welcome: String,
}

#[bon]
impl Channel {
    #[builder]
    const fn new(exceptions: String, logs: String, ooc: String, welcome: String) -> Self {
        Self { exceptions, logs, ooc, welcome }
    }
}

#[derive(Debug, Deserialize)]
pub struct Admin {
    #[serde(rename = "arquitecto")]
    pub role: String,
    #[serde(rename = "jardinero")]
    pub role_2: String,
}

#[bon]
impl Admin {
    #[builder]
    const fn new(role: String, role_2: String) -> Self {
        Self { role, role_2 }
    }
}

#[derive(Debug, Deserialize)]
pub struct Forbidden {
    #[serde(rename = "chikistrikis")]
    pub role: String,
    #[serde(rename = "meica")]
    pub user: String,
}

#[bon]
impl Forbidden {
    #[builder]
    const fn new(role: String, user: String) -> Self {
        Self { role, user }
    }
}

#[derive(Debug, Deserialize)]
pub struct Message {
    #[serde(rename = "time_out")]
    pub time_out: String,
    pub warn: String,
    pub welcome: String,
}

#[bon]
impl Message {
    #[builder]
    const fn new(time_out: String, warn: String, welcome: String) -> Self {
        Self { time_out, warn, welcome }
    }
}

#[derive(Debug, Deserialize)]
pub struct Timeout {
    pub time: String,
}

#[bon]
impl Timeout {
    #[builder]
    const fn new(time: String) -> Self {
        Self { time }
    }
}

pub fn load_data() -> Config {
    let config_data = if cfg!(debug_assertions) {
        // Si estamos en modo `Debug`, cargar el archivo `ConfigDev.toml`
        fs::read_to_string("ConfigDev.toml").expect("No se pudo leer el archivo ConfigDev.toml")
    } else {
        // Si estamos en modo `Release`, cargar el archivo `Config.toml`
        fs::read_to_string("Config.toml").expect("No se pudo leer el archivo Config.toml")
    };

    let config: Config = toml::from_str(&config_data).expect("No se pudo parsear el archivo TOML");

    // Devolver la configuración usando el builder en ambos casos
    Config::builder()
        .channels(config.channels)
        .admins(config.admins)
        .forbidden(config.forbidden)
        .messages(config.messages)
        .time_out(config.time_out)
        .build()
}

/*
use serenity::all::GuildId;
use serde::{Deserialize, Serialize};
use surrealdb::Result as SurrealResult;

use crate::DB;
use crate::utils::debug::UnwrapResult;

/// # Crea e implementa una estructura de configuración
///
/// - Crea una estructura de configuración con los campos que se le pasen
/// - Implementa un método para cada campo que permita modificarlo
/// - Simplifica la creación de métodos para seguir el patrón de diseño Builder
macro_rules! obj {
    ($name:ident, $($field:ident: $type:ty),*) => {
        #[derive(Serialize, Deserialize, Debug, Clone, Default)]
        pub struct $name {
            $(pub $field: Option<$type>,)*
        }

        impl $name {
            $(
                pub fn $field(mut self, $field: impl Into<$type>) -> Self {
                    self.$field = Some($field.into());
                    self
                }
            )*

            pub async fn update_field_in_db(&self, field_name: String, new_value: String, guild_id: String) -> UnwrapResult<()> {
                DB.use_ns("discord-namespace").use_db("discord").await?;
                let sql_query = &*format!("UPDATE guild_config SET {field_name} = $value WHERE guild_id = $guild_id");
                let _updated: Vec<Self> = DB
                    .query(sql_query)
                    .bind(("value", new_value))
                    .bind(("guild_id", guild_id))
                    .await?
                    .take(0)?;

                Ok(())
            }
        }
    };
}

macro_rules! build_obj {
    ($name:ident, $($field:ident: $type:ty),*) => {
        #[derive(Serialize, Deserialize, Debug, Clone, Default)]
        pub struct $name {
            $(pub $field: $type,)*
            pub guild_id: Option<String>,
        }

        impl $name {
            $(
                pub fn $field(mut self, $field: $type) -> Self {
                    self.$field = $field;
                    self
                }
            )*

            pub fn guild_id(mut self, guild_id: GuildId) -> Self {
                self.guild_id = Some(guild_id.to_string());
                self
            }

            pub async fn save_to_db(&self) -> SurrealResult<()> {
                DB.use_ns("discord-namespace").use_db("discord").await?;
                let _created: Vec<Self> = DB
                    .create("guild_config")
                    .content(self)
                    .await.take(0)?;

                Ok(())
            }

            pub async fn verify_data(guild_id: GuildId) -> SurrealResult<Option<Self>> {
                DB.use_ns("discord-namespace").use_db("discord").await?;
                let sql_query = "SELECT * FROM guild_config WHERE guild_id = $guild_id";
                let existing_data: Option<Self> = DB
                    .query(sql_query)
                    .bind(("guild_id", guild_id))
                    .await?;

                Ok(existing_data)
            }
        }
    };
}

obj!(Admin, role: String, role_2: String);
obj!(Forbidden, user: String, role: String);
obj!(TimeOut, time: String);
obj!(Channels, welcome: String, ooc: String, logs: String, exceptions: String);
obj!(Messages, welcome: String, time_out: String, warn: String);
build_obj!(GuildData,
    admins: Admin,
    forbidden: Forbidden,
    time_out: TimeOut,
    channels: Channels,
    messages: Messages
);
*/
