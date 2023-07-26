use std::collections::{BTreeMap, HashSet};
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::config::Config;
use tokio::sync::mpsc::Receiver;
use futures::StreamExt;
use serde_cbor::Value::{Map, Text, Array, Integer};
use serde_cbor::Value;
use serde_cbor::value::to_value;
use streamcore_message_client::client::{Client};
use tokio::runtime::Handle;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message as WSMessage;
use streamcore_chat_objects::{Message, Emote};

struct Emoji {
    pub alt_text: String,
    pub size_28_src: String
}

struct Node {
    pub text: Option<String>,
    pub emoji: Option<Emoji>
}

struct Chatter {
    //pub icon: String,
    pub name: String,
}

struct ChatEvent {
    pub nodes: Vec<Node>,
    pub chatter: Chatter,
    pub chatter_color: Vec<i8>,
    //pub chatter_badges: Vec<String>
}

fn deserialize_chat_color(chat_obj: &&BTreeMap<Value, Value>) -> Vec<i8> {
    let key = to_value("chatterColor").expect("Const String should work");
    if let Some(Array(chatter_color)) = chat_obj.get(&key) {
        let mut rv = Vec::new();

        for chatter_color_piece in chatter_color {
            if let Integer(val) = chatter_color_piece {
                rv.push(*val as i8);
            }
        }

        return rv;
    }
    println!("Failed to get Chat Color as the structure was not as expected");
    Vec::new()
}

fn deserialize_chatter(chat_obj: &&BTreeMap<Value, Value>) -> Chatter {
    let key = to_value("chatter").expect("Const String should work");
    if let Some(Map(chatter)) = chat_obj.get(&key) {
        let chatter_key = to_value(5).expect("Const String should work");
        if let Some(Text(name)) = chatter.get(&chatter_key) {
            return Chatter {
                name: name.to_string(),
            }
        }
    }
    println!("Failed to get Chatter as the structure was not as expected");
    return Chatter {
        name: "".to_string(),
    }
}

fn deserialize_nodes(chat_obj: &&BTreeMap<Value, Value>) -> Vec<Node> {
    let key = to_value("nodes").expect("Const String should work");
    if let Some(Array(nodes)) = chat_obj.get(&key) {
        let mut parsed_nodes = Vec::new();
        for fake_node in nodes {
            if let Map(node) = fake_node {
                let node_type_key = to_value(-4).expect("Const String should work");
                if let Some(Text(node_type)) = node.get(&node_type_key) {
                    match node_type.as_str() {
                        "TextChatNode" => {
                            let text_node_key = to_value("text").expect("");
                            if let Some(Text(name)) = node.get(&text_node_key) {
                                parsed_nodes.push(Node {
                                    text: Some(name.to_string()),
                                    emoji: None
                                });
                            }
                        }
                        "LinkChatNode" => {
                            let text_node_key = to_value("href").expect("");
                            if let Some(Text(href)) = node.get(&text_node_key) {
                                parsed_nodes.push(Node {
                                    text: Some(href.to_string()),
                                    emoji: None
                                });
                            }
                        }
                        "MentionChatNode" => {
                            let mention_node_key = to_value(5).expect("");
                            if let Some(Text(name)) = node.get(&mention_node_key) {
                                parsed_nodes.push(Node {
                                    text: Some(format!("@{}", name)),
                                    emoji: None
                                });
                            }
                        }
                        "EmojiChatNode" => {
                            let emoji_node_key = to_value("emoji").expect("");
                            if let Some(Map(emoji)) = node.get(&emoji_node_key) {
                                let alt_text_node_key = to_value("altText").expect("");
                                if let Some(Text(alt_text)) = emoji.get(&alt_text_node_key) {
                                    let src_node_key = to_value("size28Src").expect("");
                                    if let Some(Text(src)) = emoji.get(&src_node_key) {
                                        parsed_nodes.push(Node {
                                            text: None,
                                            emoji: Some(Emoji{
                                                alt_text: alt_text.to_string(),
                                                size_28_src: src.to_string(),
                                            }),
                                        })
                                    }
                                }
                            }
                        }
                        &_ => {
                            println!("Experienced unexpected Chat Node type: {}", node_type);
                            continue;
                        }
                    }
                }
            }
        }
        return parsed_nodes;
    }
    println!("Failed to get Message Contents as the structure was not as expected");
    return Vec::new();
}

