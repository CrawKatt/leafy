use std::cmp::Reverse;
use std::collections::HashMap;
use poise::serenity_prelude as serenity;
use serenity::all::{CreateEmbed, EditMessage, GetMessages};
use serenity::all::colours::roles::GREEN;
use crate::utils::{CommandResult, Context};

#[poise::command(
slash_command,
prefix_command,
guild_only,
)]
pub async fn top(ctx: Context<'_>, target_word: String) -> CommandResult {
    let mut user_counts = HashMap::new();

    // `.clone()` necesario para evitar el error de hilos por no implementar Send
    // SAFTETY: el comando solo funciona en un servidor, por lo que siempre habrá un guild
    let guild = ctx.guild().unwrap().clone();

    // Get the channels of the guild
    let channels = guild.channels(ctx.http()).await?;

    // Enviar el mensaje de carga y obtener el ID del mensaje para editarlo más tarde.
    let loading = ctx.say("Cargando...").await?;
    let loading_id = loading.message().await?.id;

    // Iterate over each channel
    for channel in channels.values() {
        // Get the messages from the channel
        let messages = channel.messages(ctx.http(), GetMessages::new().limit(100)).await?;
        // Iterate over each message
        for message in messages {
            // Check if the message contains the target word
            if message.content.contains(&target_word) {
                // Count each occurrence
                *user_counts.entry(message.author.id).or_insert(0) += 1;
            }
        }
    }

    // Sort the users by their counts
    let mut user_counts: Vec<_> = user_counts.into_iter().collect();
    user_counts.sort_unstable_by_key(|&(_, count)| Reverse(count));

    // Get the top users
    let top_users = user_counts.into_iter().take(10).collect::<Vec<_>>();

    // Send the top users
    let embed = CreateEmbed::default()
        .title(format!("Top Usuarios que usaron la palabra \"{}\"", target_word))
        .description(top_users
            .iter()
            .map(|(user_id, count)| format!("- <@{user_id}>: {count}"))
            .collect::<Vec<_>>()
            .join("\n")
        )
        .color(GREEN);

    // Builder del mensaje (necesario para editar el mensaje en el método `.edit_message()`)
    let builder = EditMessage::default()
        .content("")
        .embed(embed);

    // Obtener el ID del canal y editar el mensaje
    let channel_id = ctx.channel_id();
    channel_id.edit_message(&ctx.http(), loading_id, builder).await?;

    Ok(())
}