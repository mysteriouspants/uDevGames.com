// pub mod attachments;
pub mod gh_oauth;
pub mod homepage;
// pub mod jam_entries;
// pub mod jams;

use std::future::Ready;
use serde::Serialize;
use actix_web::{Error as ActixError, HttpRequest, HttpResponse, Responder, http::header::ContentType};
use actix_web::http::StatusCode;
use thiserror::Error;

use crate::view::tera;

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

#[derive(Debug, Serialize)]
struct ErrorContext {
    message: String,
    suppress_auth_controls: bool,
}

impl ErrorContext {
    pub fn new(code: i32, message: &str) -> Self {
        Self {
            message: format!("{}: {}", code, message),
            suppress_auth_controls: true,
        }
    }
}

impl Responder for HandlerError {
    type Error = ActixError;
    type Future = Ready<Result<HttpResponse, ActixError>>;

    fn respond_to(self, req: &HttpRequest) -> Self::Future {
        let status_code = match self {
            HandlerError::AttachmentStorageError(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            },
            HandlerError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            HandlerError::PoolError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            HandlerError::HttpError(_) => StatusCode::InternalServerError,
            HandlerError::ParseError(_) => StatusCode::InternalServerError,
            HandlerError::ApprovalStateParseError(_) => {
                StatusCode::InternalServerError
            }
            HandlerError::DieselError(_) => StatusCode::InternalServerError,
            HandlerError::NotFound => StatusCode::NotFound,
        };

        let response = {
            #[derive(Debug, Serialize)]
            struct ErrorContext {
                message: String,
                suppress_auth_controls: bool,
            }

            let error_context = ErrorContext {
                message: format!("{}! Could not continue with error {}.", status_code, self),
                suppress_auth_controls: true,
            };

            HttpResponse::build_from(status_code)
                .content_type(ContentType::html())
                .body(tera().render("error_page", &error_context))
        };

        Ok(response)
    }
}
