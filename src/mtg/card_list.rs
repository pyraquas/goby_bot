//! Parsing of the card list files attached to the Magic commands.
//!
//! Two formats are read. The one every deck builder exports, one card per line
//! with an optional quantity in front and optional set information after the
//! name, and the collection CSV ManaBox exports.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::csv;

/// Largest attachment accepted. A whole ManaBox collection export runs to a few
/// hundred kilobytes, a deck list to a few hundred bytes.
pub const MAX_FILE_SIZE: u32 = 4 * 1024 * 1024;

/// Header names the card name column goes by in collection exports.
const NAME_COLUMNS: [&str; 2] = ["name", "card name"];
/// Header names the number of copies goes by in collection exports.
const QUANTITY_COLUMNS: [&str; 2] = ["quantity", "count"];

/// Lines exporters use as section headers rather than as cards.
const SECTION_HEADERS: [&str; 8] = [
    "deck",
    "main",
    "maindeck",
    "commander",
    "sideboard",
    "companion",
    "maybeboard",
    "tokens",
];

/// A single entry of a card list.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
    pub name: String,
    pub quantity: u32,
}

impl Card {
    /// Key two entries are compared on, so casing and punctuation do not matter.
    pub fn key(&self) -> String {
        normalize(&self.name)
    }
}

/// A parsed card list, with one entry per distinct card.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CardList {
    pub cards: Vec<Card>,
}

impl CardList {
    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }

    /// Number of distinct cards.
    pub fn len(&self) -> usize {
        self.cards.len()
    }

    /// Number of copies, all cards taken together.
    pub fn copies(&self) -> u32 {
        self.cards.iter().map(|card| card.quantity).sum()
    }

    /// Number of copies of every card, by comparison key, to look a card of
    /// another list up.
    pub fn quantities(&self) -> HashMap<String, u32> {
        self.cards
            .iter()
            .map(|card| (card.key(), card.quantity))
            .collect()
    }
}

impl From<Vec<Card>> for CardList {
    fn from(cards: Vec<Card>) -> Self {
        Self { cards }
    }
}

/// Why a card list file could not be read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListError {
    /// A CSV whose header names no card column.
    UnknownColumns,
    /// A file holding no card at all.
    Empty,
}

/// Reads a card list, picking the format from the content of the file.
pub fn parse(filename: &str, content: &str) -> Result<CardList, ListError> {
    // A file written on Windows may open on a byte order mark.
    let content = content.trim_start_matches('\u{feff}');

    let list = match parse_csv(content) {
        Some(list) => list,
        // A file announcing itself as a CSV must not fall back to one card per
        // line, every row would turn into a card named after the whole row.
        None if is_csv_filename(filename) => return Err(ListError::UnknownColumns),
        None => parse_text(content),
    };

    if list.is_empty() {
        return Err(ListError::Empty);
    }

    Ok(list)
}

/// Reads a collection export. `None` when the content is not a CSV table whose
/// header names a card column, which is how a deck list is told apart.
fn parse_csv(content: &str) -> Option<CardList> {
    let mut records = csv::parse(content)
        .into_iter()
        .filter(|record| !csv::is_blank(record));

    let header = records.next()?;
    // A deck list line splits into a single field, or into fields that name no
    // column, so a real table is never confused with one.
    if header.len() < 2 {
        return None;
    }

    let name_column = column(&header, &NAME_COLUMNS)?;
    // ManaBox always exports a quantity, other exporters list one row per copy.
    let quantity_column = column(&header, &QUANTITY_COLUMNS);

    let cards = records.filter_map(|record| {
        // The name column of a table is authoritative, unlike a deck list line
        // it carries no set information to strip.
        let name = collapse_whitespace(record.get(name_column)?);
        if name.is_empty() {
            return None;
        }

        let quantity = quantity_column
            .and_then(|column| record.get(column))
            .map_or(Some(1), |value| value.trim().parse().ok())?;

        (quantity > 0).then_some(Card { name, quantity })
    });

    Some(merge(cards))
}