// This ugly mess is because _somebody_ thought it would be a good idea to use integers as identifiers
fn deserialize_chat_event(raw_data: Value) -> Option<ChatEvent> {
    if let Map(obj) = raw_data {
        if let Some(Text(event_type_str)) = obj.get(&to_value(-4).expect("Static int should work")) {
            if event_type_str != "ChatCreatedEvent" {
                return None;
            }

            if let Some(Map(chat_obj)) = obj.get(&to_value("chat").expect("Static String should work")) {
                return Some(ChatEvent{
                    nodes: deserialize_nodes(&chat_obj),
                    chatter: deserialize_chatter(&chat_obj),
                    chatter_color: deserialize_chat_color(&chat_obj),
                });
            }
        }
    }
    return None;
}

async fn vstream_ingest(stream_id: String, message_client: Arc<Mutex<Client>>) {
    let url = format!("https://vstream.com/v/{}/chat-popout", stream_id);
    println!("{}", url);
    let response = reqwest::get(url).await;
    let text = response.expect("").text().await.unwrap();
    let start = text.find("wss").expect("Couldn't find wss URL");
    let end = text.find("\",\"channelProfile").expect("couldn't find end of wss URL");
    let ws_url = &text[start..end];
    println!("{}", ws_url);

    let (mut socket, _) = connect_async(ws_url).await.unwrap();

    loop {
        let next = socket.next().await
            .expect("Failed to get next on socket")
            .expect("next was error");
        match next {
            WSMessage::Text(str) => {
                println!("{}", str);
            }
            WSMessage::Binary(bin) => {
                let mut d = serde_cbor::Deserializer::from_reader(bin.as_slice()).into_iter::<Value>();
                while let Some(Ok(v)) = d.next() {
                    let chat_option = deserialize_chat_event(v);
                    if chat_option.is_none() {
                        continue;
                    }
                    let chat = chat_option.unwrap();
                    if chat.chatter_color.len() < 3 {
                        continue;
                    }

                    let mut text = "".to_string();
                    let mut emote_set = HashSet::new();

                    for node in chat.nodes {
                        if node.text.is_some() {
                            text.push_str(&node.text.unwrap());

                            continue;
                        }
                        if node.emoji.is_some() {
                            println!("Found a parsed emote");
                            let emoji_node = node.emoji.unwrap();
                            let emote = Emote{
                                url: emoji_node.size_28_src,
                                name: emoji_node.alt_text.clone(),
                            };
                            emote_set.insert(emote);
                            text.push_str(&format!("{}", emoji_node.alt_text).to_string());
                            continue;
                        }
                    }

                    let outgoing_message = Message{
                        message: text.clone(),
                        raw_message: text,
                        username: chat.chatter.name,
                        user_color_r: chat.chatter_color[0].to_string(),
                        user_color_g: chat.chatter_color[1].to_string(),
                        user_color_b: chat.chatter_color[2].to_string(),
                        from: "vstream".to_string(),
                        source_badge_large: "https://vstream.com/favicon_32.png".to_string(),
                        source_badge_small: "https://vstream.com/favicon_128.png".to_string(),
                        user_badges: vec![],
                        message_emotes: emote_set.into_iter().collect(),
                    };

                    let message_json = serde_json::to_string(&outgoing_message).unwrap();

                    println!("{}", message_json);

                    message_client.lock().await.publish_message(message_json).await;
                }
            }
            WSMessage::Ping(_) => {
                println!("Ping Message Received");
            }
            WSMessage::Pong(_) => {
                println!("Pong Message Received");
            }
            WSMessage::Close(_) => {
                println!("Close Message Received");
            }
            WSMessage::Frame(_) => {
                println!("Raw Frame Message Received");
            }
        }
    }

}

pub async fn start_vstream_loop(config: Config, mut rx: Receiver<()>, message_client: Arc<Mutex<Client>>, handle: Handle) {
    loop {
        rx.recv().await;
        println!("Received notification");
        let locked_config = config.lock().unwrap();
        let stream_id = locked_config.get("stream_id");
        if stream_id.is_some() {
            println!("Got Stream_ID of {}", stream_id.unwrap());
            // Everything is working up until _this_ point. vstream_ingest isn't actually starting until the next time this gets called
            handle.spawn(vstream_ingest(stream_id.unwrap().to_string(), message_client.clone()));
        }
    }
}