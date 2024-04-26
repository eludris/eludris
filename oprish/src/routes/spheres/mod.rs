use rocket::Route;

mod create;
mod get;

pub fn get_routes() -> Vec<Route> {
    routes![
        create::create_sphere,
        get::get_sphere,
        get::get_sphere_from_slug
    ]
}
