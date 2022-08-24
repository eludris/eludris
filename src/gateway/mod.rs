//! The Eludris gateway, also called Pandemonium.

use rocket::{
    futures::{SinkExt, StreamExt},
    tokio::{
        net::{TcpListener, TcpStream},
        select,
        sync::Mutex,
        task,
        time::sleep,
    },
};
use std::{env, net::SocketAddr, sync::Arc, time::Duration};
use tokio_tungstenite::{accept_async, tungstenite::Message};

use crate::{
    models::client::{Client, Clients},
    utils::now_timestamp,
};

/// A simple function that check's if a client's last ping was over 20 seconds ago and closes the
/// gateway connection if so.
async fn check_connection(last_ping: Arc<Mutex<u32>>) {
    loop {
        if (*last_ping.lock().await + 20) < now_timestamp() {
            break;
        }
        sleep(Duration::from_secs(20)).await;
    }
}

/// A function that handles one client connecting and disconnecting.
async fn handle_connection(addr: SocketAddr, stream: TcpStream, clients: Clients) {
    let socket = accept_async(stream)
        .await
        .expect("Couldn't accept the socket stream.");

    let (tx, mut rx) = socket.split();
    let tx = Arc::new(Mutex::new(tx));

    let last_ping = Arc::new(Mutex::new(now_timestamp()));

    {
        let mut clients = clients.lock().await;
        clients.push(Client {
            addr,
            tx: tx.clone(),
        });
    }

    let handle_rx = async {
        while let Some(msg) = rx.next().await {
            log::debug!("New gateway message:\n{:#?}", msg);
            match msg {
                Ok(data) => match data {
                    Message::Ping(x) => {
                        *last_ping.lock().await = now_timestamp();
                        tx.lock()
                            .await
                            .send(Message::Pong(x))
                            .await
                            .expect("Couldn't send pong");
                    }
                    _ => todo!(),
                },
                Err(_) => break,
            }
        }
    };

    select! {
        _ = check_connection(last_ping.clone()) => {},
        _ = handle_rx => {},
    }

    log::info!("Someone disconnected");

    {
        let mut clients = clients.lock().await;
        let idex = clients
            .iter()
            .position(|c| c.addr == addr)
            .expect("Client not found");
        // We use `swap_remove` instead of `remove` here because it has O(1) complexity and we
        // don't really care about the orders of the clients here.
        clients.swap_remove(idex);
    }
}

/// A function that starts & handles the gatway.
pub async fn start(clients: Clients) {
    let gateway_address = format!(
        "{}:{}",
        env::var("gateway_ADDRESS").unwrap_or_else(|_| "127.0.0.1".to_string()),
        env::var("gateway_PORT").unwrap_or_else(|_| "5000".to_string())
    );
    let socket = TcpListener::bind(&gateway_address)
        .await
        .unwrap_or_else(|_| panic!("Couldn't start a websocket on {}", gateway_address));
    log::info!("Gateway started at {}", gateway_address);

    while let Ok((stream, addr)) = socket.accept().await {
        log::info!("New connection on ip {}", addr);
        task::spawn(handle_connection(addr, stream, clients.clone()));
    }
}
