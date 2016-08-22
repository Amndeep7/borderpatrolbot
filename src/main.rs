#![feature(type_ascription)]

extern crate discord;
extern crate serde;
extern crate serde_yaml;

use std::collections::BTreeMap;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use discord::{Discord, ChannelRef, State, Result};
use discord::model::{Event, ChannelType, PossibleServer, LiveServer, Channel, RoleId};

use serde_yaml::value::{Value, to_value};

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
        if &channel.name == MY_CHANNEL_NAME && channel.kind == ChannelType::Text {
            return Ok(Channel::Public(channel));
        }
    }

    discord.create_channel(&server.id, MY_CHANNEL_NAME, ChannelType::Text)
}

fn main() {
    let mut configuration = String::new();
    let mut f = File::open("yaml_example").expect("Unable to open yaml file");
    f.read_to_string(&mut configuration).expect("Unable to read yaml file");
    println!("1");
    let raw_config: BTreeMap<String, String> = serde_yaml::from_str(&configuration).unwrap();
    println!("2");
    let mut config: BTreeMap<String, Value> = BTreeMap::new();
    println!("3");
    for (key, value) in raw_config.into_iter() {
        config.insert(key, to_value(&value));
    }
    println!("4");
    //let config: BTreeMap<String, Value> = config.into_iter().map(|(k, v)| (k, to_value(v))).collect();
    println!("config: {:?}", config);
    let extracted = config.get("visaholder").unwrap().as_str().unwrap().to_string();
    let convert = |role: &String| role[3..role.len()-1].parse::<u64>().unwrap();
    let converted = convert(&extracted);
    println!("converted: {:?}", converted);

    panic!("Hello");

    let token = read_token_file("token");
    println!("{}", token);
    let discord = Discord::from_bot_token(&token).expect("login failed");
    let (mut connection, ready) = discord.connect().expect("connect failed");
    let mut state = State::new(ready);
    println!("Ready.");

    let mut my_channels = HashMap::new();

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
                        let _ = match identify_or_create_my_channel(&discord, liveserver) {
                            Ok(Channel::Public(c)) => my_channels.insert(c.id, c),
                            _ => continue 'forever,
                        };
                        println!("{:#?}", my_channels);
                    }
                    _ => {
                        println!("Not a live server");
                    }
                }
            }
            Event::MessageCreate(message) => {
                println!("{} says: {}", message.author.name, message.content);
                let split: Vec<_> = message.content.split(char::is_whitespace).collect();
                println!("{:?}", split);
                let user_mentions = &message.mentions;
                let role_mentions = &message.mention_roles;
                println!("Mentions {:?} {:?}", user_mentions, role_mentions);
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
