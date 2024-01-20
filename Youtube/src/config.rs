use std::sync::{Arc, Mutex};
use std::ops::Deref;
use serde::{Serialize, Deserialize};
use warp::Filter;
use std::collections::HashMap;
use tokio::sync::mpsc::{Receiver, channel, Sender};

pub type Config = Arc<Mutex<HashMap<String, String>>>;
#[derive(Debug, Deserialize, Serialize, Clone)]
struct ConfigItem {
    key: String,
    val: String
}

fn json_body() -> impl Filter<Extract = (ConfigItem,), Error = warp::Rejection> + Clone {
    return warp::body::content_length_limit(1024 * 16).and(warp::body::json());
}

fn remove_body() -> impl Filter<Extract = (String,), Error = warp::Rejection> + Clone {
    return warp::body::content_length_limit(1024 * 16).and(warp::body::json());
}

async fn add_config(item: ConfigItem, config: Config, tx: Sender<()>) -> Result<impl warp::Reply, warp::Rejection> {
    {
        let mut locked_config = config.lock().unwrap();
        locked_config.insert(item.key, item.val);
    }
    tx.send(()).await.expect("Failed to send");
    let locked_config = config.lock().unwrap();
    Ok(warp::reply::json(locked_config.deref()))
}

async fn remove_config(key: String, config: Config) -> Result<impl warp::Reply, warp::Rejection> {
    let mut locked_config = config.lock().unwrap();
    locked_config.remove(&*key);
    Ok(warp::reply::json(locked_config.deref()))
}

async fn get_config(config: Config) -> Result<impl warp::Reply, warp::Rejection> {
    let locked_config = config.lock().unwrap();
    Ok(warp::reply::json(locked_config.deref()))
}

pub fn initialize_config() -> (Config, Receiver<()>, impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone) {
    let config = Arc::new(Mutex::new(HashMap::new()));
    let return_config = config.clone();
    let config_filter = warp::any().map(move || config.clone());

    let (tx, rx) = channel(1);
    let tx_filter = warp::any().map(move || tx.clone());

    let add_config = warp::post()
        .and(warp::path("v1"))
        .and(warp::path("config"))
        .and(warp::path::end())
        .and(json_body())
        .and(config_filter.clone())
        .and(tx_filter.clone())
        .and_then(add_config);

    let get_config = warp::get()
        .and(warp::path("v1"))
        .and(warp::path("config"))
        .and(warp::path::end())
        .and(config_filter.clone())
        .and_then(get_config);

    let remove_config = warp::get()
        .and(warp::path("v1"))
        .and(warp::path("config"))
        .and(warp::path::end())
        .and(remove_body())
        .and(config_filter.clone())
        .and_then(remove_config);

    return (return_config, rx, add_config.or(get_config).or(remove_config));
}