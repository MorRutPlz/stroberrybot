mod snowflake;

use serenity::{client::Context, model::channel::Message};

pub async fn parse_and_execute(ctx: Context, msg: Message) {
    if msg.content.starts_with("/") {
        let mut split = msg.content[1..].split(' ');

        match split.next() {
            Some(command) => match command.to_lowercase().as_str() {
                "snowflake" => snowflake::execute(ctx, msg.content[11..].to_owned(), msg).await,
                _ => {}
            },
            None => {}
        }
    }
}

pub fn response(content: &str, msg: &str) -> String {
    format!("{}\n```{}```", content, msg)
}
