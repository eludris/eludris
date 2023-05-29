use rocket::{
    http::Status,
    request::Request,
    response::{self, Responder},
    serde::json::Json,
};

use crate::models::ErrorResponse;

impl<'r> Responder<'r, 'static> for ErrorResponse {
    fn respond_to(self, req: &'r Request<'_>) -> response::Result<'static> {
        let status = match &self {
            ErrorResponse::Unauthorized { shared, .. } => shared.status,
            ErrorResponse::Forbidden { shared, .. } => shared.status,
            ErrorResponse::NotFound { shared, .. } => shared.status,
            ErrorResponse::Validation { shared, .. } => shared.status,
            ErrorResponse::RateLimited { shared, .. } => shared.status,
            ErrorResponse::Server { shared, .. } => shared.status,
        };
        response::Response::build_from(Json(self).respond_to(req)?)
            .status(Status::from_code(status).unwrap())
            .ok()
    }
}
