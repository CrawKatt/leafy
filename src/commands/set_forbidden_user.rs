use serenity::all::User;
use serde::{Deserialize, Serialize};
use surrealdb::Result as SurrealResult;

use crate::DB;
use crate::utils::autocomplete::autocomplete_set_forbidden_user;
use crate::utils::{CommandResult, Context};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ForbiddenUserData {
    pub user: User,
    pub user_id: u64,
}

impl ForbiddenUserData {
    pub const fn new(user: User, user_id: u64) -> Self {
        Self { user, user_id }
    }
    pub async fn save_to_db(&self) -> SurrealResult<()> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let _created: Vec<ForbiddenUserData> = DB
            .create("forbidden_users")
            .content(self)
            .await?;

        println!("Created forbidden user: {:?}", self.user_id);

        Ok(())
    }
    pub async fn update_in_db(&self) -> SurrealResult<()> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let sql_query = "UPDATE forbidden_users SET user_id = $user_id";
        let _updated: Vec<ForbiddenUserData> = DB
            .query(sql_query)
            .bind(("user_id", &self.user_id))
            .await?
            .take(0)?;

        println!("Updated forbidden user: {:?}", self.user_id);

        Ok(())
    }
    pub async fn verify_data(&self) -> SurrealResult<Option<ForbiddenUserData>> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let sql_query = "SELECT * FROM forbidden_users WHERE user_id = $user_id";
        let existing_data: Option<ForbiddenUserData> = DB
            .query(sql_query)
            .bind(("user_id", &self.user_id))
            .await?
            .take(0)?;

        println!("Verified forbidden user: {:?}", self.user_id);

        Ok(existing_data)
    }
}

/// Establece el usuario prohibido de mencionar
#[poise::command(prefix_command, slash_command)]
pub async fn set_forbidden_user(
    ctx: Context<'_>,
    #[autocomplete = "autocomplete_set_forbidden_user"]
    #[description = "The user to set as the forbidden user"] user: User,
) -> CommandResult {
    DB.use_ns("discord-namespace").use_db("discord").await?;
    let data = ForbiddenUserData::new(user.clone(), u64::from(user.id));

    let existing_data = data.verify_data().await?;

    let Some(_) = existing_data else {
        data.save_to_db().await?;
        ctx.say(format!("Se ha prohibido mencionar a: **{}**", user.name)).await?;
        return Ok(())
    };

    data.update_in_db().await?;

    ctx.say(format!("Se ha prohibido mencionar a: **{}**", user.name)).await?;

    Ok(())
}