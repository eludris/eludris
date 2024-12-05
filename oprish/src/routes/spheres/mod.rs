use rocket::Route;

mod create;
mod create_category;
mod create_channel;
mod delete_category;
mod edit_category;
mod get;
mod join;

pub fn get_routes() -> Vec<Route> {
    routes![
        create_category::create_category,
        create::create_sphere,
        create_channel::create_channel,
        delete_category::delete_category,
        edit_category::edit_category,
        get::get_sphere,
        get::get_sphere_from_slug,
        join::join_sphere,
        join::join_sphere_from_slug,
    ]
}
