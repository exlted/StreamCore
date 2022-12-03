use std::{collections::HashMap, convert::Infallible, sync::Arc, time::Duration, thread};
use tokio::sync::{mpsc, Mutex};
use tokio::task;
use warp::{Filter, Rejection};
mod handlers;
mod ws;
use std::env;
use amiquip::{Connection, ExchangeDeclareOptions, ExchangeType, QueueDeclareOptions, FieldTable,
              ConsumerOptions, ConsumerMessage};
use warp::ws::Message;

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

#[tokio::main]
async fn main() {

    let clients: ClientMap = Arc::new(Mutex::new(HashMap::new()));

    let ampq_clients = clients.clone();
    let local = task::LocalSet::new();
    local.spawn_local(async move {
        let host = env::var("AMPQ_HOST").unwrap_or("localhost".to_string());
        let port = env::var("AMPQ_PORT").unwrap_or("5672".to_string());
        let username = env::var("AMPQ_USERNAME").unwrap_or("guest".to_string());
        let password = env::var("AMPQ_PASSWORD").unwrap_or("guest".to_string());
        let exchange = env::var("EXCHANGE_NAME").unwrap_or("chat".to_string());

        let url = format!("amqp://{}:{}@{}:{}", username, password, host, port);

        let mut connection: Connection;
        //Wait to get a successful connection
        loop {
            let connection_attempt = Connection::insecure_open(&url);
            if connection_attempt.is_err() {
                thread::sleep(Duration::from_secs(5));
                continue;
            }
            connection = connection_attempt.unwrap();
            break;
        }
        let channel = connection.open_channel(None).unwrap();
        let exchange = channel.exchange_declare(
            ExchangeType::Topic,
            exchange,
            ExchangeDeclareOptions{
                durable: true,
                ..ExchangeDeclareOptions::default()
            },
        ).unwrap();

        let queue = channel.queue_declare(
            "",
            QueueDeclareOptions {
                exclusive: true,
                ..QueueDeclareOptions::default()
            }).expect("Failed to create Queue");
        
        queue.bind(&exchange, "#", FieldTable::new()).expect("Failed to bind Queue to Exchange");
        
        let consumer = queue.consume(ConsumerOptions {
            no_local: true,
            ..ConsumerOptions::default()
        }).expect("Failed to create Consumer");

        for (_i, message) in consumer.receiver().iter().enumerate() {
            match message {
                ConsumerMessage::Delivery(delivery) => {
                    // Pass delivery on via websockets to all members
                    let body = std::str::from_utf8(&delivery.body).expect("Couldn't stringify body");
                    ws::send_to_clients(body, &ampq_clients).await;
                },
                ConsumerMessage::ServerClosedChannel(err)
                | ConsumerMessage::ServerClosedConnection(err) => {
                    println!("{}", err.to_string());
                    continue;
                },
                 ConsumerMessage::ClientCancelled
                | ConsumerMessage::ServerCancelled
                | ConsumerMessage::ClientClosedChannel
                | ConsumerMessage::ClientClosedConnection => continue,
            }
        }
    });
    
    let webserver_clients = clients.clone();
    tokio::spawn(async move {
        webserver_loop(webserver_clients).await;
    });
    
    local.await;
}
