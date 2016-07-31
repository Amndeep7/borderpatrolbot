extern crate discord;

use discord::Discord;

fn main() {
    println!("Hello, world!");

    let discord = Discord::from_bot_token().expect("login failed");
    discord.logout().expect("logout failed");
}
