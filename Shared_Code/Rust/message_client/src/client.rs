use amqprs::{
    channel::{
        BasicPublishArguments, BasicConsumeArguments, QueueBindArguments, QueueDeclareArguments
    },
    connection::OpenConnectionArguments
};
use std::mem;
use std::ptr;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;
use amqprs::{
    callbacks::{ChannelCallback, ConnectionCallback},
    channel::Channel,
    connection::Connection,
    error::Error,
    Close, Ack, Nack, Return, Cancel, CloseChannel, BasicProperties,
    consumer::AsyncConsumer
};

use amqprs::channel::ExchangeDeclareArguments;


pub type Result<T> = std::result::Result<T, Error>;

pub struct Client {
    pub host: String,
    pub port: String,
    pub username: String,
    pub password: String,
    pub exchange_name: String,
    pub routing_key: String,
    pub(crate) queue_name: String,
    pub(crate) connection: Option<Connection>,
    pub(crate) channel: Option<Channel>,
    pub(crate) open: bool,
    pub(crate) blocked: bool,
    pub(crate) flow_blocked: bool
}
static CLIENTS: Mutex<Vec<Arc<Mutex<Client>>>> = Mutex::const_new(Vec::new());

pub struct BasicConnectionCallback;

#[async_trait]
impl ConnectionCallback for BasicConnectionCallback {

    async fn close(&mut self, connection: &Connection, _close: Close) -> Result<()> {

        let clients = CLIENTS.lock().await;
        for client_mutex in clients.iter() {
            let mut client = client_mutex.lock().await;
            if client.open && client.connection.is_some() {
                let client_connection = client.connection.as_ref().unwrap();
                if ptr::eq(client_connection, connection) {
                    client.close_client().await;
                }
            }
        }

        Ok(())
    }

    async fn blocked(&mut self, connection: &Connection, _reason: String) {
        let clients = CLIENTS.lock().await;
        for client_mutex in clients.iter() {
            let mut client = client_mutex.lock().await;
            if client.open && client.connection.is_some() {
                let client_connection = client.connection.as_ref().unwrap();
                if ptr::eq(client_connection, connection) {
                    client.blocked = true;
                }
            }
        }
    }

    async fn unblocked(&mut self, connection: &Connection) {
        let clients = CLIENTS.lock().await;
        for client_mutex in clients.iter() {
            let mut client = client_mutex.lock().await;
            if client.open && client.connection.is_some() {
                let client_connection = client.connection.as_ref().unwrap();
                if ptr::eq(client_connection, connection) {
                    client.blocked = false;
                }
            }
        }
    }
}

pub struct BasicChannelCallback;

#[async_trait]
impl ChannelCallback for BasicChannelCallback {
    async fn close(&mut self, channel: &Channel, _close: CloseChannel) -> Result<()> {
        let clients = CLIENTS.lock().await;
        for client_mutex in clients.iter() {
            let mut client = client_mutex.lock().await;
            if client.open && client.connection.is_some() {
                let client_channel = client.channel.as_ref().unwrap();
                if ptr::eq(client_channel, channel) {
                    client.close_client().await;
                }
            }
        }
        Ok(())
    }
    async fn cancel(&mut self, channel: &Channel, _cancel: Cancel) -> Result<()> {
        let clients = CLIENTS.lock().await;
        for client_mutex in clients.iter() {
            let mut client = client_mutex.lock().await;
            if client.open && client.connection.is_some() {
                let client_channel = client.channel.as_ref().unwrap();
                if ptr::eq(client_channel, channel) {
                    client.close_client().await;
                }
            }
        }
        Ok(())
    }
    async fn flow(&mut self, channel: &Channel, active: bool) -> Result<bool> {
        let clients = CLIENTS.lock().await;
        for client_mutex in clients.iter() {
            let mut client = client_mutex.lock().await;
            if client.open && client.connection.is_some() {
                let client_channel = client.channel.as_ref().unwrap();
                if ptr::eq(client_channel, channel) {
                    client.flow_blocked = active;
                }
            }
        }
        Ok(active)
    }
    async fn publish_ack(&mut self, channel: &Channel, ack: Ack) {
        println!(
            "Info: handle publish ack delivery_tag={} on channel {}",
            ack.delivery_tag(),
            channel
        );
    }
    async fn publish_nack(&mut self, channel: &Channel, nack: Nack) {
        println!(
            "Warning: handle publish nack delivery_tag={} on channel {}",
            nack.delivery_tag(),
            channel
        );
    }
    async fn publish_return(
        &mut self,
        channel: &Channel,
        ret: Return,
        _basic_properties: BasicProperties,
        content: Vec<u8>,
    ) {
        println!(
            "Warning: handle publish return {} on channel {}, content size: {}",
            ret,
            channel,
            content.len()
        );
    }
}

