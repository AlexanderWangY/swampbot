use poise::{
    serenity_prelude::{CreateChannel, User},
    CreateReply,
};

use crate::Error;

#[poise::command(
    prefix_command,
    slash_command,
    required_permissions = "MODERATE_MEMBERS"
)]
pub async fn warn_user(
    ctx: crate::Context<'_>,
    #[description = "user"] name: User,
    #[description = "reason"] reason: String,
) -> Result<(), Error> {
    Ok(())
}
