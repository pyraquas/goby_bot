# Command Reference

Bot: `Goby` \
Prefix: `/` \
Last updated: 2026-08-05

## Prerequisite
Commands listed in this file need a "Politicien des cartes" role to be used and are only usable in "Magic: le rassemblement" guild category.

## List of Magic: the Gathering related commands

### `/need [card_list]

Display players who can trade/sell cards listed in provided list. Bot will also send a private message to thoose player with the list of card they have.

| Argument    | Type      | Required | Description          |
| ----------- | --------- | -------- | -------------------- |
| `card_list` | text file | Yes      | Requested cards list |

**Example:** `/need eldrazy_commander.txt`

### `/have [card_list]`

Register provided card list as available for trade/sell.

| Argument    | Type      | Required | Description          |
| ----------- | --------- | -------- | -------------------- |
| `card_list` | text file | Yes      | Available cards list |

**Example:** `/have bulk.txt`
**Note:** Register another card list will overwrite the first one.
