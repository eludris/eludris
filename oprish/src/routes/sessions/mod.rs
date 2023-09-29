mod create;
mod delete;
mod get;

use rocket::Route;

pub fn get_routes() -> Vec<Route> {
    routes![
        create::create_session,
        get::get_sessions,
        delete::delete_session
    ]
}
