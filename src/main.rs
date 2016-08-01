extern crate discord;

use std::fs::File;
use std::io::Read;

use discord::Discord;
use discord::model::Event;

fn read_token_file(name: &str) -> String {
    let mut token = String::new();
    let mut f = File::open(&name).expect("Unable to open the token file");
    f.read_to_string(&mut token).expect("Unable to read the token file");
    token = token.trim().to_string();
    token
}

fn main() {
    let token = read_token_file("token");
    println!("{}", token);
    let discord = Discord::from_bot_token(&token).expect("login failed");
    let (mut connection, _) = discord.connect().expect("connect failed");
    println!("Ready.");
    loop {
        match connection.recv_event() {
            Ok(Event::MessageCreate(message)) => {
                println!("{} says: {}", message.author.name, message.content);
                if message.content == "!test" {
                    let _ = discord.send_message(&message.channel_id,
                                                 "This is a reply to the test.",
                                                 "",
                                                 false);
                } else if message.content == "!quit" {
                    println!("Quitting.");
                    break;
                }
            }
            Ok(_) => {}
            Err(discord::Error::Closed(code, body)) => {
                println!("Gateway closed on us with code {:?}: {}",
                         code,
                         String::from_utf8_lossy(&body));
                break;
            }
            Err(err) => println!("Receive error: {:?}", err),
        }
    }
    connection.shutdown().expect("connect close failed");
}
