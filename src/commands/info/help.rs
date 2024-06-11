use std::collections::HashMap;
use std::fmt::Write;

use poise::{Command, CreateReply};
use serenity::all::{CreateActionRow, CreateEmbed, CreateEmbedFooter, CreateSelectMenu, CreateSelectMenuKind, CreateSelectMenuOption};

use crate::utils::{CommandResult, Context, Data, Error};

pub const FOOTER_URL: &str = "https://cdn.discordapp.com/guilds/983473640387518504/users/395631548629516298/avatars/456f92e6e01310c808551557833f13ad.png?size=2048";

// Se debe manejar la interacción con el SelectMenu desde el handler de interacciones
// en `events.rs` utilizando el `custom_id` para identificar el `SelectMenu`.
// Se recomienda ver como ejemplo, el manejo de botones en `handlers/misc/buttons.rs`
// y el manejo de interacciones en `handlers/interactions.rs`.
#[poise::command(
    prefix_command,
    slash_command,
    category = "Info",
    guild_only,
    ephemeral,
    user_cooldown = 10,
    description_localized("es-ES", "Muestra un menú de ayuda con los comandos del Bot"),
    description_localized("en-US", "Shows a help menu with the Bot's commands"),
    description_localized("ja", "ボットのコマンドを表示するヘルプメニュー")
)]
pub async fn help(ctx: Context<'_>) -> CommandResult {
    let select_menu = CreateSelectMenu::new("help_menu", CreateSelectMenuKind::String {
        options: vec![
            CreateSelectMenuOption::new("Moderación", "Moderator").emoji('🛠'),
            CreateSelectMenuOption::new("Fun", "Fun").emoji('🎉'),
            CreateSelectMenuOption::new("Información", "Info").emoji('ℹ'),
            CreateSelectMenuOption::new("Audio", "Audio").emoji('🎵'),
        ],
    }).placeholder("Selecciona una categoría de comandos");
    let action_row = CreateActionRow::SelectMenu(select_menu);
    let commands = get_command_categories(&ctx.framework().options.commands);
    let description = format!("Prefix del Bot: `leafy`\n\n{commands}");

    let reply = CreateReply::default()
        .ephemeral(true)
        .embed(CreateEmbed::default()
            .title("Help")
            .color(0x0000_ff00)
            .footer(CreateEmbedFooter::new("© CrawKatt").icon_url(FOOTER_URL))
            .description(description)
        )
        .components(vec![action_row.clone()]);

    ctx.send(reply).await?;

    Ok(())
}

pub fn get_command_categories(commands: &[Command<Data, Error>]) -> String {
    let categories: HashMap<String, Vec<String>> = commands.iter()
        .map(|command| (command.category.clone().unwrap_or_default(), command.name.to_string()))
        .fold(HashMap::new(), |mut acc, (category, command_name)| {
            acc.entry(category).or_default().push(command_name);
            acc
        });

    create_description(&categories)
}

pub fn filter_categories(commands: &[Command<Data, Error>], selected_category: &str) -> String {
    let categories: HashMap<String, Vec<String>> = commands.iter()
        .filter(|command| command.category.as_ref().map(|c| c.to_lowercase()) == Some(selected_category.to_lowercase()))
        .map(|command| (command.category.clone().unwrap_or_default(), command.name.to_string()))
        .fold(HashMap::new(), |mut acc, (category, command_name)| {
            acc.entry(category).or_default().push(command_name);
            acc
        });

    create_description(&categories)
}

fn create_description(categories: &HashMap<String, Vec<String>>) -> String {
    categories.iter()
        .fold(String::new(), |mut description, (category, command_names)| {
            let commands = command_names.join("\n");
            write!(description, "**{category}:**\n```\n{commands}\n```\n").unwrap(); // SAFETY: la macro `write!` nunca falla
            description
        })
}