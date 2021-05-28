use serenity::{
    async_trait,
    client::{Context, EventHandler},
    model::{
        channel::Message,
        event::TypingStartEvent,
        id::{ChannelId, GuildId, MessageId},
    },
};

use crate::{
    commands::parse_and_execute, delete_saver::handle_delete, typing_saver::handle_typing,
    MessagePool, USER_ID,
};

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, message: Message) {
        if message.author.id.0 == *USER_ID {
            parse_and_execute(ctx, message).await;
        } else {
            if message.is_private() {
                let message = match message.channel_id.messages(&ctx.http, |e| e.limit(5)).await {
                    Ok(n) => n
                        .into_iter()
                        .filter(|x| x.id == message.id)
                        .collect::<Vec<_>>()[0]
                        .to_owned(),
                    Err(e) => {
                        error!("Failed to get message: {}", e);
                        return;
                    }
                };

                let mut data = ctx.data.write().await;
                let message_pool = data.get_mut::<MessagePool>().unwrap();

                message_pool.push_back(message);

                if message_pool.len() > 1000 {
                    message_pool.pop_front();
                }
            }
        }
    }

    async fn message_delete(
        &self,
        ctx: Context,
        _: ChannelId,
        deleted_message_id: MessageId,
        _: Option<GuildId>,
    ) {
        handle_delete(ctx, deleted_message_id).await;
    }

    async fn typing_start(&self, ctx: Context, typing: TypingStartEvent) {
        handle_typing(ctx, typing).await;
    }
}
