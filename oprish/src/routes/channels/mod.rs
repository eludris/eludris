pub mod add_reaction;
pub mod clear_reactions;
pub mod create_message;
pub mod delete_message;
pub mod edit_message;
pub mod get;
pub mod get_message;
pub mod get_messages;
pub mod remove_reaction;

use rocket::Route;

pub fn get_routes() -> Vec<Route> {
    routes![
        get::get_channel,
        get_messages::get_messages,
        get_message::get_message,
        create_message::create_message,
        delete_message::delete_message,
        edit_message::edit_message,
        add_reaction::add_reaction,
        remove_reaction::remove_reaction,
        clear_reactions::clear_reactions,
    ]
}
