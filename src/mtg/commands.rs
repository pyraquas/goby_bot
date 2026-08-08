//! Magic: the Gathering commands, see `commands.md` in this directory.

use poise::serenity_prelude::{self as serenity, Mentionable};

use super::card_list::{self, CardList};
use super::store::{Match, Wanted};
use crate::{Context, Error};

/// Discord refuses messages over 2000 characters.
const MESSAGE_LIMIT: usize = 2000;
/// Cards named next to a player in the public answer, the private message
/// carries the full list.
const PREVIEW_CARDS: usize = 5;

/// Enregistre ta liste de cartes disponibles à l'échange ou à la vente.
#[poise::command(
    slash_command,
    guild_only,
    check = "super::access",
    category = "Magic: the Gathering"
)]
pub async fn have(
    ctx: Context<'_>,
    #[description = "Export ManaBox (.csv) ou liste texte de tes cartes disponibles"]
    card_list: serenity::Attachment,
) -> Result<(), Error> {
    let Some(guild_id) = ctx.guild_id() else {
        return Ok(());
    };

    // Downloading and parsing the file takes longer than Discord waits for.
    ctx.defer_ephemeral().await?;

    let Some(list) = read_card_list(ctx, &card_list).await? else {
        return Ok(());
    };

    let previous = ctx.data().mtg.get(guild_id, ctx.author().id).await;
    ctx.data()
        .mtg
        .set(guild_id, ctx.author().id, list.clone())
        .await?;

    let mut answer = format!(
        "Liste enregistrée : **{} cartes** ({} exemplaires) depuis `{}`.",
        list.len(),
        list.copies(),
        card_list.filename,
    );
    if let Some(previous) = previous {
        answer.push_str(&format!(
            "\nElle remplace ta liste précédente de {} cartes.",
            previous.len()
        ));
    }

    ctx.say(answer).await?;
    Ok(())
}

/// Affiche les joueurs qui possèdent les cartes de la liste fournie.
#[poise::command(
    slash_command,
    guild_only,
    check = "super::access",
    category = "Magic: the Gathering"
)]
pub async fn need(
    ctx: Context<'_>,
    #[description = "Export ManaBox (.csv) ou liste texte des cartes recherchées"]
    card_list: serenity::Attachment,
) -> Result<(), Error> {
    let Some(guild_id) = ctx.guild_id() else {
        return Ok(());
    };

    // The answer is public: everyone in the channel sees who owns what.
    ctx.defer().await?;

    let Some(wanted) = read_card_list(ctx, &card_list).await? else {
        return Ok(());
    };

    let matches = ctx.data().mtg.matches(guild_id, &wanted).await;

    if matches.is_empty() {
        ctx.say(format!(
            "Aucun joueur ne possède les {} cartes de `{}`.",
            wanted.len(),
            card_list.filename
        ))
        .await?;
        return Ok(());
    }

    let guild_name = ctx
        .partial_guild()
        .await
        .map_or_else(|| "ce serveur".to_owned(), |guild| guild.name);

    // Owners are told privately, so they get the full list without flooding the
    // channel. A closed inbox is reported back to the asker.
    let mut unreachable = Vec::new();
    for owner in &matches {
        let message = private_message(ctx, &guild_name, &owner.cards);
        if owner.user_id.direct_message(ctx, message).await.is_err() {
            unreachable.push(owner.user_id);
        }
    }

    ctx.say(public_report(
        &card_list.filename,
        &wanted,
        &matches,
        &unreachable,
    ))
    .await?;
    Ok(())
}

/// Downloads and parses an attached card list, answering the member and
/// returning `None` when the file cannot be used.
async fn read_card_list(
    ctx: Context<'_>,
    attachment: &serenity::Attachment,
) -> Result<Option<CardList>, Error> {
    if attachment.size > card_list::MAX_FILE_SIZE {
        ctx.say(format!(
            "`{}` est trop volumineux, la limite est de {} Kio.",
            attachment.filename,
            card_list::MAX_FILE_SIZE / 1024
        ))
        .await?;
        return Ok(None);
    }

    let Ok(content) = String::from_utf8(attachment.download().await?) else {
        ctx.say(format!(
            "`{}` doit être un fichier texte encodé en UTF-8.",
            attachment.filename
        ))
        .await?;
        return Ok(None);
    };

    match card_list::parse(&attachment.filename, &content) {
        Ok(list) => Ok(Some(list)),
        Err(error) => {
            ctx.say(explain(error, &attachment.filename)).await?;
            Ok(None)
        }
    }
}

