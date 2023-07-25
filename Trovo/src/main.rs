use chrono::Utc;
use std::{env, error::Error};
use trovo::{ClientId, EmoteFetchType};
//use trovo::chat::ChatMessageType;
use futures_util::StreamExt;
use regex::Regex;
use lazy_static::lazy_static;
use std::collections::HashMap;
use streamcore_chat_objects::{Emote, Message};
use streamcore_message_client::client::{Client, BasicConnectionCallback, BasicChannelCallback};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let message_client = Client::new(
        env::var("AMPQ_HOST").unwrap_or("localhost".to_string()),
        env::var("AMPQ_PORT").unwrap_or("5672".to_string()),
        env::var("AMPQ_USERNAME").unwrap_or("guest".to_string()),
        env::var("AMPQ_PASSWORD").unwrap_or("guest".to_string()),
        env::var("EXCHANGE_NAME").unwrap_or("chat".to_string()),
        "trovo".to_string()
    ).await;
    message_client.lock().await.open_client(BasicConnectionCallback, BasicChannelCallback).await;

    let client_id = env::var("CLIENT_ID").expect("missing CLIENT_ID env var");
    let username = env::var("CHANNEL_USERNAME").expect("missing CHANNEL_USERNAME env var");

    let client = trovo::Client::new(ClientId::new(client_id));

    println!("looking up user '{}'", username);
    let user = client
        .user(username)
        .await?
        .expect("no user found for the given username");
    println!("found user {:#?}", user);

    let emotes = client.emotes(EmoteFetchType::All, [user.channel_id.clone()].to_vec()).await?;
    let mut all_emotes = HashMap::new();
    
    for emote in emotes.global_emotes {
        all_emotes.insert(
            emote.name.clone(),
            emote
        );
    }

    for emote in emotes.event_emotes {
        all_emotes.insert(
            emote.name.clone(),
            emote
        );
    }

    for channel in emotes.customized_emotes.channel {
        for emote in channel.emotes {
            all_emotes.insert(
                emote.name.clone(),
                emote
            );
        }
    }

    let mut messages = client.chat_messages_for_channel(&user.channel_id).await?;
    println!("listening for chat messages");
    let mut timestamp = Utc::now(); 
    while let Some(msg) = messages.next().await {
        let msg = msg?;

        if msg.send_time > timestamp {
            timestamp = msg.send_time;
        } else {
            continue;
        }

        // These can always be empty right now, that's okay
        let mut text = msg.content.clone();
        let badges = Vec::new();
        let mut emotes = Vec::new();
        lazy_static! {
            static ref EMOTE_RE: Regex = Regex::new(r"(:([^\s]*)) ").unwrap();
        }
        for cap in EMOTE_RE.captures_iter(&msg.content) {
            if all_emotes.contains_key(&cap[2].to_string()) {
                let mut url = all_emotes[&cap[2].to_string()].url.clone();
                if let Some(gifp) = all_emotes[&cap[2].to_string()].gifp.as_ref() {
                    url = gifp.to_string();
                }
                else if let Some(webp) = all_emotes[&cap[2].to_string()].webp.as_ref(){
                    url = webp.to_string();
                }
                url.push_str("?imageView2");
                
                emotes.push(Emote {
                    url: url.clone(),
                    name: cap[1].to_string()
                });
                let html = format!("<img src='{}'>", url);
                text = text.replace(&cap[1].to_string(), &html);
            }

        }

        let message = Message {
            from: "Trovo".to_string(),
            source_badge_large: "https://astatic.trovocdn.net/cat/favicon.ico".to_string(),
            source_badge_small: "https://astatic.trovocdn.net/cat/favicon.ico".to_string(),
            message: text,
            raw_message: msg.content.clone(),
            username: msg.nick_name,
            user_color_r: "19".to_string(),
            user_color_g: "d6".to_string(), 
            user_color_b: "6b".to_string(),
            user_badges: badges,
            message_emotes: emotes
        };

        let message_json = serde_json::to_string(&message).unwrap();

        println!("{}", message_json);
        message_client.lock().await.publish_message(message_json).await;
    }

    Ok(())
}