use std::fs::remove_file;
use std::collections::HashMap;
use reqwest::get;
use image::DynamicImage;
use poise::serenity_prelude as serenity;
use plantita_welcomes::create_welcome::combine_images;
use serenity::all::{ChannelId, Context, CreateAttachment, CreateEmbed, CreateEmbedAuthor, CreateEmbedFooter, CreateMessage, GuildId, Member, Mentionable, RoleId, User};
use crate::{location, DB};
use crate::handlers::goodbye::SanctionedUsers;
use crate::utils::CommandResult;
use crate::utils::config::GuildData;
use crate::utils::debug::{IntoUnwrapResult, UnwrapErrors, UnwrapLog};

pub async fn handler(
    ctx: &Context,
    new_member: &Member,
) -> CommandResult {
    let guild_id = new_member.guild_id;
    let user = &new_member.user;
    let channel_id = GuildData::verify_data(guild_id).await?
        .unwrap_log(location!())?
        .channels
        .welcome
        .into_result()?
        .parse::<ChannelId>()?;

    welcome(ctx, user, channel_id, guild_id).await?;
    has_sanction_roles(ctx, new_member, guild_id).await?;

    Ok(())
}

async fn has_sanction_roles(ctx: &Context, new_member: &Member, guild_id: GuildId) -> CommandResult {
    let user_id = new_member.user.id;

    let existing_data: Option<SanctionedUsers> = DB
        .select(("sanctioned_users", user_id.to_string()))
        .await?;

    if let Some(data) = existing_data {
        let member = guild_id.member(ctx, user_id).await?;
        let roles: Vec<RoleId> = data
            .roles
            .into_iter()
            .filter_map(|role_id| role_id.parse::<RoleId>().ok())
            .collect();
        let roles_mentions = roles
            .iter()
            .map(|role| role.mention().to_string())
            .collect::<Vec<String>>()
            .join(", ");

        member.add_roles(ctx, &roles).await?;

        let log_channel = GuildData::verify_data(guild_id).await?
            .into_result()?
            .channels
            .logs
            .into_result()?
            .parse::<ChannelId>()?;

        let member_mention = member.mention();
        let member_joined_at = member.joined_at.into_result()?;

        let description = format!("Usuario detectado: {member_mention} \nRoles asignados previamente {roles_mentions}");
        let footer = CreateEmbedFooter::new("Fecha de reingreso:");
        let embed = CreateEmbed::default()
            .title("⚠️ Un usuario sancionado ha salido y regresado al servidor")
            .author(CreateEmbedAuthor::new(member.display_name())
                .icon_url(member.face()))
            .description(description)
            .color(0x00FF_0000)
            .footer(footer)
            .timestamp(member_joined_at);

        log_channel.send_message(&ctx.http, CreateMessage::default().embed(embed)).await?;
    }

    Ok(())
}

async fn welcome(ctx: &Context, user: &User, channel_id: ChannelId, guild_id: GuildId) -> CommandResult {
    let welcome_message = GuildData::verify_data(guild_id).await?
        .unwrap_log(location!())?
        .messages
        .welcome
        .into_result()?;

    let mut background = image::open("assets/background.png")?;
    let file = get_welcome_attachment(&mut background, user, 74, 74, 372).await?;

    let mut message_map = HashMap::new();
    message_map.insert("content", format!("Bienvenido {user} a {}. \n{welcome_message}", guild_id.name(&ctx.cache).unwrap_or_default()));
    let http = &ctx.http;
    let attachment = CreateAttachment::path(&file).await?;

    // En el attachment se puede pasar un archivo de imagen para la bienvenida
    http.send_message(channel_id, vec![attachment], &message_map).await?;

    // Borrar la imágen generada después de usarla
    remove_file(file)?;

    Ok(())
}

async fn get_welcome_attachment(background: &mut DynamicImage, user: &User, x: u32, y: u32, avatar_size: u32) -> Result<String, UnwrapErrors> {
    // Obtén la URL del avatar del usuario
    let avatar_url = user.face();

    // Descarga la imagen del avatar
    let response = get(avatar_url).await?;
    let bytes= response.bytes().await?;

    // Carga la imagen del avatar en memoria y redimensiona a 256x256
    let img = image::load_from_memory(&bytes)?;
    img.resize(256, 256, image::imageops::Lanczos3);

    // Guarda la imagen del avatar en un archivo temporal
    let output_path = format!("/tmp/{}_welcome.png", user.id);
    combine_images(background, &img, x, y, avatar_size)?;
    background.save(&output_path)?;

    Ok(output_path)
}