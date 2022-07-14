use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Emote {
    pub url: String,
    pub name: String
}

#[derive(Serialize, Deserialize)]
pub struct Message {
    pub message: String,
    pub raw_message: String,
    pub username: String,
    pub user_color_r: String,
    pub user_color_g: String,
    pub user_color_b: String,
    pub from: String, // ID of which program generated this message
    pub source_badge_large: String,
    pub source_badge_small: String,
    pub user_badges: Vec<String>,
    pub message_emotes: Vec<Emote>
}