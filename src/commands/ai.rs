use crate::utils::{CommandResult, Context};
use openai_api_rs::v1::api::Client;
use openai_api_rs::v1::chat_completion;
use openai_api_rs::v1::chat_completion::ChatCompletionRequest;
use poise::CreateReply;
use regex::Regex;
use serenity::all::{ButtonStyle, CreateButton};
use serenity::builder::CreateActionRow;
use crate::utils::debug::UnwrapResult;

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
    let result = request_ai(&prompt)?;

    let action_row = vec![CreateActionRow::Buttons(vec![
        CreateButton::new("close")
            .style(ButtonStyle::Danger)
            .label("Cerrar")
    ])];

    let reply = CreateReply::default()
        .content(result)
        .components(action_row);

    loading.edit(ctx, reply).await?;

    Ok(())
}

pub fn request_ai(prompt: &String) -> UnwrapResult<String> {
    let url = dotenvy::var("OPENAI_API_BASE").expect("OPENAI_API_BASE not set");
    let api_key = dotenvy::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set");
    let model = dotenvy::var("AI_MODEL").expect("AI_MODEL not set");
    let client = Client::new_with_endpoint(url, api_key);

    let req = ChatCompletionRequest::new(
        model,
        vec![
            chat_completion::ChatCompletionMessage {
                role: chat_completion::MessageRole::user,
                content: chat_completion::Content::Text(prompt.to_string()),
                name: None,
            },
            chat_completion::ChatCompletionMessage {
                role: chat_completion::MessageRole::system,
                content: chat_completion::Content::Text(String::from(
                    "Te llamas Leafy, eres un Bot de Discord y tu creador es CrawKatt. Nunca superes los 2000 carácteres en tus respuestas.",
                )),
                name: None,
            },
        ],
    ).max_tokens(1024);

    let result = client.chat_completion(req)?;
    let content = result
        .choices
        .into_iter()
        .next()
        .and_then(|choice| choice.message.content)
        .unwrap_or_else(|| "❌ No se obtuvo respuesta.".to_string());

    let re = Regex::new(r"(?s)<think>.*?</think>")?
        .replace_all(&content, "")
        .trim()
        .to_string();

    Ok(re)
}
