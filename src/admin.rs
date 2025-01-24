use poise::{
    serenity_prelude::{
        CreateActionRow, CreateButton, CreateEmbed, CreateMessage, ReactionType, Timestamp,
    },
    CreateReply,
};

use crate::Error;

#[poise::command(prefix_command, slash_command, required_permissions = "ADMINISTRATOR")]
pub async fn init_sync(ctx: crate::Context<'_>) -> Result<(), Error> {
    let channel = ctx.channel_id();

    dbg!(channel);

    let embed = CreateEmbed::new()
        .title("Sync Discord Account")
        .description("Sync your discord account with your SwampHacks profile. This will give you the proper roles and access.")
        .field("Why Sync?", "Syncing every once in a while ensures that your current roles and status are up to date with our servers at Deguzman.cloud", false)
        .timestamp(Timestamp::now());

    let components = vec![CreateActionRow::Buttons(vec![CreateButton::new("sync")
        .label("Sync")
        .style(poise::serenity_prelude::ButtonStyle::Primary)])];

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
