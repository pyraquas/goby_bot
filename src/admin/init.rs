//! Guild setup performed by `/init`.
//!
//! Every step is idempotent: an object that already exists is left untouched.

use poise::serenity_prelude as serenity;

use super::role_select;
use super::{
    ARCHI_CATEGORY, ARCHI_CHANNEL, ARCHI_ROLE, JOB_CHANNEL, MTG_CATEGORY, MTG_CHANNEL, MTG_ROLE,
};
use crate::Error;

/// Feature that `/init` can set up.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Feature {
    Mtg,
    Archi,
    Job,
    All,
}

/// What a single `/init` run changed, so the admin knows what was already there.
#[derive(Debug, Default)]
pub struct InitReport {
    pub created: Vec<String>,
    pub skipped: Vec<String>,
}

impl InitReport {
    fn created(&mut self, what: impl Into<String>) {
        self.created.push(what.into());
    }

    fn skipped(&mut self, what: impl Into<String>) {
        self.skipped.push(what.into());
    }
}

/// Creates the roles and channels the requested feature needs.
pub async fn run(
    ctx: &serenity::Context,
    guild_id: serenity::GuildId,
    feature: Feature,
) -> Result<InitReport, Error> {
    let mut report = InitReport::default();

    match feature {
        Feature::Mtg => init_mtg(ctx, guild_id, &mut report).await?,
        Feature::Archi => init_archi(ctx, guild_id, &mut report).await?,
        Feature::Job => init_job(ctx, guild_id, &mut report).await?,
        Feature::All => {
            init_mtg(ctx, guild_id, &mut report).await?;
            init_archi(ctx, guild_id, &mut report).await?;
            init_job(ctx, guild_id, &mut report).await?;
        }
    }

    Ok(report)
}

async fn init_mtg(
    ctx: &serenity::Context,
    guild_id: serenity::GuildId,
    report: &mut InitReport,
) -> Result<(), Error> {
    let role_id = ensure_role(ctx, guild_id, MTG_ROLE, report).await?;
    let category_id = ensure_private_category(ctx, guild_id, MTG_CATEGORY, role_id, report).await?;
    ensure_text_channel(ctx, guild_id, MTG_CHANNEL, category_id, role_id, report).await?;
    Ok(())
}

async fn init_archi(
    ctx: &serenity::Context,
    guild_id: serenity::GuildId,
    report: &mut InitReport,
) -> Result<(), Error> {
    let role_id = ensure_role(ctx, guild_id, ARCHI_ROLE, report).await?;
    let category_id =
        ensure_private_category(ctx, guild_id, ARCHI_CATEGORY, role_id, report).await?;
    ensure_text_channel(ctx, guild_id, ARCHI_CHANNEL, category_id, role_id, report).await?;
    Ok(())
}

/// Creates the read only channel carrying the role selection message.
async fn init_job(
    ctx: &serenity::Context,
    guild_id: serenity::GuildId,
    report: &mut InitReport,
) -> Result<(), Error> {
    if find_channel(ctx, guild_id, JOB_CHANNEL, serenity::ChannelType::Text)
        .await?
        .is_some()
    {
        report.skipped(format!("salon `{JOB_CHANNEL}`"));
        return Ok(());
    }

    // Members may read and react, only the bot writes here.
    let read_only = serenity::PermissionOverwrite {
        allow: serenity::Permissions::VIEW_CHANNEL,
        deny: serenity::Permissions::SEND_MESSAGES
            | serenity::Permissions::SEND_MESSAGES_IN_THREADS
            | serenity::Permissions::CREATE_PUBLIC_THREADS
            | serenity::Permissions::CREATE_PRIVATE_THREADS,
        kind: serenity::PermissionOverwriteType::Role(guild_id.everyone_role()),
    };

    let channel = guild_id
        .create_channel(
            ctx,
            serenity::CreateChannel::new(JOB_CHANNEL)
                .kind(serenity::ChannelType::Text)
                .topic("Choisis les fonctionnalités du serveur qui t'intéressent.")
                .permissions(vec![read_only]),
        )
        .await?;
    report.created(format!("salon `{JOB_CHANNEL}`"));

    channel.id.send_message(ctx, role_select::message()).await?;
    report.created("message de sélection des rôles");

    Ok(())
}

