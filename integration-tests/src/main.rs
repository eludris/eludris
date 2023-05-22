use std::env;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{bail, Error, Result};
use futures::future::try_join_all;
use futures::stream::{SplitSink, SplitStream, StreamExt};
use futures::SinkExt;
use rand::{rngs::StdRng, Rng, SeedableRng};
use reqwest::header::{self, HeaderValue};
use reqwest::Client;
use todel::models::{ClientPayload, InstanceInfo, Message, ServerPayload};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::time::{self, Instant};
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::{
    connect_async, tungstenite::Message as WSMessage, MaybeTlsStream, WebSocketStream,
};

struct State {
    instance_info: InstanceInfo,
    rng: Mutex<StdRng>,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    env_logger::init();

    let instance_url =
        env::var("INSTANCE_URL").unwrap_or_else(|_| "http://0.0.0.0:7159".to_string());

    let state: Arc<State> = Arc::new(State {
        instance_info: (reqwest::get(instance_url).await?.json().await?),
        rng: Mutex::new(SeedableRng::from_entropy()),
    });

    try_join_all((0..=u8::MAX).map(|client_id| {
        let state = Arc::clone(&state);
        async move {
            let ip = format!("192.168.100.{}", client_id);
            let (_, mut rx) = connect_gateway(&state, &ip, client_id).await?;
            let mut headers = header::HeaderMap::new();
            headers.insert("X-Real-IP", HeaderValue::from_str(&ip)?);
            let client = Client::builder().default_headers(headers).build()?;
            client
                .post(format!("{}/messages", state.instance_info.oprish_url))
                .json(&Message {
                    author: ip.clone(),
                    content: format!("Message from client {}", client_id),
                })
                .send()
                .await?;
            if client_id == 0 {
                log::info!("Sent message");
            }
            let instant = Instant::now();
            let mut received = 0;
            loop {
                if let Some(message) = rx.next().await {
                    if let Ok(WSMessage::Text(message)) = message {
                        if let Ok(ServerPayload::MessageCreate(_)) = serde_json::from_str(&message)
                        {
                            received += 1;
                            if received == u8::MAX {
                                break;
                            }
                        }
                    }
                } else {
                    bail!("Couldn't receive all of the messages");
                }
            }
            if client_id == 0 {
                log::info!(
                    "Received 256 messages in {}ms",
                    instant.elapsed().as_millis()
                );
            }
            Ok::<(), Error>(())
        }
    }))
    .await?;

    Ok(())
}

async fn connect_gateway(
    state: &Arc<State>,
    ip: &str,
    client_id: u8,
) -> Result<(
    Arc<Mutex<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, WSMessage>>>,
    SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
)> {
    let mut request = state
        .instance_info
        .pandemonium_url
        .as_str()
        .into_client_request()?;
    request
        .headers_mut()
        .insert("X-Real-IP", HeaderValue::from_str(ip)?);
    if client_id == 0 {
        log::info!("Connecting to pandemonium");
    }
    let (socket, _) = connect_async(request).await?;
    let (tx, mut rx) = socket.split();
    let tx = Arc::new(Mutex::new(tx));
    loop {
        if let Some(message) = rx.next().await {
            if let Ok(WSMessage::Text(message)) = message {
                if let Ok(ServerPayload::Hello {
                    heartbeat_interval, ..
                }) = serde_json::from_str(&message)
                {
                    let inner_tx = Arc::clone(&tx);
                    let starting_beat = state.rng.lock().await.gen_range(0..heartbeat_interval);
                    tokio::spawn(async move {
                        time::sleep(Duration::from_millis(starting_beat)).await;
                        loop {
                            inner_tx
                                .lock()
                                .await
                                .send(WSMessage::Text(
                                    serde_json::to_string(&ClientPayload::Ping)
                                        .expect("Could not serialise ping payload"),
                                ))
                                .await
                                .expect("Could not send ping payload");
                            time::sleep(Duration::from_millis(heartbeat_interval)).await;
                        }
                    });
                    break Ok((tx, rx));
                }
            }
        } else {
            bail!("Could not find `Hello` Payload");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::main;

    #[test]
    fn integration_tests() {
        main().unwrap();
    }
}
