mod config;
mod vstream;

use std::env;
use streamcore_message_client::client::{Client, BasicConnectionCallback, BasicChannelCallback};
use tokio::runtime::Handle;
use crate::config::initialize_config;
use warp::Filter;
use crate::vstream::start_vstream_loop;

/*
Notes taken from: https://gist.github.com/doxy-ai/364d9804d97c8d37285e7b8671d274d4

Open Stream's chat-popout (https://vstream.com/v/{stream ID}/chat-popout)
Get Channel ID from chat-popout
Get Video ID from chat-popout
Open websocket with Channel ID & Video ID (wss://vstream.com/suika/api/room/{Channel ID}/{Video ID}/websocket)
Create Message Object from received messages
*/

#[tokio::main]
async fn main() {
    let message_client = Client::new(
        env::var("AMPQ_HOST").unwrap_or("localhost".to_string()),
        env::var("AMPQ_PORT").unwrap_or("5672".to_string()),
        env::var("AMPQ_USERNAME").unwrap_or("guest".to_string()),
        env::var("AMPQ_PASSWORD").unwrap_or("guest".to_string()),
        env::var("EXCHANGE_NAME").unwrap_or("chat".to_string()),
        "vstream".to_string()
    ).await;

    message_client.lock().await.open_client(BasicConnectionCallback, BasicChannelCallback).await;
    let (config, rx, config_routes) = initialize_config();
    {
        config.lock().unwrap().insert("stream_id".to_string(), "".to_string());
    }

    // Connect to VStream and produce output
    tokio::spawn(start_vstream_loop(config, rx, message_client, Handle::current()));

    println!("Starting server");
    let public = warp::fs::dir("./public/");

    let routes = public.or(config_routes);

    let server = warp::serve(routes).run(([0, 0, 0, 0], 8081));

    server.await;
}