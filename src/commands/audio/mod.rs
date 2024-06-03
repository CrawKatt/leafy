use rusty_ytdl::search::{SearchResult, YouTube};
use serenity::all::Guild;
use crate::location;
use crate::utils::{CommandResult, Context};
use crate::utils::debug::{IntoUnwrapResult, UnwrapLog, UnwrapResult};

pub mod join;
pub mod leave;
pub mod play;
pub mod pause;
pub mod resume;
pub mod queue;
pub mod skip;
pub mod stop;

pub async fn try_join(ctx: Context<'_>, guild: Guild) -> CommandResult {
    let channel_id = guild
        .voice_states
        .get(&ctx.author().id)
        .and_then(|voice_state| voice_state.channel_id)
        .unwrap_log(location!())?;

    let manager = songbird::get(ctx.serenity_context())
        .await
        .into_result()?;

    let already_joined = manager.get(guild.id).is_some();
    if !already_joined {
        let _ = manager.join(guild.id, channel_id).await?;
    }

    Ok(())
}

pub async fn try_get_song(ctx: Context<'_>, do_search: bool, query: String) -> UnwrapResult<String> {
    let result = if do_search {
        let youtube = YouTube::new()?;
        let res = youtube.search(query, None).await?.first().into_result()?.clone();
        let SearchResult::Video(res) = res else {
            ctx.say("No se ha encontrado nada").await?;
            return Ok(String::new())
        };
        res.url
    } else {
        query
    };

    Ok(result)
}