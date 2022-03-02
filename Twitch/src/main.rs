use twitch_irc::login::StaticLoginCredentials;
use twitch_irc::TwitchIRCClient;
use twitch_irc::{ClientConfig, SecureTCPTransport};
use twitch_irc::message::ServerMessage;

#[tokio::main]
pub async fn main() {
    // default configuration is to join chat as anonymous.
    let config = ClientConfig::default();
    let (mut incoming_messages, client) =
        TwitchIRCClient::<SecureTCPTransport, StaticLoginCredentials>::new(config);

    // first thing you should do: start consuming incoming messages,
    // otherwise they will back up.
    let join_handle = tokio::spawn(async move {
        while let Some(message) = incoming_messages.recv().await {     match message {
            ServerMessage::Privmsg(msg) => {
                let mut text = msg.message_text;
                for emote in &msg.emotes {
                    let url = format!("<img href='https://static-cdn.jtvnw.net/emoticons/v1/{}/3.0'>", emote.id);
                    text = text.replace(&emote.code, &url);
                }
                //for badge in &msg.badges {
                //    println!("name:{} version:{}", badge.name, badge.version);
                //}
                println!("(#{}) {}: {}", msg.channel_login, msg.sender.name, text);
            },
            //ServerMessage::Whisper(msg) => {
            //    println!("(w) {}: {}", msg.sender.name, msg.message_text);
            //},
            _ => {}
        }
        }
    });

    // Badge Notes (Phase 2 - Work on later)
    //   Request data from "https://badges.twitch.tv/v1/badges/global/display" to get all global badges
    //   For each channel we join, query for that channel's custom badges (Need to get channel ID) https://badges.twitch.tv/v1/badges/channels/<ChannelID>/display
    //     If we allow for multiple simultaneous channel joins channel custom badges need to be stored _with_ the channel name associated as they are not unique

    // join a channel
    client.join("exlted".to_owned());

    // keep the tokio executor alive.
    // If you return instead of waiting the background task will exit.
    join_handle.await.unwrap();
}