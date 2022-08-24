//! The Eludris rest api.

mod messages;

use rocket_cors::{AllowedOrigins, CorsOptions};
use sqlx::MySqlPool;
use std::{str::FromStr, sync::Arc};
use tokio::sync::Mutex;

use crate::models::client::Clients;

/// A function that starts the rest api.
pub async fn start(clients: Clients, db: Arc<Mutex<MySqlPool>>) -> Result<(), rocket::Error> {
    // Cors stuff.
    let cors = CorsOptions {
        allowed_origins: AllowedOrigins::all(),
        allowed_methods: vec!["GET", "POST"]
            .iter()
            .map(|m| FromStr::from_str(m).expect("Couldn't generate cors methods."))
            .collect(),
        ..Default::default()
    }
    .to_cors()
    .expect("Couldn't create CorsOptions struct.");

    // The rest API.
    rocket::build()
        .mount("/", routes![messages::index])
        .manage(clients)
        .manage(db)
        .attach(cors)
        .launch()
        .await?; // Ah yes, rocket::Rocket has a must_use lint!

    Ok(())
}
