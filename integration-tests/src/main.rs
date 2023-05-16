use std::env;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Error, Result};
use futures::future::try_join_all;
use futures::stream::{SplitSink, SplitStream, StreamExt};
use futures::SinkExt;
use rand::{rngs::StdRng, Rng, SeedableRng};
use reqwest::header::HeaderValue;
use todel::models::{ClientPayload, InstanceInfo, ServerPayload};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::time;
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
            let (rx, tx) = connect_gateway(&state, client_id).await?;
            Ok::<(), Error>(())
        }
    }))
    .await?;

    Ok(())
}

async fn connect_gateway(
    state: &Arc<State>,
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
    let ip = format!("192.168.100.{}", client_id);
    log::trace!("Connected to pandemonium as {}", ip);
    request
        .headers_mut()
        .insert("X-Real-IP", HeaderValue::from_str(&ip)?);
    let (socket, _) = connect_async(request).await?;
    let (tx, mut rx) = socket.split();
    let tx = Arc::new(Mutex::new(tx));
    loop {
        if let Some(Ok(WSMessage::Text(message))) = rx.next().await {
            if let Ok(ServerPayload::Hello {
                heartbeat_interval, ..
            }) = serde_json::from_str(&message)
            {
                let tx = Arc::clone(&tx);
                let starting_beat = state.rng.lock().await.gen_range(0..heartbeat_interval);
                tokio::spawn(async move {
                    time::sleep(Duration::from_millis(starting_beat)).await;
                    loop {
                        tx.lock()
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
                break;
            }
        }
    }
    Ok((tx, rx))
}

#[cfg(test)]
mod tests {
    use super::main;

    #[test]
    fn integration_tests() {
        main().unwrap();
    }
}
