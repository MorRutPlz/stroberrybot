use serenity::{
    client::Context,
    model::{channel::Embed, event::TypingStartEvent},
};

use crate::{TYPING_WEBHOOK_ID, TYPING_WEBHOOK_TOKEN};

pub async fn handle_typing(ctx: Context, typing: TypingStartEvent) {
    let tag = typing
        .user_id
        .to_user(&ctx.http)
        .await
        .map(|x| x.tag())
        .unwrap_or("unknown-user".to_string());

    let channel = typing
        .channel_id
        .name(&ctx.cache)
        .await
        .unwrap_or("unknown-channel".to_string());

    let webhook = match ctx
        .http
        .get_webhook_with_token(*TYPING_WEBHOOK_ID, &TYPING_WEBHOOK_TOKEN)
        .await
    {
        Ok(n) => n,
        Err(e) => {
            error!("Failed to get webhook: {}", e);
            return;
        }
    };

    let mut embeds = Vec::new();

    embeds.push(Embed::fake(|e| {
        e.title("Typing")
            .field("User ID", typing.user_id.0, true)
            .field("User Tag", tag, true)
            .field("Channel ID", typing.channel_id.0, true)
            .field("Channel Name", channel, true)
    }));

    match webhook
        .execute(&ctx.http, false, |e| e.embeds(embeds))
        .await
    {
        Ok(_) => {}
        Err(e) => error!("Failed to execute webhook: {}", e),
    }
}