/// Position of the first header naming one of `candidates`.
fn column(header: &[String], candidates: &[&str]) -> Option<usize> {
    header.iter().position(|field| {
        candidates
            .iter()
            .any(|candidate| field.trim().eq_ignore_ascii_case(candidate))
    })
}

fn is_csv_filename(filename: &str) -> bool {
    filename
        .rsplit('.')
        .next()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("csv"))
}

/// Reads a deck list, one card per line.
fn parse_text(content: &str) -> CardList {
    merge(content.lines().filter_map(parse_line))
}

/// Collects entries into a list, merging the ones naming the same card.
fn merge(entries: impl IntoIterator<Item = Card>) -> CardList {
    let mut cards: Vec<Card> = Vec::new();
    let mut positions: HashMap<String, usize> = HashMap::new();

    for card in entries {
        match positions.get(&card.key()) {
            Some(&position) => cards[position].quantity += card.quantity,
            None => {
                positions.insert(card.key(), cards.len());
                cards.push(card);
            }
        }
    }

    CardList { cards }
}

/// Reads one line, or nothing when it holds no card.
fn parse_line(line: &str) -> Option<Card> {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') || line.starts_with("//") {
        return None;
    }

    let (quantity, rest) = split_quantity(line);
    let name = clean_name(rest);
    if name.is_empty() {
        return None;
    }

    // A bare "Sideboard" is a section header, the same word preceded by a
    // quantity would be a card.
    if quantity.is_none() && SECTION_HEADERS.contains(&name.to_lowercase().as_str()) {
        return None;
    }

    match quantity {
        // An explicit zero means the card is not part of the list.
        Some(0) => None,
        quantity => Some(Card {
            name,
            quantity: quantity.unwrap_or(1),
        }),
    }
}

/// Splits the `4` or `4x` a line may start with from the card name.
fn split_quantity(line: &str) -> (Option<u32>, &str) {
    let digits: String = line.chars().take_while(char::is_ascii_digit).collect();
    if digits.is_empty() {
        return (None, line);
    }

    let Ok(quantity) = digits.parse::<u32>() else {
        return (None, line);
    };

    let rest = &line[digits.len()..];
    let rest = rest.strip_prefix(['x', 'X']).unwrap_or(rest);

    // Without the separating space the digits belong to the card name.
    if rest.starts_with(char::is_whitespace) {
        (Some(quantity), rest.trim_start())
    } else {
        (None, line)
    }
}

/// Drops the set, collector number and foil markers exporters append.
fn clean_name(entry: &str) -> String {
    let mut name = entry.trim();

    for marker in [" (", " [", " *"] {
        if let Some(position) = name.find(marker) {
            name = name[..position].trim_end();
        }
    }

    collapse_whitespace(name)
}

/// Comparison key of a card name: lowercase, without the punctuation that
/// differs between exporters.
fn normalize(name: &str) -> String {
    let stripped: String = name
        .chars()
        .filter(|&character| !matches!(character, '\'' | '\u{2019}' | ',' | '.'))
        .flat_map(char::to_lowercase)
        .collect();

    collapse_whitespace(&stripped)
}

