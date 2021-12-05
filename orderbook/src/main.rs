mod orderbook;

use std::{env, io::{self, Write}};
use google_cloud::pubsub;
use google_cloud::authorize::ApplicationCredentials;

use orderbook::book::OrderBook;
use crate::orderbook::book::BookRequest;

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

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.is_empty() {
        panic!("Expected 1 argument for the orderbook asset");
    }

    let asset = args[1].clone();

    println!("Setting up Google pub/sub subscription for the asset {}.", asset);
    io::stdout().flush().unwrap();
    let mut client = assert_ok!(setup_client().await);
    let sub_name = format!("{}-sub", asset);
    let mut subscription = assert_some!(assert_ok!(client.subscription(&sub_name).await));

    println!("Creating orderbook for asset {}", asset);
    io::stdout().flush().unwrap();
    let mut orderbook = OrderBook::new(asset);

    loop {
        match subscription.receive().await {
            Some(mut msg) => {

                msg.ack().await;

                // read the message as a book request
                if let Ok(book_request) = serde_json::from_slice::<BookRequest>(msg.data()) {
                    println!("Received book request {:?}", book_request);
                    let events = orderbook.process_request(book_request);
                    println!("Events {:?}", events);
                } else {
                    println!("Failed to parse {} into a book request", std::str::from_utf8(msg.data()).unwrap());
                }
            },
            _ => (),
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
