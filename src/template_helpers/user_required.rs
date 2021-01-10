use crate::models::GhUserRecord;
use rocket::{http::Status, request::{FromRequest, Outcome, Request}};
use serde::Serialize;
use thiserror::Error;

use super::{AuthFromRequestError, TemplateContextUser, auth_from_request};


pub struct UserRequired {
    user: GhUserRecord,
    permissions: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct UserRequiredContext {
    user: TemplateContextUser,
}

#[derive(Debug, Error)]
pub enum UserRequiredFromRequestError {
    #[error("No user is logged in!")]
    NotLoggedIn,

    #[error("Could not determine if a user is logged in with error {0}")]
    AuthFromRequestError(AuthFromRequestError),
}

impl UserRequired {
    pub fn is_banned(&self) -> bool {
        self.permissions.contains(&"banned".to_string())
    }

    pub fn is_admin(&self) -> bool {
        self.permissions.contains(&"admin".to_string())
    }

    pub fn to_context(&self) -> UserRequiredContext {
        UserRequiredContext {
            user: TemplateContextUser {
                id: self.user.id,
                login: self.user.login.clone(),
                html_url: self.user.html_url.clone(),
                avatar_url: self.user.avatar_url.clone(),
                permissions: self.permissions.clone(),
            }
        }
    }
}

#[rocket::async_trait]
impl<'a, 'r> FromRequest<'a, 'r> for UserRequired {
    type Error = UserRequiredFromRequestError;

    async fn from_request(req: &'a Request<'r>) -> Outcome<Self, Self::Error> {
        match auth_from_request(req) {
            Ok(Some((user, permissions))) => Outcome::Success(UserRequired {
                user, permissions
            }),
            Ok(None) => Outcome::Failure((Status::Unauthorized, UserRequiredFromRequestError::NotLoggedIn)),
            Err(e) => match e {
                AuthFromRequestError::DbPoolError(_) => {
                    Outcome::Failure((Status::InternalServerError, UserRequiredFromRequestError::AuthFromRequestError(e)))
                },
                AuthFromRequestError::UserIdDecodeError(_) => {
                    Outcome::Failure((Status::BadRequest, UserRequiredFromRequestError::AuthFromRequestError(e)))
                },
                AuthFromRequestError::DbQueryError(_) => {
                    Outcome::Failure((Status::BadRequest, UserRequiredFromRequestError::AuthFromRequestError(e)))
                },
            },
        }
    }
}
