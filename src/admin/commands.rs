//! Administration commands, see `commands.md` in this directory.

use super::init;
use crate::{Context, Error};

/// `feature` argument of `/init`.
#[derive(Debug, Clone, Copy, poise::ChoiceParameter)]
pub enum FeatureChoice {
    #[name = "mtg"]
    Mtg,
    #[name = "archi"]
    Archi,
    #[name = "job"]
    Job,
    #[name = "all"]
    All,
}

impl From<FeatureChoice> for init::Feature {
    fn from(choice: FeatureChoice) -> Self {
        match choice {
            FeatureChoice::Mtg => init::Feature::Mtg,
            FeatureChoice::Archi => init::Feature::Archi,
            FeatureChoice::Job => init::Feature::Job,
            FeatureChoice::All => init::Feature::All,
        }
    }
}

/// Crée le rôle et la catégorie liés à une fonctionnalité.
#[poise::command(
    slash_command,
    guild_only,
    required_permissions = "ADMINISTRATOR",
    category = "Administration"
)]
pub async fn init(
    ctx: Context<'_>,
    #[description = "Fonctionnalité à initialiser"] feature: FeatureChoice,
) -> Result<(), Error> {
    let Some(guild_id) = ctx.guild_id() else {
        ctx.say("Cette commande n'est utilisable que dans un serveur.")
            .await?;
        return Ok(());
    };

    // Guild setup takes several REST calls, keep Discord from timing out.
    ctx.defer_ephemeral().await?;

    let report = match init::run(ctx.serenity_context(), guild_id, feature.into()).await {
        Ok(report) => report,
        Err(error) => {
            ctx.say(format!("L'initialisation a échoué : {error}"))
                .await?;
            return Err(error);
        }
    };

    ctx.say(format_report(&report)).await?;
    Ok(())
}

fn format_report(report: &init::InitReport) -> String {
    let mut answer = String::new();

    if report.created.is_empty() {
        answer.push_str("Rien à faire, la fonctionnalité est déjà initialisée.");
    } else {
        answer.push_str("Créé :");
        for item in &report.created {
            answer.push_str(&format!("\n- {item}"));
        }
    }

    if !report.skipped.is_empty() {
        answer.push_str("\nDéjà présent :");
        for item in &report.skipped {
            answer.push_str(&format!("\n- {item}"));
        }
    }

    answer
}
