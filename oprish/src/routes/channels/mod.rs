pub mod create_message;
pub mod get;

use rocket::Route;

pub fn get_routes() -> Vec<Route> {
    routes![get::get_channel, create_message::create_message]
}
