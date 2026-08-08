//! Magic: the Gathering commands and the card lists they work on.

pub mod card_list;
pub mod commands;
mod csv;
pub mod store;

use poise::serenity_prelude as serenity;

use crate::admin::{MTG_CATEGORY, MTG_ROLE, role_select};
use crate::{Context, Error};

/// Gate of every Magic command: the Magic role, inside the Magic category.
///
/// Tells the member what is missing rather than letting the command fail
/// silently, then returns `false` so poise skips the command.
pub async fn access(ctx: Context<'_>) -> Result<bool, Error> {
    let (Some(guild_id), Some(member)) = (ctx.guild_id(), ctx.author_member().await) else {
        deny(ctx, "Cette commande n'est utilisable que dans un serveur.").await?;
        return Ok(false);
    };

    let Some(role) = role_select::find_role(ctx.serenity_context(), guild_id, MTG_ROLE).await?
    else {
        let message = format!(
            "Le rôle `{MTG_ROLE}` n'existe pas, un administrateur doit lancer `/init mtg`."
        );
        deny(ctx, &message).await?;
        return Ok(false);
    };

    if !member.roles.contains(&role.id) {
        let message = format!("Il te faut le rôle `{MTG_ROLE}` pour utiliser cette commande.");
        deny(ctx, &message).await?;
        return Ok(false);
    }

    if !in_mtg_category(ctx).await? {
        let message =
            format!("Cette commande n'est utilisable que dans la catégorie `{MTG_CATEGORY}`.");
        deny(ctx, &message).await?;
        return Ok(false);
    }

    Ok(true)
}

/// True when the command was used in a channel of the Magic category, threads
/// of those channels included.
async fn in_mtg_category(ctx: Context<'_>) -> Result<bool, Error> {
    let Some(channel) = ctx.channel_id().to_channel(ctx).await?.guild() else {
        return Ok(false);
    };

    // A thread hangs under its parent channel, which is the one in the category.
    let channel = match channel.kind {
        serenity::ChannelType::PublicThread
        | serenity::ChannelType::PrivateThread
        | serenity::ChannelType::NewsThread => {
            let Some(parent_id) = channel.parent_id else {
                return Ok(false);
            };
            let Some(parent) = parent_id.to_channel(ctx).await?.guild() else {
                return Ok(false);
            };
            parent
        }
        _ => channel,
    };

    let Some(category_id) = channel.parent_id else {
        return Ok(false);
    };
    let Some(category) = category_id.to_channel(ctx).await?.guild() else {
        return Ok(false);
    };

    Ok(category.kind == serenity::ChannelType::Category && category.name == MTG_CATEGORY)
}

/// Answers a refused command, only the member who ran it sees the reason.
async fn deny(ctx: Context<'_>, message: &str) -> Result<(), Error> {
    ctx.send(
        poise::CreateReply::default()
            .content(message)
            .ephemeral(true),
    )
    .await?;
    Ok(())
}
