// mod admin_only;
mod attachment_context;
mod breadcrumbs;
mod jam_context;
mod user_optional;
// mod user_required;

use actix_session::Session;
use actix_web::Error as ActixError;
use serde::Serialize;

pub use crate::template_helpers::{
    /* admin_only::*, */ attachment_context::*, breadcrumbs::*,
    jam_context::*, user_optional::*, /* user_required::*, */
};
use crate::{
    db::DbConn,
    models::{GhUserRecord, ModelError, Permission},
};
use thiserror::Error;

#[derive(Debug, Serialize)]
struct TemplateContextUser {
    /// The user's numeric id.
    id: i64,

    /// The user's Github login.
    login: String,

    /// The user's Github profile link.
    html_url: String,

    /// The user's avatar.
    avatar_url: String,

    /// List of the user's permissions.
    permissions: Vec<String>,
}

#[derive(Debug, Error)]
pub enum AuthFromSessionError {
    #[error("Could not parse uid from cookie with error {0}")]
    SessionRetrieveError(#[from] ActixError),

    #[error("Could not query the database with error {0}")]
    DbQueryError(#[from] ModelError),
}

fn auth_from_session(
    conn: &DbConn,
    session: &Session,
) -> Result<Option<(GhUserRecord, Vec<String>)>, AuthFromSessionError> {
    let uid = match session.get::<i64>("gh_user_id")? {
        Some(uid) => uid,
        None => return Ok(None),
    };

    let user = match GhUserRecord::find_by_id(&conn, uid)? {
        Some(user) => user,
        None => {
            // remove the nonexistent user from the cookie, effectively
            // logging out the user
            session.remove("gh_user_id");
            return Ok(None);
        }
    };

    let permissions = Permission::find_by_gh_user_id(&conn, uid)?
        .iter()
        .map(|p| p.name.clone())
        .collect();

    return Ok(Some((user, permissions)));
}
