pub mod set_forbidden_user;
pub mod set_forbidden_role;
pub mod set_timeout_timer;
pub mod set_log_channel;
pub mod set_admins;
pub mod set_warn_message;
pub mod set_timeout_message;
pub mod set_forbidden_exception;
pub mod set_welcome_channel;
pub mod set_welcome_message;
pub mod set_ooc_channel;

use crate::DB;
use serenity::all::{ChannelId, GuildId, RoleId, UserId};
use serde::{Deserialize, Serialize};
use surrealdb::Result as SurrealResult;
use crate::utils::misc::debug::UnwrapErrors;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[allow(clippy::struct_field_names)]
pub struct AdminData {
    pub role_id: Option<String>,
    pub role_2_id: Option<String>,
    pub guild_id: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ForbiddenUserData {
    pub user_id: String,
    pub guild_id: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ForbiddenRoleData {
    pub role_id: String,
    pub guild_id: GuildId,
}

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct SetTimeoutTimer {
    pub time: i64,
    pub guild_id: GuildId,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct WarnMessageData {
    pub guild_id: GuildId,
    pub warn_message: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct TimeOutMessageData {
    pub guild_id: GuildId,
    pub time_out_message: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct GuildData {
    pub guild_id: GuildId,
    pub log_channel_id: ChannelId,
}

impl AdminData {
    pub fn new(role_id: Option<RoleId>, role_2_id: Option<RoleId>, guild_id: GuildId) -> Self {
        Self {
            role_id: role_id.map(|id| id.to_string()),
            role_2_id: role_2_id.map(|id| id.to_string()),
            guild_id: guild_id.to_string()
        }
    }

    pub async fn save_to_db(&self) -> SurrealResult<()> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let sql_query = "SELECT * FROM admins WHERE guild_id = $guild_id";
        let existing_data: Option<Self> = DB
            .query(sql_query)
            .bind(("guild_id", &self.guild_id))
            .await?
            .take(0)?;

        if existing_data.is_none() {
            let _created: Vec<Self> = DB
                .create("admins")
                .content(self)
                .await?;

            println!("Created admin role: {:?}", self.role_id);
        } else {
            let sql_query = "UPDATE admins SET role_id = $role_id, role_2_id = $role_2_id WHERE guild_id = $guild_id";
            let _updated: Option<Self> = DB
                .query(sql_query)
                .bind(("role_id", &self.role_id))
                .bind(("role_2_id", &self.role_2_id))
                .bind(("guild_id", &self.guild_id))
                .await?
                .take(0)?;

            println!("Updated admin role: {:?}", self.role_id);
        }

        Ok(())
    }
    pub async fn get_admin_role(guild_id: GuildId) -> SurrealResult<Option<String>> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let sql_query = "SELECT * FROM admins WHERE guild_id = $guild_id";
        let query_result = DB
            .query(sql_query)
            .bind(("guild_id", &guild_id.to_string()))
            .await;

        let existing_data: Option<Self> = match query_result {
            Ok(mut result) => result.take(0)?,
            Err(why) => return Err(why)
        };

        match existing_data {
            Some(data) => {
                let role_id = data.role_id.unwrap_or_default();
                let role_u64 = role_id.parse::<u64>().unwrap_or_default();
                let role_id = RoleId::new(role_u64);
                Ok(Some(role_id.to_string()))
            },
            None => Ok(None)
        }
    }

    pub async fn get_admin_role_2(guild_id: GuildId) -> SurrealResult<Option<String>> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let sql_query = "SELECT * FROM admins WHERE guild_id = $guild_id";
        let query_result = DB
            .query(sql_query)
            .bind(("guild_id", &guild_id.to_string()))
            .await;

        let existing_data: Option<Self> = match query_result {
            Ok(mut result) => result.take(0)?,
            Err(why) => return Err(why)
        };

        match existing_data {
            Some(data) => {
                data.role_2_id.map_or_else(|| Ok(None), |role_id| {
                    let role_u64 = role_id.parse::<u64>().unwrap_or_default();
                    let role_id = RoleId::new(role_u64);
                    Ok(Some(role_id.to_string()))
                })
            },
            None => Ok(None)
        }
    }
}

impl ForbiddenUserData {
    pub fn new(user_id: UserId, guild_id: GuildId) -> Self {
        Self {
            user_id: user_id.to_string(),
            guild_id: guild_id.to_string()
        }
    }
    pub async fn save_to_db(&self) -> SurrealResult<()> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let _created: Vec<Self> = DB
            .create("forbidden_users")
            .content(self)
            .await?;

        println!("Created forbidden user: {:?}", self.user_id);

        Ok(())
    }
    pub async fn update_in_db(&self) -> SurrealResult<()> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let sql_query = "UPDATE forbidden_users SET user_id = $user_id";
        let _updated: Vec<Self> = DB
            .query(sql_query)
            .bind(("user_id", &self.user_id))
            .await?
            .take(0)?;

        println!("Updated forbidden user: {:?}", self.user_id);

        Ok(())
    }
    pub async fn verify_data(&self) -> SurrealResult<Option<Self>> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let sql_query = "SELECT * FROM forbidden_users WHERE guild_id = $guild_id";
        let existing_data: Option<Self> = DB
            .query(sql_query)
            .bind(("guild_id", &self.guild_id))
            .await?
            .take(0)?;

