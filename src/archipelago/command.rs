use std::{fs::File, io::read_to_string};

use serenity::{model::channel::Message, prelude::Context};

pub async fn handle_command(ctx: &Context, msg: &Message) -> Option<String> {
    let command = msg.content.split_whitespace().next().unwrap();
    match command {
        "!help" => Some(help()),
        "!list_games" => Some(list_games()),
        "!add_game" => Some(add_game(msg)),
        _ => Some(String::from(
            "Commande inconnu ou indisponible dans ce salon. Tapez !help pour voir les commandes disponibles",
        )),
    }
}

fn help() -> String {
    String::from("Commande disponible dans ce salon: !help")
}

fn list_games() -> String {
    String::from(
        "La listes des jeux compatibles sont visibles ici : https://archipelago.miraheze.org/wiki/Category:Games",
    )
}

fn add_game(msg: &Message) -> String {
    let command_args: Vec<&str> = msg.content.split_whitespace().collect();
    if command_args.len() != 2 {
        return String::from(
            "Usage: !add_game [game_url], tapez !list_games pour voir les jeux compatibles \r\n
        Exemple: !add_game https://archipelago.miraheze.org/wiki/Final_Fantasy",
        );
    }

    //Extract game name from url
    if !command_args[1].starts_with("https://archipelago.miraheze.org/wiki/") {
        return String::from("Url invalide : https://archipelago.miraheze.org/wiki/[nom_du_jeu]");
    }
    let game_name = command_args[1]
        .replace("https://archipelago.miraheze.org/wiki/", "")
        .replace("_", " ");

    println!("Game name: {}", game_name);

    //Retrieve compatible games list from file
    let mut file = File::open("./src/archipelago/archipelago_data.json")
        .expect("Unable to open compatible games file");
    let compatible_games = read_to_string(&mut file).expect("Unable to read compatible games file");
    let compatible_games: serde_json::Value =
        serde_json::from_str(&compatible_games).expect("Unable to parse compatible games file");

    //Check if game is compatible
    if !compatible_games["games"]
        .as_array()
        .unwrap()
        .iter()
        .any(|game| game["name"].to_string() == "\"".to_owned() + &game_name + "\"")
    {
        return String::from("Ce jeu n'est pas compatible avec Archipelago");
    }

    format!(
        "{} a ajouté {} a sa liste de jeux possible",
        msg.author.name, game_name
    )
}
