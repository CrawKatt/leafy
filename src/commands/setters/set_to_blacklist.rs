use serde::{Deserialize, Serialize};
use serenity::all::GuildId;
use crate::utils::{CommandResult, Context};
use surrealdb::Result as SurrealResult;
use crate::DB;
use crate::utils::misc::debug::UnwrapResult;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct BlackListData {
    pub link: String,
    pub guild_id: GuildId,
}

impl BlackListData {
    pub fn new(
        link: &String,
        guild_id: GuildId,
    ) -> Self {
        Self {
            link: link.to_string(),
            guild_id,
        }
    }

    /// Guarda el link en la base de datos
    pub async fn save_to_db(&self) -> SurrealResult<()> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let _created: Vec<Self> = DB
            .create("blacklist")
            .content(self)
            .await?;

        Ok(())
    }

    /// Verifica si el link ya se encuentra en la base de datos
    pub async fn verify_data(&self) -> SurrealResult<Option<Self>> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let sql_query = "SELECT * FROM blacklist WHERE link = $link AND guild_id = $guild_id";
        let existing_data: Option<Self> = DB
            .query(sql_query)
            .bind(("link", &self.link))
            .bind(("guild_id", &self.guild_id))
            .await?
            .take(0)?;

        Ok(existing_data)
    }

    pub async fn get_blacklist_link(guild_id: GuildId, link: &String) -> UnwrapResult<Option<String>> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let sql_query = "SELECT * FROM blacklist WHERE guild_id = $guild_id AND link = $link";
        let existing_data: Option<Self> = DB
            .query(sql_query)
            .bind(("guild_id", guild_id))
            .bind(("link", link))
            .await?
            .take(0)?;

        Ok(existing_data.map(|data| data.link))
    }
}

/// Agrega un link a la lista negra.
#[poise::command(
    prefix_command,
    slash_command,
    category = "Moderator",
    required_permissions = "MANAGE_ROLES",
    guild_only,
    ephemeral
)]
pub async fn add_to_blacklist(
    ctx: Context<'_>,
    #[description = "Agrega un link a la lista negra"] link: String,
) -> CommandResult {
    let guild_id = ctx.guild_id().unwrap();
    let black_list = BlackListData::new(&link, guild_id);
    let existing_data = black_list.verify_data().await?;

    if !link.contains("https://") && !link.contains("www.") {
        poise::say_reply(ctx, "El link proporcionado no es válido").await?;
        return Ok(())
    }

    if existing_data.is_none() {
        black_list.save_to_db().await?;
        poise::say_reply(ctx, format!("El link **{link}** ha sido agregado a la lista negra")).await?;
        return Ok(())
    }

    poise::say_reply(ctx, "El link ya se encuentra en la lista negra").await?;

    Ok(())
}