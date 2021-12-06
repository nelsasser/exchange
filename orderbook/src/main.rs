mod orderbook;

use std::{env, io::{self, Write}};
use google_cloud::pubsub;
use google_cloud::authorize::ApplicationCredentials;
use serde::{Serialize, Deserialize};

use orderbook::book::OrderBook;
use crate::orderbook::book::{BookRequest, BookResult};

macro_rules! assert_ok {
    ($expr:expr) => {
        match $expr {
            Ok(value) => value,
            Err(err) => {
                panic!("asserted result is an Error: {}", err);
            }
        }
    };
}

macro_rules! assert_some {
    ($expr:expr) => {
        match $expr {
            Some(value) => value,
            None => {
                panic!("asserted result is None");
            }
        }
    };
}

#[derive(Serialize, Deserialize)]
struct Events {
    asset: String,
    events: Vec<BookResult>
}

#[tokio::main]
async fn main() {
    let asset = env::var("ASSET_KEY").expect("Unable to find ASSET KEY");

    println!("Setting up Google pub/sub for the asset {}.", asset);
    io::stdout().flush().unwrap();
    let sub_name = format!("{}-sub", asset);
    let events_topic = format!("{}-Events", asset);

    let mut client = assert_ok!(setup_client().await);
    let mut topic = assert_some!(assert_ok!(client.topic(&events_topic).await));
    let mut subscription = assert_some!(assert_ok!(client.subscription(&sub_name).await));

    println!("Creating orderbook for asset {}", asset);
    io::stdout().flush().unwrap();
    let mut orderbook = OrderBook::new();

    loop {
        if let Some(mut msg) = subscription.receive().await {
            assert_ok!(msg.ack().await);

            // read the message as a book request
            if let Ok(book_request) = serde_json::from_slice::<BookRequest>(msg.data()) {
                let events = Events {
                    asset: asset.clone(),
                    events: orderbook.process_request(book_request),
                };

                let out_msg = assert_ok!(serde_json::to_vec(&events));

                assert_ok!(topic.publish(out_msg).await);

                println!("Processed!");
            } else {
                println!("Failed to parse {} into a book request", std::str::from_utf8(msg.data()).unwrap());
            }
        }
    }

}

fn load_creds() -> ApplicationCredentials {
    let pth = std::path::Path::new("C:\\Users\\elsan\\Documents\\exchange\\pubsub_keys.json");
    if pth.exists() {
        let data = assert_ok!(std::fs::read_to_string(pth));
        std::env::set_var("RUST_GOOGLE_APPLICATION_CREDENTIALS", data);
    }
    let creds = std::env::var("RUST_GOOGLE_APPLICATION_CREDENTIALS")
        .expect("env RUST_GOOGLE_APPLICATION_CREDENTIALS not set");
    serde_json::from_str::<ApplicationCredentials>(&creds)
        .expect("incorrect application credentials format")
}

async fn setup_client() -> Result<pubsub::Client, pubsub::Error> {
    let creds = load_creds();
    pubsub::Client::from_credentials("project-steelieman", creds).await
}
