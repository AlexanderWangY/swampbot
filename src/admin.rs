use poise::{
    serenity_prelude::{
        Channel, CreateActionRow, CreateButton, CreateEmbed, CreateEmbedFooter, CreateMessage,
        Timestamp,
    },
    CreateReply,
};

use crate::Error;

#[poise::command(prefix_command, slash_command, required_permissions = "MANAGE_CHANNELS")]
pub async fn init_sync(ctx: crate::Context<'_>) -> Result<(), Error> {
    let channel = ctx.channel_id();

    let embed = CreateEmbed::new()
    .title("Discord Account Synchronization")
    .description("Maintain up-to-date access and permissions by linking your SwampHacks profile with your Discord account.")
    .color(0x5865F2) // Discord blurple color
    .field(
        "Benefits of Account Synchronization",
        "• Automatic role and permission updates\n• Seamless access management\n• Consistent server experience",
        false,
    )
    .field(
        "Synchronization Guidelines",
        "• Sync after checking in to the hackathon to receive your roles.",
        false,
    )
    .footer(CreateEmbedFooter::new("Last updated"))
    .timestamp(Timestamp::now());

    let components = vec![CreateActionRow::Buttons(vec![
        CreateButton::new("sync")
            .style(poise::serenity_prelude::ButtonStyle::Success)
            .label("Sync")
            .emoji('🔄'), // Emoji for better visual
    ])];

    let message = CreateMessage::new().add_embed(embed).components(components);

    let http = ctx.http();

    let _ = channel.send_message(http, message).await.unwrap();

    let reply = CreateReply {
        content: Some(String::from("Successfully created sync embed.")),
        ephemeral: Some(true),
        ..Default::default()
    };

    ctx.send(reply).await?;

    Ok(())
}

#[poise::command(prefix_command, slash_command, required_permissions = "MANAGE_CHANNELS")]
pub async fn set_image_channel(
    ctx: crate::Context<'_>,
    #[description = "What channel for image printing"] channel: Channel,
) -> Result<(), Error> {
    if channel.clone().category().is_some() {
        let reply = CreateReply {
            content: Some(String::from(
                "You can not select a category as your image channel...",
            )),
            ephemeral: Some(true),
            ..Default::default()
        };

        ctx.send(reply).await?;
    } else {
        // This is a valid channel
        let channel_id = channel.id();

        let mut image_channel = ctx.data().image_channel.lock().await;
        *image_channel = Some(channel_id);

        let reply = CreateReply {
            content: Some(format!(
                "Channel with id {} set as image channel",
                channel_id
            )),
            ephemeral: Some(true),
            ..Default::default()
        };

        ctx.send(reply).await?;
    }

    Ok(())
}
