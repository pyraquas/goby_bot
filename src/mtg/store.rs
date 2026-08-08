//! Persistence of the card lists registered with `/have`.
//!
//! Everything is kept in memory and mirrored to a JSON file, so lists survive a
//! restart without pulling a database in.

use std::collections::HashMap;
use std::path::PathBuf;

use poise::serenity_prelude as serenity;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use super::card_list::{Card, CardList};
use crate::Error;

/// Directory holding the bot state, overridable with `GOBY_DATA_DIR`.
const DEFAULT_DATA_DIR: &str = "data";
const FILE_NAME: &str = "mtg_collections.json";

/// Owner of a card list. Lists are per guild, a member may play on several.
type Key = (serenity::GuildId, serenity::UserId);

/// One member card list, as written on disk.
#[derive(Debug, Serialize, Deserialize)]
struct Record {
    guild_id: serenity::GuildId,
    user_id: serenity::UserId,
    cards: Vec<Card>,
}

/// A card the asker is looking for and a member holds.
#[derive(Debug)]
pub struct Wanted {
    pub name: String,
    /// Copies the asker is looking for.
    pub requested: u32,
    /// Copies the member holds.
    pub owned: u32,
}

/// Cards of one member that another member is looking for.
#[derive(Debug)]
pub struct Match {
    pub user_id: serenity::UserId,
    pub cards: Vec<Wanted>,
}

/// The card lists of every member.
#[derive(Debug)]
pub struct Store {
    path: PathBuf,
    collections: Mutex<HashMap<Key, Vec<Card>>>,
}

impl Store {
    /// Loads the card lists from disk, starting empty when there is no file yet.
    pub async fn load() -> Result<Self, Error> {
        let path = data_dir().join(FILE_NAME);

        let collections = match tokio::fs::read(&path).await {
            Ok(bytes) => serde_json::from_slice::<Vec<Record>>(&bytes)?
                .into_iter()
                .map(|record| ((record.guild_id, record.user_id), record.cards))
                .collect(),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => HashMap::new(),
            Err(error) => return Err(error.into()),
        };

        Ok(Self {
            path,
            collections: Mutex::new(collections),
        })
    }

    /// Replaces the card list of a member, `/have` overwrites the previous one.
    pub async fn set(
        &self,
        guild_id: serenity::GuildId,
        user_id: serenity::UserId,
        list: CardList,
    ) -> Result<(), Error> {
        let mut collections = self.collections.lock().await;
        collections.insert((guild_id, user_id), list.cards);
        save(&self.path, &collections).await
    }

    /// Card list a member registered, if any.
    pub async fn get(
        &self,
        guild_id: serenity::GuildId,
        user_id: serenity::UserId,
    ) -> Option<CardList> {
        let collections = self.collections.lock().await;
        collections
            .get(&(guild_id, user_id))
            .map(|cards| CardList::from(cards.clone()))
    }

    /// Members of the guild owning at least one of the wanted cards, the ones
    /// with the most matches first. A member asking for cards they registered
    /// themselves appears in the answer like anyone else.
    pub async fn matches(&self, guild_id: serenity::GuildId, wanted: &CardList) -> Vec<Match> {
        let requested = wanted.quantities();
        let collections = self.collections.lock().await;

        let mut matches: Vec<Match> = collections
            .iter()
            .filter(|((owner_guild, _), _)| *owner_guild == guild_id)
            .filter_map(|((_, owner), cards)| {
                // Pairing both counts here keeps them from drifting apart, a
                // card is kept only when the asker wants it.
                let cards: Vec<Wanted> = cards
                    .iter()
                    .filter_map(|card| {
                        requested.get(&card.key()).map(|&requested| Wanted {
                            name: card.name.clone(),
                            requested,
                            owned: card.quantity,
                        })
                    })
                    .collect();

                (!cards.is_empty()).then_some(Match {
                    user_id: *owner,
                    cards,
                })
            })
            .collect();

        matches.sort_by_key(|owner| std::cmp::Reverse(owner.cards.len()));
        matches
    }
}

/// Writes the whole file through a temporary copy, so an interrupted write
/// cannot leave a truncated file behind.
async fn save(path: &PathBuf, collections: &HashMap<Key, Vec<Card>>) -> Result<(), Error> {
    let records: Vec<Record> = collections
        .iter()
        .map(|((guild_id, user_id), cards)| Record {
            guild_id: *guild_id,
            user_id: *user_id,
            cards: cards.clone(),
        })
        .collect();

    if let Some(directory) = path.parent() {
        tokio::fs::create_dir_all(directory).await?;
    }

    let temporary = path.with_extension("json.tmp");
    tokio::fs::write(&temporary, serde_json::to_vec_pretty(&records)?).await?;
    tokio::fs::rename(&temporary, path).await?;

    Ok(())
}

fn data_dir() -> PathBuf {
    std::env::var("GOBY_DATA_DIR")
        .unwrap_or_else(|_| DEFAULT_DATA_DIR.to_owned())
        .into()
}
