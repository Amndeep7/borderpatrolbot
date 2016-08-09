extern crate discord;

use std::fs::File;
use std::io::Read;

use discord::{Discord, ChannelRef, State, Result, Error};
use discord::model::{Event, ChannelType, PossibleServer, LiveServer, Channel, PublicChannel};

static MY_CHANNEL_NAME: &'static str = "borderpatrolbot";

fn read_token_file(name: &str) -> String {
    let mut token = String::new();
    let mut f = File::open(&name).expect("Unable to open the token file");
    f.read_to_string(&mut token).expect("Unable to read the token file");
    token = token.trim().to_string();
    token
}

fn identify_or_create_my_channel(discord: &Discord, server: LiveServer) -> Result<Channel> {
    for channel in server.channels.into_iter() {
        if &channel.name == MY_CHANNEL_NAME && &channel.kind == ChannelType::Text {
            return Ok(Channel::Public(channel));
        }
    }

    discord.create_channel(&server.id, MY_CHANNEL_NAME, ChannelType::Text)
}

fn main() {
    let token = read_token_file("token");
    println!("{}", token);
    let discord = Discord::from_bot_token(&token).expect("login failed");
    let (mut connection, ready) = discord.connect().expect("connect failed");
    let mut state = State::new(ready);
    println!("Ready.");

    'forever: loop {
        let event = match connection.recv_event() {
            Ok(event) => event,
            Err(discord::Error::Closed(code, body)) => {
                println!("[Error] Connection closed with status {:?}: {}",
                         code,
                         String::from_utf8_lossy(&body));
                break 'forever;
            }
            Err(err) => {
                println!("[Warning] Receive error: {:?}", err);
                continue 'forever;
            }
        };
        state.update(&event);

        match event {
            Event::ServerCreate(possible_server) => {
                match possible_server {
                    PossibleServer::Online(liveserver) => {
                        println!("{:#?}", liveserver);
                        let channel = identify_or_create_my_channel(&discord, liveserver)
                            .expect("My channel wasn't created");
                        match channel {
                            Ok(Channel::Public(channel)) => my_channels.push(channel),
                            _ => continue 'forever,
                        }
                        println!("{:#?}", my_channels);
                    }
                    _ => {
                        println!("Not a live server");
                    }
                }
            }
            Event::MessageCreate(message) => {
                println!("{} says: {}", message.author.name, message.content);
                let mut split: Vec<_> = message.content.split(char::is_whitespace).collect();
                println!("{:?}", split);
                match split[0] {
                    "!test" => {
                        let _ =
                            discord.send_message(&message.channel_id, "Test on split.", "", false);
                        println!("{:?} --- {:?}", message.id, message.channel_id);
                    }
                    "!visa" => {
                        println!("{:?}", message.channel_id);
                        match state.find_channel(&message.channel_id) {
                            Some(ChannelRef::Public(server, channel)) => {
                                let serverid = server.id;
                                for user in &message.mentions {
                                    let mut channel_name = "Visa_application_for_".to_string();
                                    channel_name = channel_name + &user.name;
                                    channel_name = channel_name.to_lowercase();
                                    println!("{:?} {:?} {:?}", serverid, user, channel_name);
                                    let channel = discord.create_channel(&serverid,
                                                        &channel_name,
                                                        ChannelType::Text)
                                        .expect("Should have successfully created channel");
                                    let output = format!("Started visa application process for \
                                                          {} - vouch for them here: {}",
                                                         &user.name,
                                                         &channel_name);
                                    let _ = discord.send_message(&message.channel_id, &output, "", false);
                                }
                            }
                            Some(ChannelRef::Private(channel)) => {
                                println!("Why are we here?");
                            }
                            None => {
                                println!("Something fucked up");
                            }
                        }
                    }
                    "!quit" => {
                        println!("Quitting.");
                        break 'forever;
                    }
                    _ => {}
                }
            }
            Event::Unknown(name, data) => {
                println!("[Unknown Event] {}: {:?}", name, data);
            }
            _ => {}
        }
    }
    connection.shutdown().expect("connect close failed");
}
