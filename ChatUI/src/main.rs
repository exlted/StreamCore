use std::{collections::HashMap, convert::Infallible, sync::Arc};
use tokio::sync::{mpsc, Mutex};
use warp::{Filter, Rejection};
mod handlers;
mod ws;
use std::env;
use warp::ws::Message;
use streamcore_message_client::client::Client as Message_Client;
use streamcore_message_client::{
    client::{BasicConnectionCallback, BasicChannelCallback},
    AsyncConsumer,Channel, BasicProperties, Deliver
};
use async_trait::async_trait;


#[derive(Debug, Clone)]
pub struct Client {
    pub client_id: String,
    pub sender: Option<mpsc::UnboundedSender<std::result::Result<Message, warp::Error>>>,
}

type ClientMap = Arc<Mutex<HashMap<String, Client>>>;
type Result<T> = std::result::Result<T, Rejection>;

fn with_clients(clients: Arc<Mutex<HashMap<String, Client>>>) -> impl Filter<Extract = (Arc<Mutex<HashMap<String, Client>>>,), Error = Infallible> + Clone {
    warp::any().map(move || clients.clone())
}

async fn webserver_loop(clients: ClientMap) {

    println!("Configuring websocket route");
    let ws_route = warp::path("ws")
        .and(warp::ws())
        .and(with_clients(clients.clone()))
        .and_then(handlers::ws_handler);

    println!("Starting server");
    let routes = ws_route.with(warp::cors().allow_any_origin());
    let public = warp::fs::dir("./public/");
    let server = warp::serve(routes.or(public)).run(([0, 0, 0, 0], 8080));
    
    server.await;
}

struct Consumer{
    clients: ClientMap
}

#[async_trait]
impl AsyncConsumer for Consumer {
    async fn consume(&mut self, _channel: & Channel, _deliver: Deliver, _basic_properties: BasicProperties, content: Vec<u8>) {

        let body = std::str::from_utf8(&content).expect("Couldn't stringify body");

        ws::send_to_clients(body, &self.clients).await;
    }
}

#[tokio::main]
async fn main() {

    let clients: ClientMap = Arc::new(Mutex::new(HashMap::new()));

    let message_client = Message_Client::new(
        env::var("AMPQ_HOST").unwrap_or("localhost".to_string()),
        env::var("AMPQ_PORT").unwrap_or("5672".to_string()),
        env::var("AMPQ_USERNAME").unwrap_or("guest".to_string()),
        env::var("AMPQ_PASSWORD").unwrap_or("guest".to_string()),
        env::var("EXCHANGE_NAME").unwrap_or("chat".to_string()),
        "#".to_string()
    ).await;

    message_client.lock().await.open_client(BasicConnectionCallback, BasicChannelCallback).await;
    message_client.lock().await.attach_consumer("Name".to_string(), Consumer{clients: clients.clone()}).await;
    
    let webserver_clients = clients.clone();
    webserver_loop(webserver_clients).await;
}
