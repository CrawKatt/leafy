use crate::utils::debug::{UnwrapErrors, UnwrapResult};
use crate::utils::{CommandResult, Context};
use openai_api_rs::v1::api::Client;
use openai_api_rs::v1::chat_completion;
use openai_api_rs::v1::chat_completion::{ChatCompletionRequest, ChatCompletionResponse};
use poise::CreateReply;
use regex::Regex;
use serenity::all::{ButtonStyle, CreateButton};
use serenity::builder::CreateActionRow;

const SYSTEM_PROMPT: &str =
    "Eres un traductor multilingüe que convierte texto de un idioma a otro. Responde únicamente\
    con la traducción exacta del texto proporcionado, utilizando el alfabeto nativo del idioma de salida.\
    No uses transliteraciones (como romanji en japonés) ni caracteres que no sean propios del idioma de salida.\
    La respuesta debe ser limpia, sin paréntesis, notas, ni comentarios adicionales. Ejemplo:\
    Texto de entrada: 'Hola, ¿cómo estás?'\
    Traducción esperada (japonés): 'こんにちは、お元気ですか？'\
    Traducción incorrecta: 'Konnichiwa, ogenki desu ka?'";

#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    category = "Info",
    aliases("tr"),
    guild_cooldown = 15,
)]
pub async fn translate(
    ctx: Context<'_>,
    lang: String,
    #[description = "Texto a enviar al modelo de IA"]
    #[rest]
    prompt: String
) -> CommandResult {
    let loading = ctx.say("Cargando...").await?;
    let message = create_ai_message(prompt, lang).await;
    let Ok(msg) = message else {
        let reply = CreateReply::default().content("Ocurrió un error al obtener la traducción por IA");
        loading.edit(ctx, reply).await?;
        return Ok(())
    };

    let action_row = vec![CreateActionRow::Buttons(vec![
        CreateButton::new("close")
            .style(ButtonStyle::Danger)
            .label("Cerrar")
        ])
    ];

    let reply = CreateReply::default()
        .content(msg)
        .components(action_row);

    loading.edit(ctx, reply).await?;

    Ok(())
}

fn create_request(prompt: String, lang: &str) -> UnwrapResult<ChatCompletionResponse> {
    let url = dotenvy::var("OPENAI_API_BASE").expect("OPENAI_API_BASE not found");
    let api_key = dotenvy::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not found");
    let model = dotenvy::var("AI_MODEL").expect("AI_MODEL not found");
    let client = Client::new_with_endpoint(url, api_key);

    let req = ChatCompletionRequest::new(
        model,
        vec![
            chat_completion::ChatCompletionMessage {
                role: chat_completion::MessageRole::system,
                content: chat_completion::Content::Text(format!("{SYSTEM_PROMPT}. Idioma de salida: {lang}")),
                name: None,
            },
            chat_completion::ChatCompletionMessage {
                role: chat_completion::MessageRole::user,
                content: chat_completion::Content::Text(prompt),
                name: None,
            }
        ],
    ).max_tokens(1024);

    client.chat_completion(req).map_err(|_| UnwrapErrors::NoneError)
}

pub async fn create_ai_message(prompt: String, lang: String) -> UnwrapResult<String> {
    let response = create_request(prompt, &lang)?;
    let message = response
        .choices
        .into_iter()
        .next()
        .and_then(|char| char.message.content);

    let Some(mut message) = message else {
        return Err(UnwrapErrors::NoneError)
    };

    let re = Regex::new(r"(?s)<think>.*?</think>")?;
    message = re.replace_all(&message, "").trim().to_string();

    Ok(message)
}