pub mod create_message;
pub mod get;
pub mod get_messages;

use rocket::Route;

pub fn get_routes() -> Vec<Route> {
    routes![
        get::get_channel,
        get_messages::get_messages,
        create_message::create_message
    ]
}
