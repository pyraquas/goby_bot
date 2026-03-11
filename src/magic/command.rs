use serenity::{all::User, model::channel::Message, prelude::Context};

pub async fn handle_command(ctx: &Context, msg: &Message) -> Option<String> {
    let command_args: Vec<&str> = msg.content.split_whitespace().collect();
    match command_args[0] {
        "!help" => Some(String::from("Commande disponible dans ce salon: !help")),
        "!win" => {
            let answer;
            if msg.mentions.len() == 1 && command_args.len() == 2 {
                answer =
                    String::from("Ajout d'une victoire pour ") + &msg.mentions[0].display_name();
            } else {
                answer = String::from("Usage: !win [@user]");
            }
            Some(answer)
        }
        "!leaderboard" => Some(String::from("Affichage du leaderboard")),

        _ => Some(String::from(
            "Commande inconnu ou indisponible dans ce salon",
        )),
    }
}

fn win(user: &User) -> String {
    String::from("Ajout d'une victoire pour ") + &user.name.to_owned()
}