        println!("Verified forbidden user: {:?}", self.user_id);

        Ok(existing_data)
    }

    pub async fn get_forbidden_user_id(guild_id: GuildId) -> SurrealResult<Option<u64>> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let sql_query = "SELECT * FROM forbidden_users WHERE guild_id = $guild_id";
        let existing_data: Option<Self> = DB
            .query(sql_query)
            .bind(("guild_id", &guild_id))
            .await?
            .take(0)?;

        Ok(existing_data.map(|data| data.user_id.parse::<u64>().unwrap_or_else(|why| {
            println!("Error: {why:?}");
            0
        })))
    }
}

impl ForbiddenRoleData {
    pub fn new(role_id: RoleId, guild_id: GuildId) -> Self {
        Self {
            role_id: role_id.to_string(),
            guild_id
        }
    }
    pub async fn save_to_db(&self) -> SurrealResult<()> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let _created: Vec<Self> = DB
            .create("forbidden_roles")
            .content(self)
            .await?;

        println!("Created forbidden role: {:?}", self.role_id);

        Ok(())
    }
    pub async fn update_in_db(&self) -> SurrealResult<()> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let sql_query = "UPDATE forbidden_roles SET role_id = $role_id";
        let _updated: Option<Self> = DB
            .query(sql_query)
            .bind(("role_id", &self.role_id))
            .await?
            .take(0)?;

        println!("Updated forbidden role: {:?}", self.role_id);

        Ok(())
    }
    pub async fn verify_data(&self) -> SurrealResult<Option<Self>> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let sql_query = "SELECT * FROM forbidden_roles WHERE role_id = $role_id";
        let existing_data: Option<Self> = DB
            .query(sql_query)
            .bind(("role_id", &self.role_id))
            .await?
            .take(0)?;

        println!("Verified forbidden role: {:?}", self.role_id);

        Ok(existing_data)
    }
    pub async fn get_role_id(guild_id: GuildId) -> SurrealResult<Option<u64>> {
        DB.use_ns("discord-namespace").use_db("discord").await?;

        // Obtener el rol prphíbido de mencionar desde la base de datos
        // role.id porque `guild_id` es objeto de `role`
        let sql_query = "SELECT * FROM forbidden_roles WHERE guild_id = $guild_id";
        let existing_data: Option<Self> = DB
            .query(sql_query)
            .bind(("guild_id", guild_id))
            .await?
            .take(0)?;

        Ok(existing_data.map(|data| data.role_id.parse::<u64>().unwrap_or_default()))
    }
}

impl SetTimeoutTimer {
    pub const fn new(time: i64, guild_id: GuildId) -> Self {
        Self { time, guild_id }
    }

    pub async fn save_to_db(&self) -> SurrealResult<()> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let _created: Vec<Self> = DB
            .create("time_out_timer")
            .content(self)
            .await?;

        Ok(())
    }

    pub async fn update_in_db(&self) -> SurrealResult<()> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let sql_query = "UPDATE time_out_timer SET time = $time WHERE guild_id = $guild_id";
        let _updated: Vec<Self> = DB
            .query(sql_query)
            .bind(("time", self.time))
            .bind(("guild_id", self.guild_id))
            .await?
            .take(0)?;

        Ok(())
    }

    pub async fn verify_data(&self) -> SurrealResult<Option<Self>> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let sql_query = "SELECT * FROM time_out_timer WHERE guild_id = $guild_id";
        let existing_data: Option<Self> = DB
            .query(sql_query)
            .bind(("guild_id", self.guild_id))
            .await?
            .take(0)?;

        Ok(existing_data)
    }

    pub async fn get_time_out_timer(guild_id: GuildId) -> SurrealResult<Option<i64>> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let sql_query = "SELECT * FROM time_out_timer WHERE guild_id = $guild_id";
        let existing_data: Option<Self> = DB
            .query(sql_query)
            .bind(("guild_id", &guild_id))
            .await?
            .take(0)?;

        Ok(existing_data.map(|data| data.time))
    }
}

