//! Messaging related routes.

use rocket::{futures::SinkExt, serde::json::Json, State};
use serde_json::to_string;
use tokio_tungstenite::tungstenite::Message;

use crate::models::client::Clients;
use crate::models::message::{ClientMessage, MessageRespose};

/// The route to send a message and for it to be echoed to all connected websocket clients.
#[post("/", format = "json", data = "<message>")]
pub async fn index(clients: &State<Clients>, message: Json<ClientMessage>) -> Json<MessageRespose> {
    // Convert the message once.
    let response = MessageRespose::new(&*message);
    let message = to_string(&message.into_inner()).expect("Couldn't conver the message to json.");
    let mut clients = clients.lock().await;
    for client in clients.iter_mut() {
        client
            .tx
            .lock()
            .await
            .send(Message::Text(message.clone())) // Clone it for every client.
            .await
            .expect("Couldn't send the message.");
    }
    Json(response)
}
