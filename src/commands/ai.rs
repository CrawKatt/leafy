use crate::utils::{CommandResult, Context};
use openai_api_rs::v1::api::Client;
use openai_api_rs::v1::chat_completion;
use openai_api_rs::v1::chat_completion::ChatCompletionRequest;
use poise::CreateReply;
use serenity::all::{ButtonStyle, CreateButton};
use serenity::builder::CreateActionRow;
use crate::utils::debug::IntoUnwrapResult;

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
    let client = Client::new_with_endpoint(url, api_key);

    let req = ChatCompletionRequest::new(
        "meta/llama3-70b-instruct".to_string(),
        vec![
            chat_completion::ChatCompletionMessage {
                role: chat_completion::MessageRole::user,
                content: chat_completion::Content::Text(prompt),
                name: None,
            },
            chat_completion::ChatCompletionMessage {
                role: chat_completion::MessageRole::system,
                content: chat_completion::Content::Text(String::from("Te llamas Plantita Ayudante. Nunca superes los 2000 carácteres en tus respuestas.")),
                name: None,
            }
        ],
    ).max_tokens(1024);

    let result = client.chat_completion(req)?;
    let message = result.choices[0].message.content.as_ref().into_result()?;
    
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