use std::time::{Duration, SystemTime, UNIX_EPOCH};

use humantime::format_duration;
use serenity::{client::Context, model::channel::Message};

use crate::commands::response;

pub async fn execute(ctx: Context, args: String, msg: Message) {
    let timestamp = match args.parse::<u64>() {
        Ok(n) => (n >> 22) + 1420070400000,
        Err(_) => {
            msg.clone()
                .edit(&ctx.http, |m| {
                    m.content(response(&msg.content, "Failed to parse snowflake as u64"))
                })
                .await
                .ok();

            return;
        }
    };

    let time = UNIX_EPOCH + Duration::from_millis(timestamp);
    let difference = SystemTime::now().duration_since(time).unwrap();

    msg.clone()
        .edit(&ctx.http, |m| {
            m.content(response(
                &msg.content,
                &format_duration(difference).to_string(),
            ))
        })
        .await
        .ok();
}
