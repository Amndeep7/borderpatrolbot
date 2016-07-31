extern crate discord;

use std::fs::File;
use std::io::Read;

use discord::Discord;

fn read_token_file(name: &str) -> String {
    let mut token = String::new();
    let mut f = File::open(&name).expect("Unable to open the token file");
    f.read_to_string(&mut token).expect("Unable to read the token file");
    token
}

fn main() {
    let token = read_token_file("token");
    println!("{}", token);
    let discord = Discord::from_bot_token(&token).expect("login failed");
    discord.logout().expect("logout failed");
}