/// Tells the member what is wrong with the file they attached.
fn explain(error: card_list::ListError, filename: &str) -> String {
    match error {
        card_list::ListError::UnknownColumns => format!(
            "Les colonnes de `{filename}` ne sont pas reconnues, il faut une colonne \
             `Name` et une colonne `Quantity`, comme dans un export ManaBox."
        ),
        card_list::ListError::Empty => format!(
            "Aucune carte n'a été trouvée dans `{filename}`. Attendu un export ManaBox \
             (`.csv`) ou une carte par ligne (`.txt`)."
        ),
    }
}

/// Private message telling an owner which of their cards are wanted, with the
/// number of copies asked for and, in brackets, the number they hold.
fn private_message(
    ctx: Context<'_>,
    guild_name: &str,
    cards: &[Wanted],
) -> serenity::CreateMessage {
    let header = format!(
        "{} cherche des cartes sur **{}** ({}), tu en possèdes **{}** :",
        ctx.author().mention(),
        guild_name,
        ctx.channel_id().mention(),
        cards.len(),
    );

    let lines: Vec<String> = cards
        .iter()
        .map(|card| {
            format!(
                "- {}× {} (tu en as {})",
                card.requested, card.name, card.owned
            )
        })
        .collect();

    serenity::CreateMessage::new().content(build_message(&header, &lines))
}

/// Public answer of `/need`, one line per player holding wanted cards.
fn public_report(
    filename: &str,
    wanted: &CardList,
    matches: &[Match],
    unreachable: &[serenity::UserId],
) -> String {
    let header = format!(
        "**{}** {} des {} cartes de `{}` :",
        matches.len(),
        if matches.len() > 1 {
            "joueurs possèdent"
        } else {
            "joueur possède"
        },
        wanted.len(),
        filename,
    );

    let mut lines: Vec<String> = matches
        .iter()
        .map(|owner| {
            format!(
                "- {} — {} cartes : {}",
                owner.user_id.mention(),
                owner.cards.len(),
                preview(&owner.cards)
            )
        })
        .collect();

    if unreachable.is_empty() {
        lines.push("Chacun a reçu sa liste en message privé.".to_owned());
    } else {
        let names: Vec<String> = unreachable
            .iter()
            .map(|user_id| user_id.mention().to_string())
            .collect();
        lines.push(format!(
            "Message privé impossible pour {}, contacte-les directement.",
            names.join(", ")
        ));
    }

    build_message(&header, &lines)
}

/// First few card names of a match, for the public answer.
fn preview(cards: &[Wanted]) -> String {
    let names: Vec<&str> = cards
        .iter()
        .take(PREVIEW_CARDS)
        .map(|card| card.name.as_str())
        .collect();

    match cards.len().checked_sub(PREVIEW_CARDS) {
        Some(rest) if rest > 0 => format!("{}, … (+{rest})", names.join(", ")),
        _ => names.join(", "),
    }
}

/// Joins the lines under the header, dropping the tail that would not fit in a
/// Discord message.
fn build_message(header: &str, lines: &[String]) -> String {
    // Room kept for the line announcing what was left out.
    const OVERFLOW_ROOM: usize = 40;

    let mut message = String::from(header);
    let mut shown = 0;

    for line in lines {
        if message.len() + line.len() + 1 + OVERFLOW_ROOM > MESSAGE_LIMIT {
            break;
        }
        message.push('\n');
        message.push_str(line);
        shown += 1;
    }

    if shown < lines.len() {
        message.push_str(&format!("\n… et {} de plus.", lines.len() - shown));
    }

    message
}

#[cfg(test)]
mod tests {
    use super::*;

    fn card(name: &str) -> Wanted {
        Wanted {
            name: name.to_owned(),
            requested: 1,
            owned: 1,
        }
    }

    #[test]
    fn preview_lists_every_card_when_they_all_fit() {
        assert_eq!(
            preview(&[card("Sol Ring"), card("Duress")]),
            "Sol Ring, Duress"
        );
    }

    #[test]
    fn preview_counts_the_cards_it_leaves_out() {
        let cards: Vec<Wanted> = (0..8).map(|index| card(&format!("Card {index}"))).collect();

        assert_eq!(
            preview(&cards),
            "Card 0, Card 1, Card 2, Card 3, Card 4, … (+3)"
        );
    }

    #[test]
    fn build_message_stays_under_the_discord_limit() {
        let lines: Vec<String> = (0..200).map(|index| format!("- line {index}")).collect();

        let message = build_message("header", &lines);

        assert!(message.len() <= MESSAGE_LIMIT);
        assert!(message.ends_with(" de plus."));
    }

    #[test]
    fn build_message_keeps_every_line_that_fits() {
        let message = build_message("header", &["- one".to_owned(), "- two".to_owned()]);

        assert_eq!(message, "header\n- one\n- two");
    }
}