async fn ensure_role(
    ctx: &serenity::Context,
    guild_id: serenity::GuildId,
    name: &str,
    report: &mut InitReport,
) -> Result<serenity::RoleId, Error> {
    if let Some(role) = role_select::find_role(ctx, guild_id, name).await? {
        report.skipped(format!("rôle `{name}`"));
        return Ok(role.id);
    }

    let role = guild_id
        .create_role(
            ctx,
            serenity::EditRole::new()
                .name(name)
                .hoist(true)
                .mentionable(true),
        )
        .await?;
    report.created(format!("rôle `{name}`"));

    Ok(role.id)
}

/// Creates a category only the given role can see.
async fn ensure_private_category(
    ctx: &serenity::Context,
    guild_id: serenity::GuildId,
    name: &str,
    role_id: serenity::RoleId,
    report: &mut InitReport,
) -> Result<serenity::ChannelId, Error> {
    if let Some(channel) =
        find_channel(ctx, guild_id, name, serenity::ChannelType::Category).await?
    {
        report.skipped(format!("catégorie `{name}`"));
        return Ok(channel.id);
    }

    let channel = guild_id
        .create_channel(
            ctx,
            serenity::CreateChannel::new(name)
                .kind(serenity::ChannelType::Category)
                .permissions(private_overwrites(guild_id, role_id)),
        )
        .await?;
    report.created(format!("catégorie `{name}`"));

    Ok(channel.id)
}

/// Creates the default text channel of a private category.
async fn ensure_text_channel(
    ctx: &serenity::Context,
    guild_id: serenity::GuildId,
    name: &str,
    category_id: serenity::ChannelId,
    role_id: serenity::RoleId,
    report: &mut InitReport,
) -> Result<serenity::ChannelId, Error> {
    let existing = find_channel(ctx, guild_id, name, serenity::ChannelType::Text)
        .await?
        .filter(|channel| channel.parent_id == Some(category_id));

    if let Some(channel) = existing {
        report.skipped(format!("salon `{name}`"));
        return Ok(channel.id);
    }

    // Same overwrites as the parent category, so the channel stays in sync with it.
    let channel = guild_id
        .create_channel(
            ctx,
            serenity::CreateChannel::new(name)
                .kind(serenity::ChannelType::Text)
                .category(category_id)
                .permissions(private_overwrites(guild_id, role_id)),
        )
        .await?;
    report.created(format!("salon `{name}`"));

    Ok(channel.id)
}

/// Hides a channel from everyone but the holders of `role_id`.
fn private_overwrites(
    guild_id: serenity::GuildId,
    role_id: serenity::RoleId,
) -> Vec<serenity::PermissionOverwrite> {
    vec![
        serenity::PermissionOverwrite {
            allow: serenity::Permissions::empty(),
            deny: serenity::Permissions::VIEW_CHANNEL,
            kind: serenity::PermissionOverwriteType::Role(guild_id.everyone_role()),
        },
        serenity::PermissionOverwrite {
            allow: serenity::Permissions::VIEW_CHANNEL,
            deny: serenity::Permissions::empty(),
            kind: serenity::PermissionOverwriteType::Role(role_id),
        },
    ]
}

async fn find_channel(
    ctx: &serenity::Context,
    guild_id: serenity::GuildId,
    name: &str,
    kind: serenity::ChannelType,
) -> Result<Option<serenity::GuildChannel>, Error> {
    let channels = guild_id.channels(ctx).await?;
    Ok(channels
        .into_values()
        .find(|channel| channel.kind == kind && channel.name == name))
}
