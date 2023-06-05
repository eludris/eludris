use rocket::Route;

mod create;
mod delete;
mod get;
mod profile;
mod update;
mod verify;

pub fn get_routes() -> Vec<Route> {
    routes![
        create::create_user,
        verify::verify_user,
        get::get_self,
        get::get_user,
        get::get_user_with_username,
        update::update_user,
        profile::update_profile,
        delete::delete_user,
    ]
}
