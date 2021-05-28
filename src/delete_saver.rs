use random_color::{Luminosity, RandomColor};
use reqwest::{
    multipart::{Form, Part},
    Body,
};
use serde::Deserialize;
use serenity::{
    client::Context,
    model::{channel::Embed, id::MessageId},
    utils::Color,
};

use crate::{MessagePool, DELETE_WEBHOOK_ID, DELETE_WEBHOOK_TOKEN};

#[derive(Debug, Deserialize)]
struct AnonFiles {
    status: bool,
    data: Option<Data>,
    error: Option<Error>,
}

#[derive(Debug, Deserialize)]
struct Error {
    message: String,
    #[serde(rename = "type")]
    kind: String,
    code: u32,
}

#[derive(Debug, Deserialize)]
struct Data {
    file: File,
}

#[derive(Debug, Deserialize)]
struct File {
    url: Url,
    metadata: Metadata,
}

#[derive(Debug, Deserialize)]
struct Url {
    short: String,
}

#[derive(Debug, Deserialize)]
struct Metadata {
    size: Size,
}

#[derive(Debug, Deserialize)]
struct Size {
    bytes: usize,
    readable: String,
}

pub async fn handle_delete(ctx: Context, id: MessageId) {
    let webhook = match ctx
        .http
        .get_webhook_with_token(*DELETE_WEBHOOK_ID, &DELETE_WEBHOOK_TOKEN)
        .await
    {
        Ok(n) => n,
        Err(e) => {
            error!("Failed to get webhook: {}", e);
            return;
        }
    };

    let message = {
        let mut data = ctx.data.write().await;
        let message_pool = data.get_mut::<MessagePool>().unwrap();

        let (i, message) = {
            let mut iter = message_pool.iter().enumerate();

            loop {
                match iter.next() {
                    Some((i, n)) => {
                        if n.id == id {
                            break (i, n.to_owned());
                        }
                    }
                    None => return,
                }
            }
        };

        message_pool.remove(i);
        message
    };

    let mut embeds = Vec::new();
    let rgb = RandomColor::new()
        .luminosity(Luminosity::Light)
        .seed(rand::random::<i32>())
        .alpha(1.0)
        .to_rgb_array();

    let color = Color::from_rgb(rgb[0] as u8, rgb[1] as u8, rgb[2] as u8);

    embeds.push(Embed::fake(|e| {
        e.title("Deleted Message")
            .color(color)
            .field("Message ID", id.0, true)
            .field("User ID", message.author.id.0, true)
            .field("User Tag", message.author.tag(), true);

        if message.content != "" {
            e.field("Message Content", &message.content, false);
        }

        e
    }));

    for (i, attachment) in message.attachments.into_iter().enumerate() {
        let client = reqwest::Client::new();

        let (result, bytes) = match client.get(&attachment.url).send().await {
            Ok(n) => match n.content_length() {
                Some(len) => {
                    let file_name = n
                        .headers()
                        .get("content-disposition")
                        .map(|x| x.to_str().unwrap().to_owned())
                        .unwrap_or("file".to_string());

                    match client
                        .post("https://api.anonfiles.com/upload")
                        .multipart(
                            Form::new().part(
                                "file",
                                Part::stream_with_length(Body::wrap_stream(n.bytes_stream()), len)
                                    .file_name(file_name),
                            ),
                        )
                        .send()
                        .await
                    {
                        Ok(n) => match n.json::<AnonFiles>().await {
                            Ok(n) => {
                                if n.data.is_some() {
                                    (Some(n), Some(len))
                                } else {
                                    error!("AnonFiles error: {:?}", n.error);
                                    (None, Some(len))
                                }
                            }
                            Err(e) => {
                                error!("Failed to get response from AnonFiles: {}", e);
                                (None, Some(len))
                            }
                        },
                        Err(e) => {
                            error!("Failed to make POST request: {}", e);
                            (None, Some(len))
                        }
                    }
                }
                None => {
                    error!("content-length not found");
                    (None, None)
                }
            },
            Err(e) => {
                error!("Failed to make GET request: {}", e);
                (None, None)
            }
        };

        embeds.push(Embed::fake(|e| {
            e.title(format!("Attachment #{}", i + 1))
                .color(color)
                .field("ID", attachment.id, true);

            if attachment.content_type.is_some() {
                e.field("Content Type", attachment.content_type.unwrap(), true);
            }

            if attachment.width.is_some() {
                e.field(
                    "Dimensions (wxh)",
                    format!(
                        "{}x{}",
                        attachment.width.unwrap(),
                        attachment.height.unwrap()
                    ),
                    true,
                );

                e.image(&attachment.url);
            }

            e.field("Filename", attachment.filename, true);

            match result {
                Some(n) => {
                    let size = &n.data.as_ref().unwrap().file.metadata.size;
                    e.field("Size", format!("{} ({})", size.readable, size.bytes), true);
                    e.field("URL", attachment.url, false);
                    e.field("AnonFile URL", n.data.unwrap().file.url.short, false)
                }
                None => {
                    match bytes {
                        Some(n) => {
                            e.field("Size", n, true);
                        }
                        None => {}
                    }

                    e.field("URL", attachment.url, false)
                }
            }
        }));
    }

    match webhook
        .execute(&ctx.http, false, |e| e.embeds(embeds))
        .await
    {
        Ok(_) => {}
        Err(e) => error!("Failed to execute webhook: {}", e),
    }
}
