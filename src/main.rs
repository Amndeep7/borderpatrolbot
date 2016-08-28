#![feature(custom_derive, plugin, type_ascription)]
#![plugin(serde_macros)]

extern crate discord;
extern crate regex;
extern crate serde;
extern crate serde_yaml;

use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use discord::{Discord, ChannelRef, State, Result};
use discord::model::{Event, ChannelType, PossibleServer, LiveServer, Channel, RoleId,
                     PublicChannel};

use regex::Regex;

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

#[derive(Debug, Deserialize)]
struct RawConfig {
    version: u32,
    visaholders: String,
}

#[derive(Debug)]
struct Config {
    version: u32,
    visaholders: RoleId,
}

fn convert(raw: Option<RawConfig>,
           roles: Vec<RoleId>)
           -> std::result::Result<Config, &'static str> {
    if raw.is_none() {
        return Err("Couldn't parse rawconfig");
    }

    let raw = raw.unwrap();

    let mut vh = None;

    let convert = |role: &String| {
        let re = Regex::new(r"<@&(\d*)>").unwrap();
        // should only have one capture
        let id: u64 = re.captures_iter(role).next().unwrap().at(1).unwrap().parse::<u64>().unwrap();
        id
    };

    for roleid in roles {
        let RoleId(id) = roleid;
        if id == convert(&raw.visaholders) {
            vh = Some(roleid);
        }
    }

    match vh {
        Some(roleid) => {
            Ok(Config {
                version: raw.version,
                visaholders: roleid,
            })
        }
        None => Err("Couldn't match roleids"),
    }
}

#[derive(Debug)]
struct ChannelInfo {
    channel: PublicChannel,
    config: Option<Config>,
}

fn main() {
    let token = read_token_file("token");
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
                        let _ = match identify_or_create_my_channel(&discord, liveserver) {
                            Ok(Channel::Public(c)) => {
                                match discord.get_pinned_messages(c.id) {
                                    Ok(messages) => {
                                        println!("Size of messages: {}", messages.len());
                                        if messages.len() >= 1 {
                                            let config_msg = messages[0].clone();
                                            let raw_config: Option<RawConfig> =
                                                serde_yaml::from_str(config_msg.content.as_str())
                                                    .ok();
                                            let config = convert(raw_config,
                                                                 config_msg.mention_roles);
                                            my_channels.insert(c.id,
                                                               ChannelInfo {
                                                                   channel: c,
                                                                   config: config.ok(),
                                                               });
                                        } else {
                                            println!("Config message needs to be the first \
                                                      pinned message");
                                            // todo: make a message on the channel
                                            my_channels.insert(c.id,
                                                               ChannelInfo {
                                                                   channel: c,
                                                                   config: None,
                                                               });
                                        }
                                    }
                                    _ => {
                                        println!("Something fucked up while getting messages");
                                        // todo: make a message on the channel
                                    }
                                };
                            }
                            _ => continue 'forever,
                        };
                        println!("my channels: {:#?}", my_channels);
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
