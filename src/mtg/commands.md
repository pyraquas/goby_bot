# Command Reference

Bot: `Goby` \
Prefix: `/` \
Last updated: 2026-08-08

## Prerequisite
Commands listed in this file need a "Politicien des cartes" role to be used and are only usable in "Magic: le rassemblement" guild category.

## Card list file format

Both commands below accept two kinds of attachment, the format is detected from the file itself:

- **ManaBox export** (`.csv`): any CSV holding a `Name` column and a `Quantity` column. The other columns are ignored, and the foil and regular rows of a card are added up.
- **Deck list** (`.txt`): one card per line, with an optional `4` or `4x` quantity in front. Set and collector information (`(M10) 146`, `*F*`), `#` and `//` comments, and section headers such as `Sideboard` are ignored.

Card names are matched regardless of case, apostrophes and commas, so a ManaBox export and a deck list can be compared with each other.

## List of Magic: the Gathering related commands

### `/need [card_list]

Display players who can trade/sell cards listed in provided list. Bot will also send a private message to thoose player with the list of card they have.

| Argument    | Type              | Required | Description          |
| ----------- | ----------------- | -------- | -------------------- |
| `card_list` | text or CSV file  | Yes      | Requested cards list |

**Example:** `/need eldrazy_commander.txt`

### `/have [card_list]`

Register provided card list as available for trade/sell.

| Argument    | Type              | Required | Description          |
| ----------- | ----------------- | -------- | -------------------- |
| `card_list` | text or CSV file  | Yes      | Available cards list |

**Example:** `/have manabox_collection.csv`
**Note:** Register another card list will overwrite the first one.