fn collapse_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Header of a ManaBox collection export.
    const MANABOX_HEADER: &str = "Name,Set code,Set name,Collector number,Foil,Rarity,Quantity,\
         ManaBox ID,Scryfall ID,Purchase price,Misprint,Altered,Condition,Language,\
         Purchase price currency";

    fn entries(list: &CardList) -> Vec<(&str, u32)> {
        list.cards
            .iter()
            .map(|card| (card.name.as_str(), card.quantity))
            .collect()
    }

    #[test]
    fn reads_the_usual_export_formats() {
        let list = parse_text(
            "# my deck\n\
             1 Sol Ring\n\
             4x Lightning Bolt (M10) 146\n\
             2 Arcane Signet *F*\n\
             Command Tower\n",
        );

        assert_eq!(
            entries(&list),
            [
                ("Sol Ring", 1),
                ("Lightning Bolt", 4),
                ("Arcane Signet", 2),
                ("Command Tower", 1),
            ]
        );
    }

    #[test]
    fn skips_blank_lines_comments_and_section_headers() {
        let list = parse_text("\nDeck\n// note\nSideboard\n\n2 Duress\n");

        assert_eq!(list.len(), 1);
        assert_eq!(list.cards[0].name, "Duress");
    }

    #[test]
    fn merges_the_lines_naming_the_same_card() {
        let list = parse_text("2 sol ring\n1 Sol Ring\n1 SOL RING (LEA) 1\n");

        assert_eq!(list.len(), 1);
        assert_eq!(list.copies(), 4);
        // The first spelling seen is the one kept for display.
        assert_eq!(list.cards[0].name, "sol ring");
    }

    #[test]
    fn matches_names_across_punctuation_and_casing() {
        let owned = parse_text("1 Urza's Saga\n").quantities();

        assert!(owned.contains_key(&parse_text("1 urzas saga").cards[0].key()));
    }

    #[test]
    fn keeps_leading_digits_that_are_not_followed_by_a_space() {
        let list = parse_text("Chandra, Torch of Defiance\n7th Edition Forest\n");

        assert_eq!(list.cards[0].name, "Chandra, Torch of Defiance");
        assert_eq!(list.cards[0].quantity, 1);
        assert_eq!(list.cards[1].name, "7th Edition Forest");
        assert_eq!(list.cards[1].quantity, 1);
    }

    #[test]
    fn drops_entries_with_an_explicit_zero() {
        assert!(parse_text("0 Sol Ring\n").is_empty());
    }

    #[test]
    fn reads_a_manabox_export() {
        let content = format!(
            "{MANABOX_HEADER}\n\
             \"Chandra, Torch of Defiance\",KLD,Kaladesh,110,normal,mythic,1,1234,abc,5.20,\
             false,false,near_mint,en,EUR\n\
             Sol Ring,C21,Commander 2021,263,normal,uncommon,2,5678,def,1.50,\
             false,false,near_mint,fr,EUR\n"
        );

        let list = parse("collection.csv", &content).unwrap();

        assert_eq!(
            entries(&list),
            [("Chandra, Torch of Defiance", 1), ("Sol Ring", 2)]
        );
    }

    #[test]
    fn adds_up_the_foil_and_regular_rows_of_a_card() {
        let content = format!(
            "{MANABOX_HEADER}\n\
             Sol Ring,C21,Commander 2021,263,normal,uncommon,2,1,a,1.50,false,false,near_mint,en,EUR\n\
             Sol Ring,C21,Commander 2021,263,foil,uncommon,1,2,b,4.00,false,false,near_mint,en,EUR\n"
        );

        let list = parse("collection.csv", &content).unwrap();

        assert_eq!(entries(&list), [("Sol Ring", 3)]);
    }

    #[test]
    fn matches_a_manabox_name_against_a_deck_list_name() {
        let content = format!(
            "{MANABOX_HEADER}\n\
             \"Urza's Saga\",MH2,Modern Horizons 2,259,normal,rare,1,1,a,30.0,\
             false,false,near_mint,en,EUR\n"
        );

        let owned = parse("collection.csv", &content).unwrap().quantities();

        assert!(owned.contains_key(&parse_text("1 Urza's Saga (MH2) 259").cards[0].key()));
    }

    #[test]
    fn reads_a_csv_holding_only_the_two_useful_columns() {
        let list = parse("export.csv", "Quantity,Name\n3,Duress\n").unwrap();

        assert_eq!(entries(&list), [("Duress", 3)]);
    }

    #[test]
    fn refuses_a_csv_whose_header_names_no_card_column() {
        let error = parse("collection.csv", "Foo,Bar\n1,2\n").unwrap_err();

        assert_eq!(error, ListError::UnknownColumns);
    }

    #[test]
    fn reads_a_deck_list_whose_names_hold_commas_as_text() {
        // Two fields once split on commas, but neither names a column.
        let list = parse("deck.txt", "1 Chandra, Torch of Defiance\n").unwrap();

        assert_eq!(entries(&list), [("Chandra, Torch of Defiance", 1)]);
    }

    #[test]
    fn reports_a_file_holding_no_card() {
        let error = parse("deck.txt", "\n# nothing\n").unwrap_err();

        assert_eq!(error, ListError::Empty);
    }
}
