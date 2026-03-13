mod game_management;
use game_management::*;

use serenity::{model::channel::Message, prelude::Context};

pub async fn handle_command(ctx: &Context, msg: &Message) -> Option<String> {
    let command: Vec<&str> = msg.content.split_whitespace().collect();
    match command[0] {
        "!help" => Some(help()),
        "!listGames" => Some(list_games().await),
        "!addGame" => Some(add_game(&msg.author.display_name(), command[1]).await),
        "!myGames" => Some(my_games(&msg.author.display_name()).await),
        "!register" => Some(register(&msg.author.display_name()).await),
        _ => Some(String::from(
            "Commande inconnu ou indisponible dans ce salon. Tapez !help pour voir les commandes disponibles",
        )),
    }
}

fn help() -> String {
    String::from(
        "Voici les commandes disponibles :\r\n\
        !help : Affiche ce message d'aide\r\n\
        !register : Vous enregistre en tant que joueur archipelago !!! A UTILISER AVANT LES AUTRES COMMANDES !!!\r\n\
        !listGames : Donne un lien avec les jeux compatible archipelago\r\n\
        !addGame <game_url> : Ajoute un jeu à votre liste de jeu compatible\r\n\
        !myGames : Affiche votre liste des jeux archipelago enregistré avec !addGame",
    )
}
