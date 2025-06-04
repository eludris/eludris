use rocket::Route;

mod delete_emoji;
mod edit_emoji;
mod get_emoji;

pub fn get_routes() -> Vec<Route> {
    routes![
        get_emoji::get_emoji,
        edit_emoji::edit_emoji,
        delete_emoji::delete_emoji,
    ]
}
