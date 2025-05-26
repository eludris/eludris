use rocket::Route;

mod create;
mod create_category;
mod create_channel;
mod delete_category;
mod delete_channel;
mod edit_category;
mod edit_channel;
mod get;
mod join;

pub fn get_routes() -> Vec<Route> {
    routes![
        create_category::create_category,
        create::create_sphere,
        create_channel::create_channel,
        delete_category::delete_category,
        edit_category::edit_category,
        delete_channel::delete_channel,
        edit_channel::edit_channel,
        get::get_sphere,
        join::join_sphere,
    ]
}
