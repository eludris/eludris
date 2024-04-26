use rocket::Route;

mod create;

pub fn get_routes() -> Vec<Route> {
    routes![create::create_sphere]
}
