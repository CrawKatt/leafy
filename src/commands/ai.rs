use crate::utils::{CommandResult, Context};
use openai_api_rs::v1::api::Client;
use openai_api_rs::v1::chat_completion;
use openai_api_rs::v1::chat_completion::ChatCompletionRequest;
use poise::CreateReply;
use regex::Regex;
use serenity::all::{ButtonStyle, CreateButton};
use serenity::builder::CreateActionRow;

#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    category = "Info",
    guild_cooldown = 15,
)]
pub async fn ask(
    ctx: Context<'_>,
    #[description = "Texto a enviar al modelo de IA"]
    #[rest]
    prompt: String
) -> CommandResult {
    let loading = ctx.say("Cargando...").await?;
    let url = dotenvy::var("OPENAI_API_BASE")?;
    let api_key = dotenvy::var("OPENAI_API_KEY")?;
    let model = dotenvy::var("AI_MODEL")?;
    let client = Client::new_with_endpoint(url, api_key);

    let req = ChatCompletionRequest::new(
        model,
        vec![
            chat_completion::ChatCompletionMessage {
                role: chat_completion::MessageRole::user,
                content: chat_completion::Content::Text(prompt),
                name: None,
            },
            chat_completion::ChatCompletionMessage {
                role: chat_completion::MessageRole::system,
                content: chat_completion::Content::Text(String::from("Te llamas Leafy, eres un Bot de Discord y tu creador es CrawKatt. Nunca superes los 2000 carácteres en tus respuestas.")),
                name: None,
            }
        ],
    ).max_tokens(1024);

    let Ok(result) = client.chat_completion(req) else {
        let reply = CreateReply::default().content("Ocurrió un error al solicitar la respuesta por IA");
        loading.edit(ctx, reply).await?;
        return Ok(())
    };

    let message = result.choices.first().and_then(|char| char.message.content.as_ref());
    let Some(mut message) = message.map(std::string::ToString::to_string) else {
        let reply = CreateReply::default().content("Ocurrió un error al solicitar la respuesta por IA");
        loading.edit(ctx, reply).await?;
        return Ok(())
    };

    let re = Regex::new(r"(?s)<think>.*?</think>")?;
    message = re.replace_all(&message, "").trim().to_string();

    let action_row = vec![CreateActionRow::Buttons(vec![
        CreateButton::new("close")
            .style(ButtonStyle::Danger)
            .label("Cerrar")
        ])
    ];

    let reply = CreateReply::default()
        .content(message)
        .components(action_row);

    loading.edit(ctx, reply).await?;

    Ok(())
}