impl Client {
    pub async fn new(host: String, port: String, username: String, password: String, exchange_name: String, routing_key: String) -> Arc<Mutex<Client>> {
        let client = Arc::new(Mutex::new(Client{
            host,
            port,
            username,
            password,
            exchange_name,
            routing_key,
            queue_name: "".to_string(),
            connection: None,
            channel: None,
            open: false,
            blocked: false,
            flow_blocked: false,
        }));

        CLIENTS.lock().await.push(client.clone());

        return client;
    }

    pub fn is_connection_good(&self) -> bool
    {
        if !self.open || self.blocked || self.flow_blocked {
            return false;
        }
        return true;
    }

    pub async fn open_client<F, G>(&mut self, connection_callback: F, channel_callback: G) -> Option<()>
        where
            F: ConnectionCallback + Send + 'static,
            G: ChannelCallback + Send + 'static,
    {
        self.connection = Some(Connection::open(&OpenConnectionArguments::new(
            &*self.host.clone(),
            self.port.clone().parse().unwrap(),
            &*self.username.clone(),
            &*self.password.clone()
        )).await.unwrap());

        self.connection.as_ref()?
            .register_callback(connection_callback)
            .await
            .unwrap();

        self.channel = Some(self.connection.as_ref()?.open_channel(None).await.unwrap());
        self.channel.as_ref()?
            .register_callback(channel_callback)
            .await
            .unwrap();

        let mut exchange_opts = ExchangeDeclareArguments::new(&self.exchange_name.clone(), "topic");
        exchange_opts.durable = true;

        self.channel.as_ref()?
            .exchange_declare(exchange_opts).await.unwrap();

        let (queue_name, _, _) = self.channel.as_ref()?
            .queue_declare(QueueDeclareArguments::default())
            .await
            .unwrap()
            .unwrap();

        self.queue_name = queue_name;

        self.channel.as_ref()?
            .queue_bind(QueueBindArguments::new(
                &*self.queue_name.clone(),
                &*self.exchange_name.clone(),
                &*self.routing_key.clone()
            ))
            .await
            .unwrap();

        self.open = true;

        Some(())
    }

    pub async fn close_client(&mut self) -> Option<()> {
        if !self.open {
            return None
        }

        mem::replace(&mut self.channel, None)?.close().await.unwrap();
        mem::replace(&mut self.connection, None)?.close().await.unwrap();

        self.open = false;
        Some(())
    }

    pub async fn publish_message(&self, message: String) -> Option<()> {
        if !self.is_connection_good() {
            return None
        }

        let content = message.into_bytes();

        let args = BasicPublishArguments::new(&*self.exchange_name.clone(), &*self.routing_key.clone());

        self.channel.as_ref()?
            .basic_publish(BasicProperties::default(), content, args)
            .await
            .unwrap();

        Some(())
    }

    pub async fn attach_consumer<F>(&mut self, name: String, consumer: F) -> Option<()>
        where
            F: AsyncConsumer + Send + 'static
    {
        if !self.is_connection_good() {
            return None
        }

        let args = BasicConsumeArguments::new(
            &*self.queue_name.clone(),
            &*name
        );

        self.channel.as_ref()?
            .basic_consume(consumer, args)
            .await
            .unwrap();

        Some(())
    }
}