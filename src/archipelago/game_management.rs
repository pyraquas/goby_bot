use serde::{self, Deserialize, Serialize};
use serde_json::from_str;
use std::fs::{File, read_to_string};

#[derive(Debug, Serialize, Deserialize)]
struct Player {
    name: String,
    games: Vec<String>,
}

impl Player {
    fn new(name: &str) -> Self {
        Player {
            name: name.to_string(),
            games: Vec::new(),
        }
    }

    /// Load player data from json file. If file doesn't exist, create a new player with empty game list
    fn load(name: &str) -> Self {
        let player_file_str = format!("src/archipelago/players/{}.json", name);
        let player_file = read_to_string(&player_file_str).unwrap();
        from_str(&player_file).unwrap()
    }

    /// Save player data to json file
    fn save(&self) {
        let player_file_str = format!("src/archipelago/players/{}.json", self.name);
        std::fs::write(player_file_str, serde_json::to_string(self).unwrap()).unwrap();
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Players {
    players: Vec<Player>,
}

pub async fn add_game(player_name: &str, game_url: &str) -> String {
    //Check if game url is valid
    if !game_url.starts_with("https://archipelago.miraheze.org/wiki/") {
        return String::from("Url invalide : https://archipelago.miraheze.org/wiki/[nom_du_jeu]");
    }

    //Extract game name from url
    let game_name = game_url.replace("https://archipelago.miraheze.org/wiki/", "");

    //Retrieve compatible archipelago game
    let game_list: Vec<String> = read_to_string("src/archipelago/archipelago_games.txt")
        .unwrap()
        .lines()
        .map(String::from)
        .collect();

    // Case where game is in compatible game list
    if game_list.contains(&game_name) {
        let mut player = Player::load(player_name);

        //Check if game is already in player game list
        if player.games.contains(&game_name) {
            return format!(
                "{} est déjà dans la liste de {} !",
                game_name.replace("_", " "),
                player_name
            );
        }

        //Add game to player game list
        player.games.push(game_name.clone());

        //Update player json file
        player.save();

        return format!(
            "{} ajouté à la liste de {} !",
            game_name.replace("_", " "),
            player_name
        );
    }

    //If game is not in compatible game list, return error message
    String::from("Jeu non trouvé dans la liste des jeux compatibles")
}

pub async fn my_games(player_name: &str) -> String {
    //Open Json file for player and deserialize it
    let player = Player::load(player_name);

    //Check if player has any game in his list
    if player.games.is_empty() {
        //No game registered
        return format!(
            "{} n'a aucun jeu archipelago d'inscrit. Utilisez !add_game pour ajouter un jeu.",
            player_name
        );
    } else {
        //List of game registered for player
        let game_list_string = player
            .games
            .iter()
            .map(|game| game.replace("_", " "))
            .collect::<Vec<String>>()
            .join("\r\n ");
        return format!("Liste de jeu de {} :\r\n{}", player_name, game_list_string);
    }
}

pub async fn register(player_name: &str) -> String {
    if File::open(format!("src/archipelago/players/{}.json", player_name)).is_ok() {
        return format!("{} est déjà inscrit !", player_name);
    }
    let player = Player::new(player_name);
    player.save();
    format!("{} a rejoins la course !", player_name)
}

pub async fn list_games() -> String {
    String::from(
        "La listes des jeux compatibles sont visibles ici : https://archipelago.miraheze.org/wiki/Category:Games",
    )
}
