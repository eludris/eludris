use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::{Header, Method, Status};
use rocket::{Request, Response};

pub struct Cors;

#[rocket::async_trait]
impl Fairing for Cors {
    fn info(&self) -> Info {
        Info {
            name: "Add CORS headers to responses",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new(
            "Access-Control-Allow-Methods",
            "POST, GET, OPTIONS, PATCH, DELETE",
        ));
        response.set_header(Header::new("Access-Control-Allow-Headers", "Authorization"));
        response.set_header(Header::new(
            "Access-Control-Expose-Headers",
            "X-Ratelimit-Last-Reset, X-Ratelimit-Max, X-Ratelimit-Request-Count, X-Ratelimit-Reset",
        ));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));

        if request.method() == Method::Options && response.status() == Status::NotFound {
            response.set_status(Status::NoContent);
        }
    }
}
