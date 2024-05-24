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
        #[allow(clippy::module_name_repetitions)]
        #[allow(clippy::struct_field_names)]
        #[derive(Serialize, Deserialize, Debug, Clone, Default)]
        pub struct $name {
            $(pub $field: Option<$type>,)*
        }

        #[allow(clippy::missing_const_for_fn)]
        impl $name {
            $(
                pub fn $field(mut self, $field: impl Into<$type>) -> Self {
                    self.$field = Some($field.into());
                    self
                }
            )*
            
            pub async fn update_field_in_db(&self, field_name: &str, new_value: &str, guild_id: &str) -> UnwrapResult<()> {
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
        #[allow(clippy::module_name_repetitions)]
        #[allow(clippy::struct_field_names)]
        #[derive(Serialize, Deserialize, Debug, Clone, Default)]
        pub struct $name {
            $(pub $field: $type,)*
            pub guild_id: Option<String>,
        }

        #[allow(clippy::missing_const_for_fn)]
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
                DB.query("DEFINE INDEX guild_id ON TABLE guild_config COLUMNS guild_id UNIQUE").await?;
                let _created: Vec<Self> = DB
                    .create("guild_config")
                    .content(self)
                    .await?;

                Ok(())
            }
            
            pub async fn verify_data(guild_id: GuildId) -> SurrealResult<Option<Self>> {
                DB.use_ns("discord-namespace").use_db("discord").await?;
                let sql_query = "SELECT * FROM guild_config WHERE guild_id = $guild_id";
                let existing_data: Option<Self> = DB
                    .query(sql_query)
                    .bind(("guild_id", guild_id))
                    .await?
                    .take(0)?;

                Ok(existing_data)
            }
        }
    };
}

obj!(Admin, role_id: String, role_2_id: String);
obj!(Forbidden, user_id: String, role_id: String);
obj!(TimeOut, time: String);
obj!(Channels, welcome_channel_id: String, ooc_channel_id: String, log_channel_id: String);
obj!(Messages, welcome: String, time_out: String, warn: String);
build_obj!(GuildData,
    admins: Admin,
    forbidden_config: Forbidden,
    time_out_config: TimeOut,
    channel_config: Channels,
    messages_config: Messages
);