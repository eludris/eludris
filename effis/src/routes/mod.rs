mod buckets;
mod index;
mod static_routes;

use rocket::Route;

pub fn routes() -> Vec<Route> {
    routes![
        index::upload_attachment,
        index::get_attachment,
        index::download_attachment,
        index::get_attachment_data,
        buckets::upload_file,
        buckets::get_file,
        buckets::download_file,
        buckets::get_file_data,
    ]
}

pub fn static_routes() -> Vec<Route> {
    routes![
        static_routes::get_static_file,
        static_routes::download_static_file,
    ]
}
