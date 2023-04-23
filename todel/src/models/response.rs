use serde::{Deserialize, Serialize};

/// Base type for error responses
#[autodoc(category = "Errors")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub status: u16,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<ErrorData>,
}

/// Preset error types
#[autodoc(category = "Errors")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ErrorData {
    RateLimitedError(RateLimitError),
    FileSizeRateLimitedError(FileSizeRateLimitedError),
    ValidationError(ValidationError),
    NotFoundError(NotFoundError),
    ServerError(ServerError),
}

/// The error when a client is rate limited
#[autodoc(category = "Errors")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitError {
    pub retry_after: u64,
}

/// The error caused when a client surpasses the maximum amount of bytes in an Effis rate limit
/// bucket
#[autodoc(category = "Errors")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSizeRateLimitedError {
    pub retry_after: u64,
    pub bytes_left: u64,
}

/// The error when the supplied request body is invalid
#[autodoc(category = "Errors")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub field_name: String,
    pub error: String,
}

/// The error when the requested resource is not found
#[autodoc(category = "Errors")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotFoundError;

/// The error when the requested resource is not found
#[autodoc(category = "Errors")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerError {
    pub error: String,
}

#[cfg(feature = "logic")]
/// The trait for valid error response data types
pub trait ErrorResponseData {
    fn to_error_response(self) -> ErrorResponse;
}

#[cfg(feature = "logic")]
impl ErrorResponseData for RateLimitError {
    fn to_error_response(self) -> ErrorResponse {
        ErrorResponse {
            status: 429,
            message: "You have been rate limited".to_string(),
            data: Some(ErrorData::RateLimitedError(self)),
        }
    }
}

#[cfg(feature = "logic")]
impl ErrorResponseData for FileSizeRateLimitedError {
    fn to_error_response(self) -> ErrorResponse {
        ErrorResponse {
            status: 429,
            message: "You have surpassed your file size limit".to_string(),
            data: Some(ErrorData::FileSizeRateLimitedError(self)),
        }
    }
}

#[cfg(feature = "logic")]
impl ErrorResponseData for ValidationError {
    fn to_error_response(self) -> ErrorResponse {
        ErrorResponse {
            status: 422,
            message: "Invalid request body".to_string(),
            data: Some(ErrorData::ValidationError(self)),
        }
    }
}

#[cfg(feature = "logic")]
impl ErrorResponseData for NotFoundError {
    fn to_error_response(self) -> ErrorResponse {
        ErrorResponse {
            status: 404,
            message: "The requested resource cannot be found".to_string(),
            data: None,
        }
    }
}

#[cfg(feature = "logic")]
impl ErrorResponseData for ServerError {
    fn to_error_response(self) -> ErrorResponse {
        ErrorResponse {
            status: 500,
            message: "The server encountered an error while performing the requested action"
                .to_string(),
            data: Some(ErrorData::ServerError(self)),
        }
    }
}
