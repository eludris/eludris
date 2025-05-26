use rocket::Route;

mod create;
mod delete;
mod edit;
mod get;
mod profile;
mod resend_verification;
mod reset_password;
mod verify;

pub fn get_routes() -> Vec<Route> {
    routes![
        create::create_user,
        verify::verify_user,
        get::get_self,
        get::get_user,
        edit::edit_user,
        profile::edit_profile,
        delete::delete_user,
        reset_password::create_password_reset_code,
        reset_password::reset_password,
        resend_verification::resend_verification,
    ]
}
