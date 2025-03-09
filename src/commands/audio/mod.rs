use std::sync::Arc;
use serenity::all::{ChannelId, GetMessages, Guild, GuildId};
use tokio::sync::Mutex;
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
pub mod tts;
pub mod translate_tts;

#[derive(PartialEq, Eq, Debug)]
pub enum AudioState {
    Music,
    Tts,
    Idle
}

impl AudioState {
    pub const fn update_state(&mut self, new_state: Self) {
        *self = new_state;
    }
}

pub async fn is_music_state(ctx: Context<'_>) -> bool {
    let audio_state = ctx.data().voice_chat_state.lock().await;
    *audio_state == AudioState::Music
} 

pub async fn set_audio_state(state: Arc<Mutex<AudioState>>, new_state: AudioState) {
    let mut current_state = state.lock().await;
    *current_state = new_state;
}

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

pub async fn get_guild_id_and_channel_id(ctx: Context<'_>) -> UnwrapResult<(GuildId, Option<ChannelId>)> {
    let messages = ctx.channel_id().messages(&ctx.http(), GetMessages::default()).await?;
    let msg = messages.first().unwrap_log(location!())?;

    let guild = ctx.guild().unwrap();
    let channel_id = guild
        .voice_states
        .get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id);

    Ok((guild.id, channel_id))
}