use std::{env, error::Error};
use trovo::ClientId;
use serde::{Serialize, Deserialize};
use amiquip::{Connection, ExchangeDeclareOptions, ExchangeType, Publish};
use futures_util::StreamExt;

#[derive(Serialize, Deserialize)]
struct Message {
    message: String,
    raw_message: String,
    username: String,
    user_color_r: String,
    user_color_g: String,
    user_color_b: String,
    from: String, // ID of which program generated this message
    source_badge_large: String,
    source_badge_small: String,
    user_badges: Vec<String>
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut connection = Connection::insecure_open("amqp://guest:guest@localhost:5672").unwrap();
    let channel = connection.open_channel(None).unwrap();
    let exchange = channel.exchange_declare(
        ExchangeType::Topic,
        "chat",
        ExchangeDeclareOptions{
            durable: true,
            ..ExchangeDeclareOptions::default()
        },
    ).unwrap();

    let routing_key = "twitch".to_string();

    let client_id = env::var("CLIENT_ID").expect("missing CLIENT_ID env var");
    let username = env::var("USER_NAME").expect("missing USER_NAME env var");

    let client = trovo::Client::new(ClientId::new(client_id));

    println!("looking up user '{}'", username);
    let user = client
        .user(username)
        .await?
        .expect("no user found for the given username");
    println!("found user {:#?}", user);

    let mut messages = client.chat_messages_for_channel(&user.channel_id).await?;
    println!("listening for chat messages");
    while let Some(msg) = messages.next().await {
        let msg = msg?;
        println!("[{}] {}", msg.nick_name, msg.content);
        let message = Message {
            from: "Trovo".to_string(),
            source_badge_large: "https://static.twitchcdn.net/assets/favicon-32-e29e246c157142c94346.png".to_string(),
            source_badge_small: "https://static.twitchcdn.net/assets/favicon-16-52e571ffea063af7a7f4.png".to_string(),
            message: msg.content.clone(),
            raw_message: msg.content.clone(),
            username: msg.nick_name,
            user_color_r: "FF".to_string(),
            user_color_g: "FF".to_string(), 
            user_color_b: "FF".to_string(),
            user_badges: ["".to_string()].to_vec()
        };

        let message_json = serde_json::to_string(&message).unwrap();

        println!("{}", message_json);
        exchange.publish(Publish::new(message_json.as_bytes(), routing_key.clone())).unwrap();
    }

    Ok(())
}