impl WarnMessageData {
    pub const fn new(guild_id: GuildId, warn_message: String) -> Self {
        Self { guild_id, warn_message }
    }

    pub async fn save_to_db(&self) -> SurrealResult<()> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let _created: Vec<Self> = DB
            .create("warn_message")
            .content(self)
            .await?;

        println!("Created warn message: {:?}", self.warn_message);

        Ok(())
    }

    pub async fn update_in_db(&self) -> SurrealResult<()> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let sql_query = "UPDATE warn_message SET warn_message = $warn_message";
        let _updated: Vec<Self> = DB
            .query(sql_query)
            .bind(("warn_message", &self.warn_message))
            .await?
            .take(0)?;

        println!("Updated warn message: {:?}", self.warn_message);

        Ok(())
    }

    pub async fn verify_data(&self) -> Result<Option<Self>, UnwrapErrors> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let sql_query = "SELECT * FROM warn_message WHERE guild_id = $guild_id";
        let existing_data: Option<Self> = DB
            .query(sql_query)
            .bind(("guild_id", &self.guild_id))
            .await?
            .take(0)?;

        println!("Verified warn message: {:?}", self.warn_message);

        Ok(existing_data)
    }

    pub async fn get_warn_message(guild_id: GuildId) -> SurrealResult<Option<String>> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let sql_query = "SELECT * FROM warn_message WHERE guild_id = $guild_id";
        let existing_data: Option<Self> = DB
            .query(sql_query)
            .bind(("guild_id", &guild_id))
            .await?
            .take(0)?;

        Ok(existing_data.map(|data| data.warn_message))
    }
}

impl TimeOutMessageData {
    pub const fn new(guild_id: GuildId, time_out_message: String) -> Self {
        Self { guild_id, time_out_message }
    }

    pub async fn save_to_db(&self) -> SurrealResult<()> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let _created: Vec<Self> = DB
            .create("time_out_message")
            .content(self)
            .await?;

        println!("Created time out message: {:?}", self.time_out_message);

        Ok(())
    }

    pub async fn update_in_db(&self) -> SurrealResult<()> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let sql_query = "UPDATE time_out_message SET time_out_message = $time_out_message";
        let _updated: Vec<Self> = DB
            .query(sql_query)
            .bind(("time_out_message", &self.time_out_message))
            .await?
            .take(0)?;

        println!("Updated time out message: {:?}", self.time_out_message);

        Ok(())
    }

    pub async fn verify_data(&self) -> SurrealResult<Option<Self>> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let sql_query = "SELECT * FROM time_out_message WHERE guild_id = $guild_id";
        let existing_data: Option<Self> = DB
            .query(sql_query)
            .bind(("guild_id", &self.guild_id))
            .await?
            .take(0)?;

        println!("Verified time out message: {:?}", self.time_out_message);

        Ok(existing_data)
    }

    pub async fn get_time_out_message(guild_id: GuildId) -> SurrealResult<Option<String>> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let sql_query = "SELECT * FROM time_out_message WHERE guild_id = $guild_id";
        let existing_data: Option<Self> = DB
            .query(sql_query)
            .bind(("guild_id", &guild_id))
            .await?
            .take(0)?;

        Ok(existing_data.map(|data| data.time_out_message))
    }
}

impl GuildData {
    pub const fn new(guild_id: GuildId, log_channel_id: ChannelId) -> Self {
        Self { guild_id, log_channel_id }
    }
    async fn save_to_db(&self) -> SurrealResult<()> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let _created: Vec<Self> = DB
            .create("guilds")
            .content(self)
            .await?;

        Ok(())
    }
    async fn update_in_db(&self) -> SurrealResult<()> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let sql_query = "UPDATE guilds SET log_channel_id = $log_channel_id WHERE guild_id = $guild_id";
        let _updated: Vec<Self> = DB
            .query(sql_query)
            .bind(("log_channel_id", self.log_channel_id))
            .bind(("guild_id", self.guild_id))
            .await?
            .take(0)?;

        Ok(())
    }
    async fn verify_data(&self) -> SurrealResult<Option<Self>> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let sql_query = "SELECT * FROM guilds WHERE guild_id = $guild_id";
        let existing_data: Option<Self> = DB
            .query(sql_query)
            .bind(("guild_id", self.guild_id))
            .await?
            .take(0)?;

        Ok(existing_data)
    }

    pub async fn get_log_channel(guild_id: GuildId) -> SurrealResult<Option<Self>> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let sql_query = "SELECT * FROM guilds WHERE guild_id = $guild_id";
        let existing_data: Option<Self> = DB
            .query(sql_query)
            .bind(("guild_id", guild_id))
            .await?
            .take(0)?;

        Ok(existing_data)
    }
}
