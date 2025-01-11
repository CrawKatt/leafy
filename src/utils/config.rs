use bon::Builder;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serenity::all::GuildId;
use surrealdb::opt::PatchOp;
use surrealdb::{RecordId, Result as SurrealResult};

use crate::utils::debug::UnwrapResult;
use crate::DB;

#[derive(Serialize, Deserialize, Debug, Clone, Default, Builder)]
pub struct AutoRole {
    pub guild: String,
    pub assignments: Vec<Assignment>,
    pub messages: Vec<AutoRoleMessage>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, Builder)]
pub struct Assignment {
    pub emoji_id: String,
    pub emoji_name: String,
    pub role: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, Builder)]
pub struct AutoRoleMessage {
    pub channel_id: String,
    pub message_id: String,
}

impl AutoRole {
    pub async fn add_message(
        guild_id: GuildId,
        channel_id: String,
        message_id: String,
    ) -> SurrealResult<()> {
        DB.use_ns("discord-namespace").use_db("discord").await?;

        // Buscar el registro del guild
        let existing: Option<Self> = DB
            .select(("autoroles", guild_id.to_string()))
            .await?;

        let Some(mut data) = existing else {
            let new_data = Self::builder()
                .guild(guild_id.to_string())
                .assignments(vec![])
                .messages(vec![AutoRoleMessage::builder()
                    .channel_id(channel_id)
                    .message_id(message_id)
                    .build()
                ])
                .build();

            let _: Option<Self> = DB
                .create(("autoroles", guild_id.to_string()))
                .content(new_data)
                .await?;

            return Ok(())
        };

        // Agregar el mensaje a la lista
        data.messages.push(AutoRoleMessage { channel_id, message_id });

        // Actualizar en la base de datos
        let _updated: Option<Self> = DB
            .update(("autoroles", guild_id.to_string()))
            .patch(PatchOp::replace("messages", data.messages))
            .await?;

        Ok(())
    }

    pub async fn add_assignment(
        guild_id: GuildId,
        emoji_id: String,
        emoji_name: String,
        role: String,
    ) -> SurrealResult<()> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        
        let existing: Option<Self> = DB
            .select(("autoroles", guild_id.to_string()))
            .await?;

        let Some(mut data) = existing else {
            let new_data = Self::builder()
                .guild(guild_id.to_string())
                .assignments(vec![
                    Assignment::builder()
                        .emoji_id(emoji_id)
                        .emoji_name(emoji_name)
                        .role(role)
                        .build()
                ])
                .messages(vec![])
                .build();

            let _: Option<Self> = DB
                .create(("autoroles", guild_id.to_string()))
                .content(new_data)
                .await?;

            return Ok(())
        };

        // Verifica si la asignación ya existe
        if let Some(assignment) = data
            .assignments
            .iter_mut()
            .find(|a| a.emoji_id == emoji_id)
        {
            // Actualiza el rol y el nombre del emoji si ya existe
            assignment.role = role;
            assignment.emoji_name = emoji_name;
        } else {
            // Agrega una nueva asignación
            data.assignments.push(Assignment { emoji_id, emoji_name, role });
        }

        // Actualiza el registro en la base de datos
        let _updated: Option<Self> = DB
            .update(("autoroles", guild_id.to_string()))
            .patch(PatchOp::replace("assignments", data.assignments))
            .await?;

        Ok(())
    }
    
    pub async fn get_assignments(guild_id: GuildId) -> SurrealResult<Option<Self>> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let existing = DB
            .select(("autoroles", guild_id.to_string()))
            .await?;

        Ok(existing)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Builder)]
pub struct GuildData {
    #[builder(default)]
    pub admins: Admin,
    #[builder(default)]
    pub forbidden: Forbidden,
    pub id: Option<RecordId>,
    #[builder(default)]
    pub time_out: TimeOut,
    #[builder(default)]
    pub channels: Channels,
    #[builder(default)]
    pub messages: Messages,
}
impl GuildData {
    pub async fn save_to_db(self, guild_id: GuildId) -> SurrealResult<()> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let _created: Option<Self> = DB
            .create(("guild_config", guild_id. to_string()))
            .content(self)
            .await?;

        Ok(())
    }
    pub async fn verify_data(guild_id: GuildId) -> SurrealResult<Option<Self>> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let existing_data = DB
            .select(("guild_config", guild_id. to_string()))
            .await?;

        Ok(existing_data)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, Builder)]
#[builder(on(String, into))]
pub struct Admin {
    pub role: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, Builder)]
#[builder(on(String, into))]
pub struct Forbidden {
    pub user: Option<String>,
    pub role: Option<String>
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, Builder)]
#[builder(on(String, into))]
pub struct TimeOut {
    pub time: Option<String>
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, Builder)]
#[builder(on(String, into))]
pub struct Channels {
    pub welcome: Option<String>,
    pub ooc: Option<String>,
    pub logs: Option<String>,
    pub exceptions: Option<String>
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, Builder)]
#[builder(on(String, into))]
pub struct Messages {
    pub welcome: Option<String>,
    pub time_out: Option<String>,
    pub warn: Option<String>
}

pub trait DatabaseOperations: Serialize + DeserializeOwned + Clone + Default + Send + Sync {
    async fn update_field_in_db(&self, field_name: &str, new_value: &str, guild_id: &str) -> UnwrapResult<()> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let sql_query = &*format!("UPDATE guild_config SET {field_name} = $value WHERE guild_id = $guild_id");
        let _updated: Option<Self> = DB
            .query(sql_query)
            .bind(("value", new_value.to_string()))
            .bind(("guild_id", guild_id.to_string()))
            .await?
            .take(0)?;

        Ok(())
    }

    async fn update_admins(&self, field_name: &str, new_value: Vec<String>, guild_id: &str) -> UnwrapResult<()> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let sql_query = &*format!("UPDATE guild_config SET {field_name} = $value WHERE guild_id = $guild_id");
        let _updated: Option<Self> = DB
            .query(sql_query)
            .bind(("value", new_value))
            .bind(("guild_id", guild_id.to_string()))
            .await?
            .take(0)?;

        Ok(())
    }
}

impl DatabaseOperations for Admin {}
impl DatabaseOperations for Forbidden {}
impl DatabaseOperations for TimeOut {}
impl DatabaseOperations for Channels {}
impl DatabaseOperations for Messages {}

pub trait Getter {
    fn to_id(&self) -> String;
}

impl Getter for RecordId {
    fn to_id(&self) -> String {
        self
            .key()
            .to_string()
            .trim_matches(|c| c == '⟨' || c == '⟩')
            .to_string()
    }
}