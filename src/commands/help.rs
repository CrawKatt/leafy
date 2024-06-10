use std::collections::HashMap;
use std::fmt::Write;

use poise::{Command, CreateReply};
use serenity::all::{CreateActionRow, CreateEmbed, CreateEmbedFooter, CreateSelectMenu, CreateSelectMenuKind, CreateSelectMenuOption};

use crate::utils::{CommandResult, Context, Data, Error};
use crate::utils::debug::UnwrapResult;

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
    let description = get_command_categories(&ctx.framework().options.commands)?;

    let reply = CreateReply::default()
        .ephemeral(true)
        .embed(CreateEmbed::default()
            .title("Help")
            .color(0x0000_ff00)
            .footer(CreateEmbedFooter::new("© CrawKatt")) // Colocar icon URL a futuro
            .description(description)
        )
        .components(vec![action_row.clone()]);

    ctx.send(reply).await?;

    Ok(())
}

pub fn get_command_categories(commands: &Vec<Command<Data, Error>>) -> UnwrapResult<String> {
    let mut categories: HashMap<String, Vec<String>> = HashMap::new();
    for command in commands {
        let new = &String::new();
        let category = command.category.as_ref().unwrap_or(new);
        let command_name = format!("`${}` {}", command.name, command.description.as_ref().unwrap_or(new));
        categories.entry(category.to_string()).or_default().push(command_name);
    }

    let mut description = String::new();
    for (category, command_names) in &categories {
        writeln!(description, "**{category}:**")?;
        for command_name in command_names {
            writeln!(description, "{command_name}")?;
        }
        writeln!(description)?;
    }

    Ok(description)
}

pub fn filter_categories(commands: &Vec<Command<Data, Error>>, selected_category: &str) -> UnwrapResult<String> {
    let mut categories: HashMap<String, Vec<String>> = HashMap::new();
    for command in commands {
        let new = &String::new();
        let category = command.category.as_ref().unwrap_or(new);
        if category.to_lowercase() == selected_category.to_lowercase() {
            let command_name = format!("${} ", command.name);
            categories.entry(category.to_string()).or_default().push(command_name);
        }
    }

    let mut description = String::new();
    for (category, command_names) in &categories {
        writeln!(description, "**{category}:**\n```")?;
        for command_name in command_names {
            writeln!(description, "{command_name}")?;
        }
        writeln!(description, "```")?;
    }

    Ok(description)
}