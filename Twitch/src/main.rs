use twitch_irc::login::StaticLoginCredentials;
use twitch_irc::TwitchIRCClient;
use twitch_irc::{ClientConfig, SecureTCPTransport};
use twitch_irc::message::{RGBColor, ServerMessage};
use tokio::{task};
use serde_json;
use std::env;
use streamcore_chat_objects::{Emote, Message};
use streamcore_message_client::client::Client as Message_Client;
use streamcore_message_client::client::{BasicConnectionCallback, BasicChannelCallback};

#[tokio::main]
pub async fn main() {
    // default configuration is to join chat as anonymous.
    let config = ClientConfig::default();
    let (mut incoming_messages, client) =
        TwitchIRCClient::<SecureTCPTransport, StaticLoginCredentials>::new(config);

    let message_client = Message_Client::new(
        env::var("AMPQ_HOST").unwrap_or("localhost".to_string()),
        env::var("AMPQ_PORT").unwrap_or("5672".to_string()),
        env::var("AMPQ_USERNAME").unwrap_or("guest".to_string()),
        env::var("AMPQ_PASSWORD").unwrap_or("guest".to_string()),
        env::var("EXCHANGE_NAME").unwrap_or("chat".to_string()),
        "twitch".to_string()
    ).await;

    message_client.lock().await.open_client(BasicConnectionCallback, BasicChannelCallback).await;


    let local = task::LocalSet::new();
    local.spawn_local( async move{
        while let Some(message) = incoming_messages.recv().await {
            match message {
                ServerMessage::Privmsg(msg) => {
                    let mut text = msg.message_text.clone();
                    let mut emotes = Vec::new();
                    for emote in &msg.emotes {
                        let url = format!("https://static-cdn.jtvnw.net/emoticons/v1/{}/3.0", emote.id);
                        emotes.push(Emote {
                            url: url.clone(),
                            name: emote.code.clone()
                        });
                        let html = format!("<img src='{}'>", url);
                        text = text.replace(&emote.code, &html);
                    }
                    let badges = Vec::new();
                    //for badge in &msg.badges {
                    //    println!("name:{} version:{}", badge.name, badge.version);
                    //}

                    let name_color = msg.name_color.unwrap_or(RGBColor{r: 144, g: 70, b: 255});

                    let message = Message {
                        from: "Twitch".to_string(),
                        source_badge_large: "https://static.twitchcdn.net/assets/favicon-32-e29e246c157142c94346.png".to_string(),
                        source_badge_small: "https://static.twitchcdn.net/assets/favicon-16-52e571ffea063af7a7f4.png".to_string(),
                        message: text,
                        raw_message: msg.message_text,
                        username: msg.sender.name,
                        // Ensure these all are 2 characters, no more, no less
                        user_color_r: format!("{:02x}", name_color.r),
                        user_color_g: format!("{:02x}", name_color.g),
                        user_color_b: format!("{:02x}", name_color.b),
                        user_badges: badges,
                        message_emotes: emotes
                    };

                    let message_json = serde_json::to_string(&message).unwrap();

                    println!("{}", message_json);

                    message_client.lock().await.publish_message(message_json).await;
                },
                //ServerMessage::Whisper(msg) => {
                //    println!("(w) {}: {}", msg.sender.name, msg.message_text);
                //},
                _ => {}
            }
        }
    });

    // - Just the freeware version -
    // Chat Inputs (Twitch, Youtube, Trovo, Facebook, Discord, etc)
    // Chat Outputs (^ Same)
    // Chat Window (Displayed on Stream, needs to be highly customizable)
    // Moderation Window (Not displayed on stream, allows for per-chat moderation commands easily)
    // Chat Bot system (Needs to be configurable)

    // Alert Inputs (Twitch, Youtube, Trovo, Facebook, StreamLabs, StreamElements, Paypal, Discord, etc)
    // Alert Window (Displayed on Stream, needs to be highly customizable)
    // Tops/Recents Display (Displayed on Stream, needs to be configurable)
    // Goal Display (Displayed on Stream, needs to be configurable)
    // Alert DB Layer (Keep track of alert history, allows us to track "tops" and "recents")
    // Moderation Window (Displays alert history, Displays "tops" and "recents", configure goals)

    // - Paid Version -
    // UI Around configuration of _everything_ that is configurable
    //   Chat connections, Bot connections, Chat Window, Alert Window, etc
    // Automating the process of starting up servers/running programs, configuring them

    // Badge Notes (Phase 2 - Work on later)
    //   Request data from "https://badges.twitch.tv/v1/badges/global/display" to get all global badges
    //   For each channel we join, query for that channel's custom badges (Need to get channel ID) https://badges.twitch.tv/v1/badges/channels/<ChannelID>/display
    //     If we allow for multiple simultaneous channel joins channel custom badges need to be stored _with_ the channel name associated as they are not unique

    // join a channel
    let twitch_account = env::var("CHANNEL_USERNAME").unwrap();
    client.join(twitch_account).unwrap();

    // keep the tokio executor alive.
    // If you return instead of waiting the background task will exit.
    local.await;
}
