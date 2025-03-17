use std::sync::Arc;
use poise::async_trait;
use crate::handlers::error::handler;
use crate::utils::debug::{IntoUnwrapResult, UnwrapLog};
use crate::utils::{CommandResult, Context};
use songbird::{input, Event, EventContext, EventHandler, TrackEvent};
use serenity::all::{CreateEmbed, CreateEmbedAuthor, CreateMessage};
use songbird::input::YoutubeDl;
use tokio::sync::Mutex;
use crate::{location, HttpKey};
use crate::commands::audio;
use crate::commands::audio::{set_audio_state, AudioState};
use crate::commands::audio::music::queue::AuxMetadataKey;

struct MusicStateUpdater {
    audio_state: Arc<Mutex<AudioState>>,
    handler: Arc<Mutex<songbird::Call>>,
}

#[async_trait]
impl EventHandler for MusicStateUpdater {
    async fn act(&self, _: &EventContext<'_>) -> Option<Event> {
        let handler = self.handler.lock().await;
        if handler.queue().is_empty() {
            set_audio_state(self.audio_state.clone(), AudioState::Idle).await;
        }
        drop(handler);
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
    aliases("p"),
)]
pub async fn play(
    ctx: Context<'_>,
    #[rest]
    query: String
) -> CommandResult {
    {
        let audio_state = ctx.data().voice_chat_state.lock().await;
        if *audio_state == AudioState::Tts {
            ctx.say("❌ No puedes reproducir música mientras el modo TTS está activo").await?;
            return Ok(())
        }
    }

    let do_search = !query.starts_with("http");

    let guild = ctx.guild().into_result()?.clone();
    let guild_id = guild.id;
    audio::try_join(ctx, guild).await?;

    let http_client = {
        let data = ctx.serenity_context().data.read().await;
        data.get::<HttpKey>()
            .cloned()
            .unwrap_log(location!())?
    };

    let author_name = ctx.author_member().await.into_result()?.distinct();
    let author_face = ctx.author_member().await.into_result()?.face();

    let manager = songbird::get(ctx.serenity_context())
        .await
        .into_result()?;

    let Some(handler_lock) = manager.get(guild_id) else {
        ctx.say("No estás en un canal de voz").await?;
        return Ok(())
    };

    {
        let mut audio_state = ctx.data().voice_chat_state.lock().await;
        audio_state.update_state(AudioState::Music);
    }

    let message = ctx.say("Buscando...").await?;

    // Utilizar tor + tornet para evadir el baneo de YouTube
    let mut handler = handler_lock.lock().await;
    let source = if do_search {
        YoutubeDl::new_search(http_client, query).user_args(vec!["socks5://127.0.0.1:9050".to_string()])
    } else if query.starts_with("https://youtube.com/") {
        YoutubeDl::new(http_client, query).user_args(vec!["socks5://127.0.0.1:9050".to_string()])
    } else {
        YoutubeDl::new(http_client, query)
    };

    let mut src: input::Input = source.into();

    let aux_metadata = src.aux_metadata().await?;
    let title = aux_metadata.title.clone().into_result()?;
    let thumbnail = aux_metadata.thumbnail.clone().into_result()?;

    let track = handler.enqueue_input(src).await;

    let mut map = track.typemap().write().await;
    map.entry::<AuxMetadataKey>().or_insert(aux_metadata);

    let song_name = if handler.queue().is_empty() { format!("Reproduciendo {title}") } else { format!("{title} Añadido a la cola") };

    message.delete(ctx).await?;
    let desc = format!("**Solicitado por:** {author_name}");
    let embed = CreateEmbed::new()
        .title(song_name)
        .author(CreateEmbedAuthor::new(author_name)
            .icon_url(author_face))
        .description(desc)
        .thumbnail(thumbnail)
        .color(0x00ff_0000);

    let builder = CreateMessage::new().embed(embed);
    ctx.channel_id().send_message(ctx.http(), builder).await?;
    
    track.add_event(
        Event::Track(TrackEvent::End),
        MusicStateUpdater {
            audio_state: ctx.data().voice_chat_state.clone(),
            handler: handler_lock.clone(),
        }
    )?;

    drop(handler);
    drop(map);

    Ok(())
}