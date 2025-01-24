use poise::{
    serenity_prelude::{
        CreateActionRow, CreateButton, CreateEmbed, CreateEmbedFooter, CreateMessage, Timestamp,
    },
    CreateReply,
};

use crate::Error;

#[poise::command(prefix_command, slash_command, required_permissions = "ADMINISTRATOR")]
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
