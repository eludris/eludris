use rocket::Route;

mod create;
mod create_channel;
mod get;
mod join;

pub fn get_routes() -> Vec<Route> {
    routes![
        create::create_sphere,
        create_channel::create_channel,
        get::get_sphere,
        get::get_sphere_from_slug,
        join::join_sphere,
        join::join_sphere_from_slug,
    ]
}
