# Command Reference

Bot: `Goby` \
Prefix: `/` \
Last updated: YYYY-MM-DD

## Prerequisite
Commands listed in this file need a "Explorateurs des îles" role to be used and only usable in "Archipelago" guild category.

## List of archipelago related commands

## Game management commands

### `/start [run_name] [number_of_player]`

Start a new archipelago run. It also create a temporary thread for the run.

| Argument           | Type   | Required | Description                       |
| ------------------ | ------ | -------- | --------------------------------- |
| `run_name`         | string | Yes      | New archipelago run name to start |
| `number_of_player` | int    | Yes      | Number of player expected         |

**Example:** `/start "hell run" 4`

### `/close [run_name]`

Stop an existing archipelago run. This command is not reversible.

| Argument   | Type   | Required | Description                           |
| ---------- | ------ | -------- | ------------------------------------- |
| `run_name` | string | Yes      | Existing archipelago run name to stop |


## Player commands

### `/join [run_name]`
| Argument   | Type   | Required | Description                           |
| ---------- | ------ | -------- | ------------------------------------- |
| `run_name` | string | No       | Existing archipelago run name to join |

**Example:** `/join "hell run"`\
**Note:** If no session name is given, player will join latest created session

### `/leave [run_name]`
| Argument   | Type   | Required | Description                   |
| ---------- | ------ | -------- | ----------------------------- |
| `run_name` | string | Yes      | Archipelago run name to leave |

**Example:** `/leave "hell run"`\
**Note:** If archipelago run is not started, player will free reserved spot but if run is ongoing, it will free items located in his game.
