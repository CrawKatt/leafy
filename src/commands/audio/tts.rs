use crate::commands::audio::{set_audio_state, AudioState};
use crate::handlers::error::handler;
use crate::utils::debug::IntoUnwrapResult;
use crate::utils::{CommandResult, Context};
use elevenlabs_rs::utils::save;
use elevenlabs_rs::{ElevenLabsClient, Model, TextToSpeech, TextToSpeechBody};
use poise::async_trait;
use songbird::{input, Event, EventContext, EventHandler, TrackEvent};
use std::sync::Arc;
use tokio::sync::Mutex;

struct TtsStateUpdater {
    audio_state: Arc<Mutex<AudioState>>,
}

#[async_trait]
impl EventHandler for TtsStateUpdater {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        if let EventContext::Track(track_list) = ctx {
            if track_list.len() == 1 {
                set_audio_state(self.audio_state.clone(), AudioState::Idle).await;
            }
        }
        None
    }
}

#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    on_error = "handler",
    user_cooldown = 10,
    category = "Audio",
)]
pub async fn tts(
    ctx: Context<'_>,
    #[rest]
    text: String
) -> CommandResult {
    {
        let audio_state = ctx.data().voice_chat_state.lock().await;
        if *audio_state == AudioState::Music {
            ctx.say("❌ No puedes utilizar TTS mientras hay música en reproducción").await?;
            return Ok(())
        }
    }

    let guild = ctx.guild().into_result()?.clone();
    let guild_id = guild.id;
    super::try_join(ctx, guild).await?;

    let manager = songbird::get(ctx.serenity_context())
        .await
        .into_result()?;

    let Some(handler_lock) = manager.get(guild_id) else {
        ctx.say("No estás en un canal de voz").await?;
        return Ok(())
    };

    let mut handler = handler_lock.lock().await;

    let client = ElevenLabsClient::default()?;
    let body = TextToSpeechBody::new(&format!("Usuario {}: {}", ctx.author().name ,text), Model::ElevenMultilingualV2);
    let endpoint = TextToSpeech::new("bIQlQ61Q7WgbyZAL7IWj", body);
    let speech = client.hit(endpoint).await?;
    save("result.mp3", speech)?;

    let source = input::File::new("result.mp3");
    let track = handler.play_input(source.into());

    {
        let mut audio_state = ctx.data().voice_chat_state.lock().await;
        audio_state.update_state(AudioState::Tts);
    }

    track.add_event(
        Event::Track(TrackEvent::End),
        TtsStateUpdater {
            audio_state: ctx.data().voice_chat_state.clone(),
        },
    )?;

    drop(handler);

    Ok(())
}