use crate::commands::audio::{is_music_state, AudioState};
use crate::handlers::error::handler;
use crate::utils::debug::IntoUnwrapResult;
use crate::utils::{CommandResult, Context};
use elevenlabs_rs::utils::save;
use elevenlabs_rs::{ElevenLabsClient, Model, TextToSpeech, TextToSpeechBody};
use songbird::{input, Event, TrackEvent};
use crate::commands::audio::tts::TtsStateUpdater;
use crate::commands::translate::create_ai_message;

/// Traduce el texto de entrada al idioma deseado en el Voice Chat mediante TTS
#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    on_error = "handler",
    user_cooldown = 10,
    category = "Audio",
)]
pub async fn translate_tts(
    ctx: Context<'_>,
    lang: String,
    #[rest]
    text: String,
) -> CommandResult {
    {
        if is_music_state(ctx).await {
            ctx.say("❌ No puedes utilizar TTS mientras hay música en reproducción").await?;
            return Ok(())
        }
    }

    let message = create_ai_message(text, lang).await?;
    let guild = ctx.guild().into_result()?.clone();
    let guild_id = guild.id;
    let author = ctx.author_member().await.into_result()?;
    let author_name = author.distinct();
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
    let body = TextToSpeechBody::new(&format!("Usuario {author_name}: {message}"), Model::ElevenMultilingualV2);
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