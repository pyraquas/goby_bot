//! Self service role selection posted in the "Abaye des vocations" channel.

use poise::serenity_prelude as serenity;

use super::{ARCHI_ROLE, MTG_ROLE};
use crate::Error;

const MTG_BUTTON_ID: &str = "goby:role:mtg";
const ARCHI_BUTTON_ID: &str = "goby:role:archi";

/// Message carrying one toggle button per selectable role.
pub fn message() -> serenity::CreateMessage {
    let buttons = vec![
        serenity::CreateButton::new(MTG_BUTTON_ID)
            .label(MTG_ROLE)
            .style(serenity::ButtonStyle::Primary),
        serenity::CreateButton::new(ARCHI_BUTTON_ID)
            .label(ARCHI_ROLE)
            .style(serenity::ButtonStyle::Primary),
    ];

    serenity::CreateMessage::new()
        .content(
            "**Abaye des vocations**\nClique sur une vocation pour l'obtenir, \
             clique à nouveau pour l'abandonner.",
        )
        .components(vec![serenity::CreateActionRow::Buttons(buttons)])
}

/// Grants or removes the role tied to the pressed button.
pub async fn handle(
    ctx: &serenity::Context,
    interaction: &serenity::ComponentInteraction,
) -> Result<(), Error> {
    let role_name = match interaction.data.custom_id.as_str() {
        MTG_BUTTON_ID => MTG_ROLE,
        ARCHI_BUTTON_ID => ARCHI_ROLE,
        // Not one of our buttons.
        _ => return Ok(()),
    };

    let (Some(guild_id), Some(member)) = (interaction.guild_id, interaction.member.as_ref()) else {
        return Ok(());
    };

    let answer = match find_role(ctx, guild_id, role_name).await? {
        None => format!(
            "Le rôle `{role_name}` n'existe pas encore, un administrateur doit lancer `/init`."
        ),
        Some(role) if member.roles.contains(&role.id) => {
            member.remove_role(&ctx.http, role.id).await?;
            format!("Le rôle **{role_name}** t'a été retiré.")
        }
        Some(role) => {
            member.add_role(&ctx.http, role.id).await?;
            format!("Le rôle **{role_name}** t'a été attribué.")
        }
    };

    interaction
        .create_response(
            ctx,
            serenity::CreateInteractionResponse::Message(
                serenity::CreateInteractionResponseMessage::new()
                    .content(answer)
                    .ephemeral(true),
            ),
        )
        .await?;

    Ok(())
}

/// Looks a guild role up by name, over the REST API so a cold cache is fine.
pub async fn find_role(
    ctx: &serenity::Context,
    guild_id: serenity::GuildId,
    name: &str,
) -> Result<Option<serenity::Role>, Error> {
    let roles = guild_id.roles(ctx).await?;
    Ok(roles.into_values().find(|role| role.name == name))
}
