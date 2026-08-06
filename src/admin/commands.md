# Command Reference
Bot: `Goby` \
Prefix: `/` \
Last updated: 2026-08-05

## Prerequisite
Commands listed in this file need a guild admistrator role to be used.

## Administration command list
### `/init [feature]`
Create role and guild category related to feature pass in argument.

| Argument  | Type   | Required | Description           |
| --------- | ------ | -------- | --------------------- |
| `feature` | string | yes      | Feature to initialise |

**Permissions:** Admin \
**Possible feature argument:**
- **mtg** : Create "Politicien des cartes" role and "Magic: le rassemblement" guild category
- **archi**: Create "Explorateur des îles" role and "Archipelago" guild category
- **job**: Create "Abaye des vocations" text channel.
- **all**: Init every features above

**Example:** `/init mtg`

**Note** This commands do nothing if feature already initialise.