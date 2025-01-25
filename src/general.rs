use poise::{
    serenity_prelude::{CreateActionRow, CreateButton, CreateEmbed, CreateEmbedFooter, Timestamp},
    CreateReply,
};

use crate::Error;

#[poise::command(prefix_command, slash_command)]
pub async fn link(ctx: crate::Context<'_>) -> Result<(), Error> {
    let author = ctx.author();

    let link_embed = CreateEmbed::new()
    .title("🌐 Link Your Discord Account")
    .description("🚀 Connect your Discord account to the SwampHacks portal to unlock all features!")
    .url(format!(
        "https://app.swamphacks.com/hacker/link/discord?snowflake={}",
        author.id
    ))
    .thumbnail(author.avatar_url().unwrap_or(author.default_avatar_url()))
    .color(0x1E90FF) // A nice blue color
    .field("Why Link?", "🔗  Linking your discord account to your SwampHacks portal allows automatic role syncing and more!", false)
    .field("Quick Steps", "1️⃣ Click the button below.\n2️⃣ Authorize your Discord account.\n3️⃣ You're all set!", false)
    .footer(CreateEmbedFooter::new("SwampHacks"))
    .timestamp(Timestamp::now()); // Adds the current timestamp

    let message = CreateReply {
        embeds: vec![link_embed],
        components: Some(vec![CreateActionRow::Buttons(vec![
            CreateButton::new_link(format!(
                "https://app.swamphacks.com/hacker/link/discord?snowflake={}",
                author.id
            ))
            .label("Link Discord")
            .style(poise::serenity_prelude::ButtonStyle::Primary),
        ])]),
        ephemeral: Some(true),
        ..Default::default()
    };

    ctx.send(message).await?;

    Ok(())
}
