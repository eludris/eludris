use rocket::{
    http::Status,
    request::Request,
    response::{self, Responder},
    serde::json::Json,
};

use crate::models::ErrorResponse;

impl<'r> Responder<'r, 'static> for ErrorResponse {
    fn respond_to(self, req: &'r Request<'_>) -> response::Result<'static> {
        let status = self.shared().status;
        response::Response::build_from(Json(self).respond_to(req)?)
            .status(Status::from_code(status).unwrap())
            .ok()
    }
}
