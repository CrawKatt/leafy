use crate::utils::{CommandResult, Context};
use openai_api_rs::v1::api::Client;
use openai_api_rs::v1::chat_completion;
use openai_api_rs::v1::chat_completion::ChatCompletionRequest;
use poise::CreateReply;
use regex::Regex;
use serenity::all::{ButtonStyle, CreateButton};
use serenity::builder::CreateActionRow;
use crate::utils::debug::IntoUnwrapResult;

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
    let url = dotenvy::var("OPENAI_API_BASE")?;
    let api_key = dotenvy::var("OPENAI_API_KEY")?;
    let model = dotenvy::var("AI_MODEL")?;
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

    let result = client.chat_completion(req)?;
    let mut message = result.choices[0].message.content.as_ref().into_result()?.clone();
    let re = Regex::new(r"(?s)<think>.*?</think>")?;
    message = re.replace_all(&message, "").to_string();
    message = message.trim().to_string();

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
