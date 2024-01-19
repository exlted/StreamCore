use chromiumoxide::{MethodType, types::MethodId, browser::{Browser, BrowserConfig}, cdp::CustomEvent};
use chromiumoxide::cdp::js_protocol::runtime::{AddBindingParams, EventBindingCalled};
use std::sync::Arc;
use futures::StreamExt;
use serde::Deserialize;
use tokio::sync::Mutex;
use streamcore_message_client::client::Client;
use crate::config::Config;
use tokio::sync::mpsc::Receiver;
use tokio::runtime::Handle;
use streamcore_chat_objects::{Emote, Message};

#[derive(Clone, Eq, PartialEq, Deserialize)]
struct MessageReceived {
    message: String,
    raw_message: String,
    username: String,
    emotes: Vec<Emote>
}

impl MethodType for MessageReceived {
    fn method_id() -> MethodId {
        "Custom.MessageReceived".into()
    }
}

impl CustomEvent for MessageReceived {}

async fn youtube_ingest(browser: Arc<Browser>, stream_id: String, message_client: Arc<Mutex<Client>>) {
    let page = browser.new_page(format!("https://www.youtube.com/live_chat?is_popout=1&v={}", stream_id).as_str()).await.expect("Couldn't create new tab");

    let _ = page.execute(AddBindingParams::new("message_received")).await;
    page.wait_for_navigation().await.expect("Failed to navigate");
    page.evaluate(r#"
        let blah = document.querySelector('#item-offset');
        const observer = new MutationObserver(mutations => {
            for (const mutation of mutations) {
                if (mutation.addedNodes.length !== 0) {
                    mutation.addedNodes.forEach(element => {
                        if (element.tagName == "YT-LIVE-CHAT-TEXT-MESSAGE-RENDERER") {
                            let emotes = [];
                            let message = element.querySelector('#message');
                            for (child of message.children) {
                                if (child.tagName == 'IMG') {
                                    let name = "";
                                    let url = "";

                                    for (attr of child.attributes) {
                                        if (attr.name == 'shared-tooltip-text') {
                                            name = attr.value;
                                            continue;
                                        }
                                        if (attr.name == 'src') {
                                            url = attr.value
                                        }
                                    }

                                    emotes.push({
                                        url: url,
                                        name: name
                                    });
                                }
                            }
                            let rawHTML = message.innerHTML;
                            let cleaned_msg = rawHTML.replace(/<.*?shared-tooltip-text="(.*?)".*?>/gm, " $1 ");
                            cleaned_msg = cleaned_msg.replace(/<.*?>/gm, "");

                            let data = {
                                message: rawHTML,
                                raw_message: cleaned_msg,
                                username: element.querySelector('#author-name').innerText,
                                emotes: emotes
                            }

                            message_received(JSON.stringify(data));
                        }
                    });
                }
            }
        });
        observer.observe(blah, {childList: true, subtree: true});
    "#).await.expect("Failed to create MutationObserver");

    let mut events = page.event_listener::<EventBindingCalled>().await.expect("");
    while let Some(event) = events.next().await {
        let data: MessageReceived = serde_json::from_str(&event.payload).unwrap();

        let message = Message{
            message: data.message.clone(),
            raw_message: data.raw_message.clone(),
            username: data.username.clone(),
            user_color_r: "FF".to_string(),
            user_color_g: "00".to_string(),
            user_color_b: "00".to_string(),
            from: "Youtube".to_string(),
            source_badge_large: "https://www.youtube.com/s/desktop/f9ccd8c6/img/favicon_32x32.png".to_string(),
            source_badge_small: "https://www.youtube.com/s/desktop/f9ccd8c6/img/favicon.ico".to_string(),
            user_badges: vec![],
            message_emotes: data.emotes.clone(),
        };

        let message_json = serde_json::to_string(&message).unwrap();
        println!("{}", message_json);
        message_client.lock().await.publish_message(message_json).await;
    }
}


pub async fn start_youtube_loop(config: Config, mut rx: Receiver<()>, message_client: Arc<Mutex<Client>>, handle: Handle) {

    let (base_browser, mut handler) = Browser::launch(BrowserConfig::builder().arg("--user-agent=\"Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36\"").build().expect("")).await.expect("");
    let browser = Arc::new(base_browser);

    tokio::task::spawn(async move {
        loop {
            let _event = handler.next().await.unwrap();
        }
    });

    loop {
        rx.recv().await;
        let locked_config = config.lock().unwrap();
        let stream_id = locked_config.get("stream_id");
        if stream_id.is_some() {
            println!("Connecting to Stream with ID of {}", stream_id.unwrap());
            handle.spawn(youtube_ingest(browser.clone(), stream_id.unwrap().to_string(), message_client.clone()));
        }
    }
}