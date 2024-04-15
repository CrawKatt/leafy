use std::cmp::Reverse;
use std::collections::HashMap;
use poise::serenity_prelude as serenity;
use serenity::all::{CreateEmbed, EditMessage, GetMessages, Member};
use serenity::all::colours::roles::GREEN;
use crate::utils::{CommandResult, Context};

#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
)]
pub async fn top_words(ctx: Context<'_>, target: Member) -> CommandResult {

    let user_id = target.user.id;

    println!("Top words for user {}", user_id);
    let mut word_counts = HashMap::new();

    // Get the guild from the context
    let guild = ctx.guild().unwrap().clone(); // clone necesario para evitar el error de hilos por no implementar Send

    // Get the channels of the guild
    let channels = guild.channels(ctx.http()).await?;
    let loading = ctx.say("Cargando...").await?;
    let loading_id = loading.message().await?.id;

    // Iterate over each channel
    for channel in channels.values() {
        // Get the messages from the channel
        let messages = channel.messages(ctx.http(), GetMessages::new().limit(100)).await?;

        // Iterate over each message
        for message in messages {
            // Check if the message is from the specified user
            if message.author.id == user_id {
                // Split the message content into words
                let words = message.content.split_whitespace();

                // Count each word
                words.for_each(|word| *word_counts.entry(word.to_lowercase()).or_insert(0) += 1);
            }
        }
    }

    // Sort the words by their counts
    let mut word_counts: Vec<_> = word_counts.into_iter().collect();
    word_counts.sort_unstable_by_key(|&(_, count)| Reverse(count));

    // Get the top words
    let top_words = word_counts.into_iter().take(10).collect::<Vec<_>>();

    // Send the top words
    let embed = CreateEmbed::default()
        .title(format!("Top Palabras usadas por {}", target.distinct()))
        .description(top_words.iter().map(|(word, count)| format!("- {}: {}", word, count)).collect::<Vec<_>>().join("\n"))
        .color(GREEN);

    let builder = EditMessage::default()
        .content("")
        .embed(embed);

    let channel_id = ctx.channel_id();
    channel_id.edit_message(&ctx.http(), loading_id, builder).await?;

    Ok(())
}