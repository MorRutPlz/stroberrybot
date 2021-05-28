#[macro_use]
mod logger;

mod commands;
mod delete_saver;
mod handler;
mod typing_saver;

use dotenv::dotenv;
use lazy_static::lazy_static;
use serenity::{client::Client, model::channel::Message, prelude::TypeMapKey};
use std::{collections::VecDeque, env::var};

use crate::handler::Handler;

lazy_static! {
    pub static ref USER_TOKEN: String = {
        dotenv().ok();
        var("USER_TOKEN").unwrap()
    };
    pub static ref USER_ID: u64 = var("USER_ID").unwrap().parse().unwrap();
    pub static ref DELETE_WEBHOOK_ID: u64 = var("DELETE_WEBHOOK_ID").unwrap().parse().unwrap();
    pub static ref DELETE_WEBHOOK_TOKEN: String = var("DELETE_WEBHOOK_TOKEN").unwrap();
    pub static ref TYPING_WEBHOOK_ID: u64 = var("TYPING_WEBHOOK_ID").unwrap().parse().unwrap();
    pub static ref TYPING_WEBHOOK_TOKEN: String = var("TYPING_WEBHOOK_TOKEN").unwrap();
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();

    info!("Starting bot");

    let mut client = Client::builder(USER_TOKEN.as_str())
        .event_handler(Handler)
        .await
        .expect("Error creating client");

    client
        .data
        .write()
        .await
        .insert::<MessagePool>(VecDeque::new());

    if let Err(why) = client.start().await {
        error!("An error occurred while running the client: {:?}", why);
    }
}

struct MessagePool;

impl TypeMapKey for MessagePool {
    type Value = VecDeque<Message>;
}
