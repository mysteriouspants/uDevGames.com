// pub mod attachments;
pub mod gh_oauth;
pub mod homepage;
// pub mod jam_entries;
// pub mod jams;

use serde::Serialize;
use actix_web::{Error as ActixError, HttpResponse, ResponseError, http::header::ContentType};
use actix_web::http::StatusCode;
use thiserror::Error;

use crate::{template_helpers::AuthFromSessionError, view::render_template};

/// Unified error type for most (all?) handlers. Puts all the annoying
/// boilerplate of derives into one spot with a single implementation of
/// Responder.
///
/// Note that it would be very tempting to use anyhow for this, however we
/// cannot implement Responder for it. Even if we could, inferring the http
/// status code from a Boxed error would be rather challenging.
#[derive(Debug, Error)]
pub enum HandlerError {
    #[error("The resource was not found")]
    NotFound,

    #[error("Could not get a connection from the pool with error {0}")]
    PoolError(#[from] diesel::r2d2::PoolError),

    #[error("Failed to query the database with error {0}")]
    DatabaseError(#[from] crate::models::ModelError),

    #[error("Failed to extract data from session with error {0}")]
    SessionError(#[from] ActixError),

    #[error("Failed to store/retrieve attachment with error {0}")]
    AttachmentStorageError(#[from] crate::attachments::AttachmentStorageError),

    #[error("HTTP Error {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("Parse Error {0}")]
    ParseError(#[from] chrono::ParseError),

    #[error("Diesel Error {0}")]
    DieselError(#[from] diesel::result::Error),

    #[error("Approval State Parse Error {0}")]
    ApprovalStateParseError(#[from] crate::models::ApprovalStateParseError),
}

impl ResponseError for HandlerError {
    fn status_code(&self) -> StatusCode {
        match self {
            HandlerError::AttachmentStorageError(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            },
            HandlerError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            HandlerError::PoolError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            HandlerError::HttpError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            HandlerError::ParseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            HandlerError::ApprovalStateParseError(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
            HandlerError::DieselError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            HandlerError::NotFound => StatusCode::NOT_FOUND,
            HandlerError::SessionError(_) => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let status_code = self.status_code();

        #[derive(Debug, Serialize)]
        struct ErrorContext {
            message: String,
            suppress_auth_controls: bool,
        }

        let error_context = ErrorContext {
            message: format!("{}! Could not continue with error {}.", status_code, self),
            suppress_auth_controls: true,
        };

        HttpResponse::build(status_code)
            .set(ContentType::html())
            .body(render_template("error_page.html.tera", &error_context))
    }
}

impl From<AuthFromSessionError> for HandlerError {
    fn from(error: AuthFromSessionError) -> Self {
        match error {
            AuthFromSessionError::DbQueryError(e) =>
                HandlerError::DatabaseError(e),
            AuthFromSessionError::SessionRetrieveError(e) =>
                HandlerError::SessionError(e)
        }
    }
}
