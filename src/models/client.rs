//! Client models.

use rocket::{
    futures::stream::SplitSink,
    tokio::{net::TcpStream, sync::Mutex},
};
use std::{net::SocketAddr, sync::Arc};
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};

/// A struct representing a gateway client.
pub struct Client {
    // `addr` mainly exists so that we can remove the clients from the connected gateway peers later.
    pub addr: SocketAddr,
    pub tx: Arc<Mutex<SplitSink<WebSocketStream<TcpStream>, Message>>>,
}

pub type Clients = Arc<Mutex<Vec<Client>>>;
